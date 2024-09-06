use crate::schema::rooms;
use crate::{
    models::{Conversation, NewConversation, Room, RoomResponse, User},
    schema::{conversations::user_id, users},
};
use bcrypt::{hash, DEFAULT_COST};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    time::SystemTime,
};
use uuid::Uuid;

use super::{iso_date, DbError};

pub fn get_all_rooms(conn: &mut SqliteConnection) -> Result<Vec<RoomResponse>, DbError> {
    let rooms_data: Vec<Room> = rooms::table.get_results(conn)?;
    let mut ids = HashSet::new(); // user ids
    let mut rooms_map = HashMap::new();
    let data = rooms_data.to_vec();
    for room in &data {
        let user_ids = room
            .participant_ids
            .split(",")
            .into_iter()
            .collect::<Vec<_>>();
        for id in user_ids.to_vec() {
            ids.insert(id.to_string());
        }
        rooms_map.insert(room.id.to_string(), user_ids.to_vec());
    }

    let ids = ids.into_iter().collect::<Vec<_>>();

    let users_data: Vec<User> = users::table
        .filter(users::id.eq_any(ids))
        .get_results(conn)?;

    let users_map: HashMap<String, User> = HashMap::from_iter(
        users_data
            .into_iter()
            .map(|item| (item.id.to_string(), item)),
    );

    let response_rooms = rooms_data
        .into_iter()
        .map(|room| {
            let users = rooms_map
                .get(&room.id.to_string())
                .unwrap()
                .into_iter()
                .map(|id| users_map.get(id.to_owned()).unwrap().clone())
                .collect::<Vec<_>>();

            RoomResponse { room, users }
        })
        .collect::<Vec<_>>();

    Ok(response_rooms)
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
        participant_ids: creator_id.to_string(),
        created_at: iso_date(),
    };

    diesel::insert_into(rooms).values(&new_room).execute(conn)?;

    Ok(new_room)
}
