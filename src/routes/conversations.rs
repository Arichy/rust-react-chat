use actix_session::Session;
use actix_web::{
    error::{Error, ErrorInternalServerError},
    post, web, HttpRequest, HttpResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    db,
    server::ChatServerHandle,
    types::DbPool,
    utils::{get_conn_id, get_user_id},
    ConnId,
};

#[derive(Debug, Serialize, Deserialize)]
struct CreateConversation {
    message: String,
    room_id: String,
}

#[post("")]
pub async fn create_conversation(
    request: HttpRequest,
    pool: web::Data<DbPool>,
    form_data: web::Json<CreateConversation>,
    session: Session,
    chat_server: web::Data<ChatServerHandle>,
) -> Result<HttpResponse, Error> {
    println!("enter create conversation");
    println!("{:?}", session.entries());
    let user_id = get_user_id(&session);
    let conn_id = get_conn_id(&request)?;

    let CreateConversation { message, room_id } = form_data.0;

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
