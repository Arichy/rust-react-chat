use actix_web::{error::ErrorInternalServerError, get, web, Error, HttpResponse};
use serde_json::json;
use uuid::Uuid;

use crate::{db, types::DbPool};

#[get("/users/{user_id}")]
pub async fn get_user_by_id(
    pool: web::Data<DbPool>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, Error> {
    let user_id = id.to_owned();
    let user = web::block(move || {
        let mut conn = pool.get()?;

        db::users::find_user_by_uid(&mut conn, user_id)
    })
    .await?
    .map_err(ErrorInternalServerError)?;

    if let Some(user) = user {
        Ok(HttpResponse::Ok().json(user))
    } else {
        let res = HttpResponse::NotFound().body(
            json!({
                "error": 404,
                "message": format!("No user found with phone: {id}")
            })
            .to_string(),
        );
        Ok(res)
    }
}
