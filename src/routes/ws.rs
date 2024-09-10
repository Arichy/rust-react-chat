use std::{
    pin::pin,
    time::{Duration, Instant},
};

use actix_session::Session;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_ws::{AggregatedMessage, Message};
use futures_util::{
    future::{select, Either},
    StreamExt as _,
};
use serde_json::json;
use tokio::{sync::mpsc, task::spawn_local, time::interval};

use crate::{server::ChatServerHandle, utils::get_user_id, ConnId};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

async fn chat_ws_handler(
    chat_server: ChatServerHandle,
    mut session: actix_ws::Session,
    msg_stream: actix_ws::MessageStream,
    user_id: String,
) {
    log::info!("connected");
    let mut name = None;
    let mut last_heartbeat = Instant::now();
    let mut interval = interval(HEARTBEAT_INTERVAL);

    let (conn_tx, mut conn_rx) = mpsc::unbounded_channel();

    let conn_id = chat_server.connect(conn_tx, user_id).await;

    session
        .text(
            json!({
                "type":"init",
                "data": {
                    "conn_id": conn_id.to_string(),
                }
            })
            .to_string(),
        )
        .await
        .unwrap();

    let msg_stream = msg_stream
        .max_frame_size(128 * 1024)
        .aggregate_continuations()
        .max_continuation_size(2 * 1024 * 1024);

    let mut msg_stream = pin!(msg_stream);

    let close_reason = loop {
        let tick = pin!(interval.tick());
        let msg_rx = pin!(conn_rx.recv());

        let messages = pin!(select(msg_stream.next(), msg_rx));

        match select(messages, tick).await {
            Either::Left((Either::Left((Some(Ok(msg)), _)), _)) => {
                log::debug!("msg: {msg:?}");

                match msg {
                    AggregatedMessage::Ping(bytes) => {
                        last_heartbeat = Instant::now();
                        session.pong(&bytes).await.unwrap();
                    }

                    AggregatedMessage::Pong(_) => {
                        last_heartbeat = Instant::now();
                    }

                    AggregatedMessage::Text(text) => {
                        process_text_msg(&chat_server, &mut session, &text, conn_id, &mut name)
                            .await;
                    }

                    AggregatedMessage::Binary(_bin) => {
                        log::warn!("unexpected binary message");
                    }

                    AggregatedMessage::Close(reason) => break reason,
                }
            }

            // client WebSocket stream error
            Either::Left((Either::Left((Some(Err(err)), _)), _)) => {
                log::error!("{}", err);
                break None;
            }

            // client WebSocket stream ended
            Either::Left((Either::Left((None, _)), _)) => break None,

            // chat messages received from other room participants
            Either::Left((Either::Right((Some(chat_msg), _)), _)) => {
                println!("from others:{}", chat_msg);
                session.text(chat_msg).await.unwrap();
            }

            // all connection's message senders were dropped
            Either::Left((Either::Right((None, _)), _)) => unreachable!(
                "all connection message senders were dropped; chat server may have panicked"
            ),

            // heartbeat internal tick
            Either::Right((_inst, _)) => {
                // if no heartbeat ping/pong received recently, close the connection
                if Instant::now().duration_since(last_heartbeat) > CLIENT_TIMEOUT {
                    log::info!(
                        "client has not sent heartbeat in over {CLIENT_TIMEOUT:?}; disconnecting"
                    );
                    break None;
                }

                // send heartbeat ping
                let _ = session.ping(b"").await;
            }
        };
    };

    println!("disconnect:: {conn_id}");

    chat_server.disconnect(conn_id).await;

    let _ = session.close(close_reason).await;
}

pub async fn chat_ws(
    req: HttpRequest,
    stream: web::Payload,
    http_session: actix_session::Session,
    chat_server: web::Data<ChatServerHandle>,
) -> Result<HttpResponse, Error> {
    println!("here!");
    let user_id = get_user_id(&http_session).to_string();

    let (res, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;

    spawn_local(chat_ws_handler(
        (**chat_server).clone(),
        session,
        msg_stream,
        user_id,
    ));

    // actix_web::rt::spawn(async move {
    //     while let Some(Ok(msg)) = msg_stream.next().await {
    //         match msg {
    //             Message::Ping(bytes) => {
    //                 println!("ping {:?}", bytes);
    //                 if session.pong(&bytes).await.is_err() {
    //                     return;
    //                 }
    //             }

    //             Message::Text(msg) => println!("Got text: {msg}"),
    //             _ => break,
    //         }
    //     }

    //     let _ = session.close(None).await;
    // });

    Ok(res)
}

async fn process_text_msg(
    chat_server: &ChatServerHandle,
    session: &mut actix_ws::Session,
    text: &str,
    conn: ConnId,
    name: &mut Option<String>,
) {
    let msg = text.trim();

    println!("msg in process_text_msg:{}", msg);

    if msg.starts_with("/") {
        let mut cmd_args = msg.splitn(2, ' ');

        match cmd_args.next().unwrap() {
            "/list" => {
                log::info!("conn {conn}: listing rooms");

                let rooms = chat_server.list_rooms().await;

                session.text(json!(rooms).to_string()).await.unwrap();
            }

            "/join" => match cmd_args.next() {
                Some(room) => {
                    log::info!("conn {conn} joining room {room}");

                    chat_server.join_room(conn, room).await;

                    session.text(format!("joined {room}")).await.unwrap();
                }
                None => {
                    session.text("!!! room name is required").await.unwrap();
                }
            },
            _ => {
                session
                    .text(format!("!!! unknown command: {msg}"))
                    .await
                    .unwrap();
            }
        }
    }
}
