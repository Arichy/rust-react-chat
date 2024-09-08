use crate::db;
use crate::models;
use crate::server;
use crate::session;
use crate::types::DbPool;
use actix::*;
use actix_files::NamedFile;
use actix_session::Session;
use actix_web::error::ErrorInternalServerError;
use actix_web::{get, post, web, Error, HttpRequest, HttpResponse, Responder, Scope};
use actix_web_actors::ws;
use bcrypt::verify;
use diesel::result::DatabaseErrorKind;
use diesel::{
    prelude::*,
    r2d2::{self, ConnectionManager},
};
use serde::Deserialize;
use serde_json::json;

use std::time::Instant;
use uuid::Uuid;

pub async fn index() -> impl Responder {
    NamedFile::open_async("./static/index.html").await.unwrap()
}

pub async fn chat_server(
    req: HttpRequest,
    stream: web::Payload,
    pool: web::Data<DbPool>,
    srv: web::Data<Addr<server::ChatServer>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        session::WsChatSession {
            id: 0,
            hb: Instant::now(),
            room: "main".to_string(),
            name: None,
            addr: srv.get_ref().clone(),
            db_pool: pool,
        },
        &req,
        stream,
    )
}

#[get("/conversations/{uid}")]
pub async fn get_conversation_by_id(
    pool: web::Data<DbPool>,
    uid: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let room_id = uid.to_owned();
    let conversations = web::block(move || {
        let mut conn = pool.get()?;
        db::get_conversation_by_room_uid(&mut conn, room_id)
    })
    .await?
    .map_err(ErrorInternalServerError)?;
    if let Some(data) = conversations {
        Ok(HttpResponse::Ok().json(data))
    } else {
        let res = HttpResponse::NotFound().body(
            json!({
                "error": 404,
                "message": format!("No conversation with room_id: {room_id}")
            })
            .to_string(),
        );
        Ok(res)
    }
}

pub mod auth;
pub mod conversations;
pub mod rooms;
pub mod users;

pub fn create_auth_scope() -> Scope {
    web::scope("/auth")
        .service(auth::sign_up)
        .service(auth::sign_in)
        .service(auth::get_current_user)
        .service(auth::log_out)
}

pub fn create_room_scope() -> Scope {
    web::scope("/rooms")
        .service(rooms::get_rooms)
        .service(rooms::create_room)
        .service(rooms::join_room)
        .service(rooms::exit_room)
        .service(rooms::get_room)
}
