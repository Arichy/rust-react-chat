use crate::session;
use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use serde_json::json;
use std::collections::{HashMap, HashSet};

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
}
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub id: usize,
    pub msg: String,
    pub room: String,
}
pub struct ListRooms;

impl actix::Message for ListRooms {
    type Result = Vec<String>;
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct Join {
    pub id: usize,
    pub name: String,
}

#[derive(Debug)]
pub struct ChatServer {
    sessions: HashMap<usize, Recipient<Message>>,
    rooms: HashMap<String, HashSet<usize>>, // room name => session set
    rng: ThreadRng,
}
impl ChatServer {
    pub fn new() -> Self {
        let mut rooms = HashMap::new();
        rooms.insert("main".to_string(), HashSet::new());
        Self {
            sessions: HashMap::new(),
            rooms,
            rng: rand::thread_rng(),
        }
    }

    fn send_message(&self, room: &str, message: &str, skip_id: usize) {
        if let Some(sessions) = self.rooms.get(room) {
            for id in sessions {
                if *id != skip_id {
                    if let Some(addr) = self.sessions.get(id) {
                        addr.do_send(Message(message.to_string()));
                    }
                }
            }
        }
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for ChatServer {
    type Result = usize;
    fn handle(&mut self, msg: Connect, _: &mut Self::Context) -> Self::Result {
        // generate a random session id
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);
        // insert the session id to main room
        self.rooms
            .entry("main".to_string())
            .or_insert_with(HashSet::new)
            .insert(id);

        // send to main room a message
        self.send_message(
            "main",
            &json!({
                "value": vec![format!("{}",id)],
                "chat_type": session::ChatType::CONNECT,
            })
            .to_string(),
            0,
        );
        id
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();
    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) -> Self::Result {
        // rooms which contain the session id
        let mut rooms: Vec<String> = vec![];
        if self.sessions.remove(&msg.id).is_some() {
            for (name, sessions) in &mut self.rooms {
                if sessions.remove(&msg.id) {
                    rooms.push(name.to_owned());
                }
            }
        }
        for room in rooms {
            self.send_message(
                "main",
                &json!({
                    "room": room,
                    "value": vec![format!("Someone disconnect!")],
                    "chat_type": session::ChatType::DISCONNECT,
                })
                .to_string(),
                0,
            )
        }
    }
}

impl Handler<ClientMessage> for ChatServer {
    type Result = ();
    fn handle(&mut self, msg: ClientMessage, _: &mut Self::Context) -> Self::Result {
        self.send_message(&msg.room, &msg.msg, msg.id);
    }
}

impl Handler<Join> for ChatServer {
    type Result = ();
    fn handle(&mut self, msg: Join, _: &mut Self::Context) -> Self::Result {
        println!("Join message: {:?}", msg);
        // exit other rooms
        let Join { name, id } = msg;
        let mut rooms = vec![];
        for (n, sessions) in &mut self.rooms {
            if sessions.remove(&id) {
                rooms.push(n.to_owned());
            }
        }
        for room in rooms {
            self.send_message(
                &room,
                &json!({
                   "room": room,
                   "value":vec!["Someone disconnect!".to_string()],
                   "chat_type": session::ChatType::DISCONNECT,
                })
                .to_string(),
                0,
            )
        }

        // enter the target room
        self.rooms.entry(name).or_insert(HashSet::new()).insert(id);
    }
}
