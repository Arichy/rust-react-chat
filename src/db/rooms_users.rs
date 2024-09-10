use diesel::prelude::*;
use serde_json::json;
use uuid::Uuid;

use crate::models::{ListRoomResponse, RoomUser};

use super::DbError;

pub fn join_room(conn: &mut SqliteConnection, user_id: Uuid, room_id: Uuid) -> Result<(), DbError> {
    use crate::schema::rooms_users;

    diesel::insert_into(rooms_users::table)
        .values((
            rooms_users::room_id.eq(room_id.to_string()),
            rooms_users::user_id.eq(user_id.to_string()),
        ))
        .execute(conn)?;

    Ok(())
}

pub fn exit_room(conn: &mut SqliteConnection, user_id: Uuid, room_id: Uuid) -> Result<(), DbError> {
    use crate::schema::rooms_users;

    diesel::delete(
        rooms_users::table.filter(
            rooms_users::room_id
                .eq(room_id.to_string())
                .and(rooms_users::user_id.eq(user_id.to_string())),
        ),
    )
    .execute(conn)?;

    Ok(())
}
