use actix_session::Session;
use actix_web::{
    error::{Error, ErrorInternalServerError},
    post, web, HttpResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{db, server::ChatServerHandle, types::DbPool, utils::get_user_id};

#[derive(Debug, Serialize, Deserialize)]
struct CreateConversation {
    conn_id: String,
    message: String,
    room_id: String,
}

#[post("")]
pub async fn create_conversation(
    pool: web::Data<DbPool>,
    form_data: web::Json<CreateConversation>,
    session: Session,
    chat_server: web::Data<ChatServerHandle>,
) -> Result<HttpResponse, Error> {
    println!("enter create conversation");
    println!("{:?}", session.entries());
    let user_id = get_user_id(&session);

    let CreateConversation {
        conn_id,
        message,
        room_id,
    } = form_data.0;

    let conn_id: Result<usize, _> = conn_id.parse();
    if conn_id.is_err() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "message": "Invalid conn_id",
        })));
    }

    let conn_id = conn_id.unwrap();

    let res = {
        let message = message.clone();
        let room_id = room_id.clone();
        web::block(move || {
            let mut conn = pool.get()?;

            db::conversations::create_conversation(&mut conn, message, room_id, user_id.to_string())
        })
        .await?
        .map_err(ErrorInternalServerError)?
    };

    // send ws message
    chat_server
        .send_message(
            json!({
                "type": "message",
                "data": res,
            })
            .to_string(),
            room_id,
            conn_id,
        )
        .await;

    Ok(HttpResponse::Ok().json(res))
}
