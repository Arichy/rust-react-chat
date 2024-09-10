use crate::models::*;

use crate::{
    models::{Conversation, ListRoomResponse, NewConversation, Room, RoomResponse, RoomUser, User},
    schema::{conversations, rooms, rooms_users, users},
};
use actix_web::web;
use bcrypt::{hash, DEFAULT_COST};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    time::SystemTime,
};

use uuid::Uuid;
// use crate::schema::rooms_users::dsl::rooms_users;
use super::{iso_date, DbError};

pub fn get_room(
    conn: &mut SqliteConnection,
    room_id: &str,
) -> Result<Option<RoomResponse>, DbError> {
    let room = rooms::table
        .filter(rooms::id.eq(room_id))
        .first::<Room>(conn);

    if room.is_err() {
        let err = room.unwrap_err();
        match err {
            diesel::result::Error::NotFound => {
                return Ok(None);
            }
            _ => return Err(err.into()),
        }
    }

    let room = room.unwrap();

    let users = RoomUser::belonging_to(&room)
        .inner_join(users::table)
        .select(User::as_select())
        .load(conn)?;

    let conversations = Conversation::belonging_to(&room)
        .select(Conversation::as_select())
        .load(conn)?;

    Ok(Some(RoomResponse {
        room,
        users,
        conversations,
    }))
}

pub fn get_all_rooms(conn: &mut SqliteConnection) -> Result<Vec<ListRoomResponse>, DbError> {
    let all_rooms = rooms::table.select(Room::as_select()).load(conn)?;

    let users: Vec<(RoomUser, User)> = RoomUser::belonging_to(&all_rooms)
        .inner_join(users::table)
        .select((RoomUser::as_select(), User::as_select()))
        .load(conn)?;

    let users_per_room = users
        .grouped_by(&all_rooms)
        .into_iter()
        .zip(all_rooms)
        .map(|(users, room)| {
            return ListRoomResponse {
                room,
                users: users.into_iter().map(|(_, user)| user).collect(),
            };
        })
        .collect();

    Ok(users_per_room)
}

pub fn create_room(
    conn: &mut SqliteConnection,
    creator_id: &Uuid,
    room_name: &str,
) -> Result<Room, DbError> {
    use crate::schema::rooms::dsl::*;

    let new_room = Room {
        id: Uuid::new_v4().to_string(),
        name: room_name.to_string(),
        last_message: "".to_string(),
        owner_id: creator_id.to_string(),
        created_at: iso_date(),
    };

    diesel::insert_into(rooms).values(&new_room).execute(conn)?;

    Ok(new_room)
}

pub fn delete_room(conn: &mut SqliteConnection, room_id: &str) -> Result<(), DbError> {
    use crate::schema::conversations;
    use crate::schema::rooms;
    use crate::schema::rooms_users;

    conn.transaction(|connection| {
        // delete room
        diesel::delete(rooms::table.filter(rooms::id.eq(room_id))).execute(connection)?;

        // delete conversations in the room
        diesel::delete(conversations::table.filter(conversations::room_id.eq(room_id)))
            .execute(connection)?;

        // delete rooms_users in the room
        diesel::delete(rooms_users::table.filter(rooms_users::room_id.eq(room_id)))
            .execute(connection)?;

        diesel::result::QueryResult::Ok(())
    });

    Ok(())
}

pub fn get_user_joined_rooms(conn: &mut SqliteConnection, user_id: String) -> Result<Vec<Room>, DbError> {
    // use crate::schema::{rooms, rooms_users};

    let rooms = rooms_users::table.filter(rooms_users::user_id.eq(user_id)).inner_join(rooms::table).select(Room::as_select()).load(conn)?;

    Ok(rooms)
}