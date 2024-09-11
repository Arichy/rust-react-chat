use actix_web::{error::ErrorInternalServerError, web};
use uuid::Uuid;

use crate::{
    db::{self, DbError},
    models::User,
    types::DbPool,
};

pub async fn find_user_by_uid(
    pool: web::Data<DbPool>,
    user_id: Uuid,
) -> Result<Option<User>, DbError> {
    web::block(move || {
        let mut conn = pool.get()?;
        db::users::find_user_by_uid(&mut conn, user_id)
    })
    .await?
}
