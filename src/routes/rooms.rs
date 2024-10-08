use std::str::FromStr;

use crate::{
    db,
    server::ChatServerHandle,
    services,
    types::DbPool,
    utils::{get_conn_id, get_user_id},
};
use actix_session::Session;
use actix_web::{
    delete, error::ErrorInternalServerError, get, post, web, Error, HttpRequest, HttpResponse,
};
use futures_util::TryFutureExt;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[get("")]
pub async fn get_rooms(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let rooms = web::block(move || {
        let mut conn = pool.get()?;
        db::rooms::get_all_rooms(&mut conn)
    })
    .await?
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(rooms))
}

#[derive(Deserialize)]
struct CreateRoomData {
    room_name: String,
}

#[post("")]
pub async fn create_room(
    pool: web::Data<DbPool>,
    data: web::Json<CreateRoomData>,
    session: Session,
    chat_server: web::Data<ChatServerHandle>,
) -> Result<HttpResponse, Error> {
    println!("create room with name: {}", data.room_name);
    let user_id = get_user_id(&session);

    // get room and user info
    let (room_res, user_res) = tokio::join!(
        web::block({
            let pool = pool.clone();
            move || {
                let mut conn = pool.get()?;

                db::rooms::create_room(&mut conn, &user_id, &data.room_name)
            }
        }),
        web::block({
            let pool = pool.clone();
            move || {
                let mut conn = pool.get()?;

                db::users::find_user_by_uid(&mut conn, user_id)
            }
        })
    );

    let room = room_res?.map_err(ErrorInternalServerError)?;
    let user = user_res?.map_err(ErrorInternalServerError)?;

    let room_id = Uuid::from_str(&room.id).unwrap();

    // join the room
    services::rooms::join_room(pool.clone(), user_id, room_id)
        .await
        .map_err(ErrorInternalServerError)?;

    let room = web::block(move || {
        let mut conn = pool.get()?;
        db::rooms::get_room(&mut conn, room_id)
    })
    .await?
    .map_err(ErrorInternalServerError)?;

    chat_server
        .broadcast(
            0,
            json!({
                "type": "create_room",
                "data": room,
            })
            .to_string(),
        )
        .await;

    Ok(HttpResponse::Ok().json(json!({
        "room": room
    })))
}

#[post("/{room_id}/join")]
pub async fn join_room(
    request: HttpRequest,
    pool: web::Data<DbPool>,
    session: Session,
    room_id: web::Path<Uuid>,
    chat_server: web::Data<ChatServerHandle>,
) -> Result<HttpResponse, Error> {
    let room_id = room_id.to_owned();
    let user_id = get_user_id(&session);
    // let conn_id = get_conn_id(&request);

    let _ = services::rooms::join_room(pool.clone(), user_id, room_id)
        .await
        .map_err(ErrorInternalServerError);

    // if !conn_id.is_err() {
    let user = services::users::find_user_by_uid(pool, user_id)
        .await
        .map_err(ErrorInternalServerError)?
        .unwrap();

    chat_server
        .broadcast(
            // conn_id.unwrap(),
            0,
            json!({
                "type": "join_room",
                "data": {
                    "room_id": room_id.to_string(),
                    "user": user,
                }
            })
            .to_string(),
        )
        .await;
    // }

    Ok(HttpResponse::Ok().finish())
}

#[post("/{room_id}/exit")]
pub async fn exit_room(
    request: HttpRequest,
    pool: web::Data<DbPool>,
    session: Session,
    room_id: web::Path<Uuid>,
    chat_server: web::Data<ChatServerHandle>,
) -> Result<HttpResponse, Error> {
    let room_id = room_id.to_owned();
    let user_id = get_user_id(&session);
    // let conn_id = get_conn_id(&request);

    let _ = services::rooms::exit_room(pool, user_id, room_id)
        .await
        .map_err(ErrorInternalServerError);

    // if !conn_id.is_err() {
    chat_server
        .broadcast(
            // conn_id.unwrap(),
            0,
            json!({
                "type": "exit_room",
                "data": {
                    "room_id": room_id.to_string(),
                    "user_id": user_id.to_string(),
                }
            })
            .to_string(),
        )
        .await;
    // }
    // send ws messages

    Ok(HttpResponse::Ok().finish())
}

#[delete("/{room_id}")]
pub async fn delete_room(
    request: HttpRequest,
    pool: web::Data<DbPool>,
    session: Session,
    room_id: web::Path<Uuid>,
    chat_server: web::Data<ChatServerHandle>,
) -> Result<HttpResponse, Error> {
    let room_id = room_id.to_owned();
    let user_id = get_user_id(&session);

    let room = {
        let pool = pool.clone();
        let room_id = room_id.clone();

        web::block(move || {
            let mut conn = pool.get()?;

            db::rooms::get_room(&mut conn, room_id)
        })
        .await?
        .map_err(ErrorInternalServerError)?
    };

    if room.is_none() {
        return Ok(HttpResponse::NotFound().json(json!({
            "message": format!("Room {} is not found.", room_id)
        })));
    }

    let room = room.unwrap();

    if room.room.owner_id != user_id.to_string() {
        return Ok(HttpResponse::Unauthorized().json(json!({
            "message": "You're not the owner."
        })));
    }

    let res = web::block(move || {
        let mut conn = pool.get()?;

        db::rooms::delete_room(&mut conn, room_id)
    })
    .await?
    .map_err(ErrorInternalServerError)?;

    chat_server.broadcast(
        0,
        json!({
            "type": "delete_room",
            "data": {
                "room_id": room_id.to_string(),
            }
        })
        .to_string(),
    ).await;

    Ok(HttpResponse::Ok().finish())
}

#[get("/{room_id}")]
pub async fn get_room(
    pool: web::Data<DbPool>,
    room_id: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let room_id = room_id.to_owned();
    let room = {
        let pool = pool.clone();

        web::block(move || {
            let mut conn = pool.get()?;

            db::rooms::get_room(&mut conn, room_id)
        })
        .await?
        .map_err(ErrorInternalServerError)?
    };

    match room {
        Some(room) => Ok(HttpResponse::Ok().json(room)),
        None => Ok(HttpResponse::NotFound().json(json!({
            "message": format!("Room {} is not found.", room_id)
        }))),
    }
}
