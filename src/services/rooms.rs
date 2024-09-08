use actix_web::{error::ErrorInternalServerError, web};
use uuid::Uuid;

use crate::{
    db::{self, DbError},
    types::DbPool,
};

pub async fn join_room(
    pool: web::Data<DbPool>,
    user_id: Uuid,
    room_id: Uuid,
) -> Result<(), DbError> {
    web::block(move || {
        let mut conn = pool.get()?;
        db::rooms_users::join_room(&mut conn, user_id, room_id)
    })
    .await?
}

pub async fn exit_room(
    pool: web::Data<DbPool>,
    user_id: Uuid,
    room_id: Uuid,
) -> Result<(), DbError> {
    web::block(move || {
        let mut conn = pool.get()?;
        db::rooms_users::exit_room(&mut conn, user_id, room_id)
    })
    .await?
}
