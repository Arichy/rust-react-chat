use crate::{db, types::DbPool};
use actix_session::Session;
use actix_web::{error::ErrorInternalServerError, get, post, web, Error, HttpResponse};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[get("")]
pub async fn get_rooms(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    println!("get room");
    let rooms = web::block(move || {
        let mut conn = pool.get()?;
        db::rooms::get_all_rooms(&mut conn)
    })
    .await?
    .map_err(actix_web::error::ErrorInternalServerError)?;
    if !rooms.is_empty() {
        Ok(HttpResponse::Ok().json(rooms))
    } else {
        let res = HttpResponse::NotFound().body(
            json!({
                "error": 404,
                "message": "No rooms available at the moment.",
            })
            .to_string(),
        );
        Ok(res)
    }
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
) -> Result<HttpResponse, Error> {
    println!("create room with name: {}", data.room_name);
    let user_id: Option<Uuid> = session.get("user_id").unwrap();

    match user_id {
        None => Ok(HttpResponse::Unauthorized().json(json!({
            "message": "Signin required."
        }))),
        Some(user_id) => {
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

            Ok(HttpResponse::Ok().json(json!({
                "room": room
            })))
        }
    }
}

// #[get("/{room_id}")]
// pub async fn get_room(pool: web::Data<DbPool>)-> Result<HttpResponse,Error> {

// }
