use std::{
    collections::{HashMap, HashSet},
    io,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use actix_web::web;
use rand::{thread_rng, Rng as _};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

use crate::{db, types::DbPool, ConnId, Msg, RoomId, UserId};

// type ListRoom = Vec<>
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct WsRoom {
    room_id: String,
    users: HashSet<(ConnId, UserId)>,
}

// A command received by the ChatServer
#[derive(Debug)]
enum Command {
    Connect {
        conn_tx: mpsc::UnboundedSender<Msg>,
        res_tx: oneshot::Sender<ConnId>,
        user_id: String,
    },

    Disconnect {
        conn: ConnId,
    },

    List {
        res_tx: oneshot::Sender<Vec<WsRoom>>,
    },

    // TODO
    // CreateRoom {
    //     conn: ConnId,
    //     room: RoomId,
    //     res_tx: oneshot::Sender<()>,
    // },

    // DeleteRoom {
    //     conn: ConnId,
    //     room: RoomId,
    //     res_tx: oneshot::Sender<()>,
    // },
    Join {
        conn: ConnId,
        room: RoomId,
        res_tx: oneshot::Sender<()>,
    },

    Exit {
        conn: ConnId,
        room: RoomId,
        res_tx: oneshot::Sender<()>,
    },

    Message {
        msg: Msg,
        conn: ConnId,
        room_id: RoomId,
        res_tx: oneshot::Sender<()>,
    },

    Broadcast {
        msg: Msg,
        conn: ConnId,
        res_tx: oneshot::Sender<()>,
    },
}

#[derive(Debug)]
pub struct ChatServer {
    /// Map of connection IDs to their message receivers.
    sessions: HashMap<ConnId, (mpsc::UnboundedSender<Msg>, UserId)>,

    /// Map of room name to participant IDs in that room.
    rooms: HashMap<RoomId, HashSet<ConnId>>,

    /// Tracks total number of historical connections established.
    visitor_count: Arc<AtomicUsize>,

    /// Command receiver.
    cmd_rx: mpsc::UnboundedReceiver<Command>,

    pool: DbPool,
}

impl ChatServer {
    pub fn new(pool: DbPool) -> (Self, ChatServerHandle) {
        let mut rooms = HashMap::new();

        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

        (
            Self {
                sessions: HashMap::new(),
                rooms,
                visitor_count: Arc::new(AtomicUsize::new(0)),
                cmd_rx,
                pool,
            },
            ChatServerHandle { cmd_tx },
        )
    }

    async fn broadcast(&self, conn: ConnId, msg: impl Into<Msg>) {
        let msg = msg.into();

        for (conn_id, (tx, _)) in &self.sessions {
            if *conn_id == conn {
                continue;
            }
            tx.send(msg.clone());
        }
    }

    /// Send message to users in a room.
    ///
    /// `skip` is used to prevent messages triggered by a connection also being received by it.
    async fn send_system_message(&self, room: &str, skip: ConnId, msg: impl Into<Msg>) {
        if let Some(sessions) = self.rooms.get(room) {
            let msg = msg.into();

            for conn_id in sessions {
                if *conn_id == skip {
                    continue;
                }
                println!("send message {msg} to session:{}", conn_id);
                if let Some((tx, _)) = self.sessions.get(conn_id) {
                    tx.send(msg.clone());
                }
            }
        }
    }

    /// Send message to all other users in current room.
    ///
    /// `conn` is used to find current room and prevent messages sent by a connection also being
    /// received by it.
    async fn send_mesage(&self, conn: ConnId, room_id: RoomId, msg: impl Into<Msg>) {
        // if let Some(room) = self
        //     .rooms
        //     .iter()
        //     .find_map(|(room, participants)| participants.contains(&conn).then_some(room))
        // {
        //     self.send_system_message(room, conn, msg).await;
        // }
        self.send_system_message(&room_id, conn, msg).await;
    }

    /// Register new session and assign unique ID to this session
    async fn connect(&mut self, tx: mpsc::UnboundedSender<Msg>, user_id: UserId) -> ConnId {
        // register session with random connection ID
        let id = thread_rng().gen::<ConnId>();
        self.sessions.insert(id, (tx, user_id.clone()));

        let pool = self.pool.clone();

        // TODO: 1.join all rooms joined by the user
        // 2. broadcast to all rooms

        // join all rooms joined by the user
        let joined_rooms = web::block(move || {
            let mut conn = pool.get()?;
            db::rooms::get_user_joined_rooms(&mut conn, user_id)
        })
        .await
        .unwrap()
        .unwrap();

        for room in joined_rooms {
            self.rooms.entry(room.id).or_default().insert(id);
        }

        id
    }

    /// Unregister connection from room map and broadcast disconnection message.
    async fn disconnect(&mut self, conn_id: ConnId) {
        let mut rooms: Vec<RoomId> = Vec::new();

        if self.sessions.remove(&conn_id).is_some() {
            for (room_id, sessions) in &mut self.rooms {
                if sessions.remove(&conn_id) {
                    rooms.push(room_id.to_owned());
                }
            }
        }

        for room in rooms {
            self.send_system_message(&room, 0, "Someone disconnected")
                .await;
        }
    }

    fn list_rooms(&mut self) -> Vec<WsRoom> {
        self.rooms
            .iter()
            .map(|(room_id, conn_ids)| WsRoom {
                room_id: room_id.to_owned(),
                users: conn_ids
                    .iter()
                    .map(|conn_id| {
                        let user_id = if let Some((_, user_id)) = self.sessions.get(conn_id) {
                            user_id.clone()
                        } else {
                            "".to_string()
                        };

                        (*conn_id, user_id)
                    })
                    .collect(),
            })
            .collect()
    }

    /// Join room, send join message to new room.
    async fn join_room(&mut self, conn_id: ConnId, room: RoomId) {
        // send message to other users
        self.rooms.entry(room.clone()).or_default().insert(conn_id);

        self.send_system_message(&room, conn_id, "Someone connected")
            .await;
    }

    async fn exit_room(&mut self, conn_id: ConnId, room: RoomId) {
        for (room_id, sessions) in &mut self.rooms {
            sessions.remove(&conn_id);
        }
    }

    async fn init(&mut self) {
        let pool = self.pool.clone();
        let rooms = web::block(move || {
            let mut conn = pool.get()?;
            db::rooms::get_all_rooms(&mut conn)
        })
        .await
        .unwrap()
        .unwrap();

        for room in rooms {
            let room_id = room.room.id;
            self.rooms.insert(room_id, HashSet::new());
        }
    }

    pub async fn run(mut self) -> io::Result<()> {
        self.init().await;

        while let Some(cmd) = self.cmd_rx.recv().await {
            match cmd {
                Command::Connect {
                    conn_tx,
                    res_tx,
                    user_id,
                } => {
                    let conn_id = self.connect(conn_tx, user_id).await;
                    res_tx.send(conn_id);
                }
                Command::Disconnect { conn } => {
                    self.disconnect(conn).await;
                }

                Command::List { res_tx } => {
                    res_tx.send(self.list_rooms());
                }

                Command::Join { conn, room, res_tx } => {
                    self.join_room(conn, room).await;
                    res_tx.send(());
                }

                Command::Exit { conn, room, res_tx } => {
                    self.exit_room(conn, room).await;
                    res_tx.send(());
                }

                Command::Message {
                    msg,
                    conn,
                    room_id,
                    res_tx,
                } => {
                    self.send_mesage(conn, room_id, msg).await;
                    res_tx.send(());
                }

                Command::Broadcast { msg, conn, res_tx } => {
                    self.broadcast(conn, msg).await;
                    res_tx.send(());
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ChatServerHandle {
    cmd_tx: mpsc::UnboundedSender<Command>,
}

impl ChatServerHandle {
    pub async fn connect(&self, conn_tx: mpsc::UnboundedSender<Msg>, user_id: UserId) -> ConnId {
        let (res_tx, res_rx) = oneshot::channel();

        self.cmd_tx
            .send(Command::Connect {
                conn_tx,
                res_tx,
                user_id,
            })
            .unwrap();

        res_rx.await.unwrap()
    }

    pub async fn disconnect(&self, conn: ConnId) {
        let (res_tx, res_rx) = oneshot::channel();

        self.cmd_tx.send(Command::Disconnect { conn }).unwrap();

        res_rx.await.unwrap()
    }

    pub async fn send_message(&self, msg: Msg, room_id: String, conn: ConnId) {
        let (res_tx, res_rx) = oneshot::channel();

        println!("send message: {msg}, {conn},{room_id}");
        self.cmd_tx
            .send(Command::Message {
                msg,
                conn,
                room_id,
                res_tx,
            })
            .unwrap();

        res_rx.await.unwrap()
    }

    pub async fn broadcast(&self, conn: ConnId, msg: Msg) {
        let (res_tx, res_rx) = oneshot::channel();

        self.cmd_tx
            .send(Command::Broadcast { msg, conn, res_tx })
            .unwrap();

        res_rx.await.unwrap()
    }

    pub async fn list_rooms(&self) -> Vec<WsRoom> {
        let (res_tx, res_rx) = oneshot::channel();

        self.cmd_tx.send(Command::List { res_tx }).unwrap();

        res_rx.await.unwrap()
    }

    pub async fn join_room(&self, conn: ConnId, room: impl Into<RoomId>) {
        let (res_tx, res_rx) = oneshot::channel();

        self.cmd_tx
            .send(Command::Join {
                conn,
                room: room.into(),
                res_tx,
            })
            .unwrap();

        res_rx.await.unwrap()
    }

    pub async fn exit_room(&self, conn: ConnId, room: RoomId) {
        let (res_tx, res_rx) = oneshot::channel();

        self.cmd_tx
            .send(Command::Exit { conn, room, res_tx })
            .unwrap();

        res_rx.await.unwrap()
    }
}
