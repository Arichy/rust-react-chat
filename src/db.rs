use crate::models::{Conversation, NewConversation, Room, RoomResponse, User};
use bcrypt::{hash, DEFAULT_COST};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    time::SystemTime,
};
use uuid::Uuid;
type DbError = Box<dyn std::error::Error + Send + Sync>;

fn iso_date() -> String {
    let now = SystemTime::now();
    let now: DateTime<Utc> = now.into();
    now.to_rfc3339()
}

pub fn get_conversation_by_room_uid(
    conn: &mut SqliteConnection,
    uid: Uuid,
) -> Result<Option<Vec<Conversation>>, DbError> {
    use crate::schema::conversations;

    let convo = conversations::table
        .filter(conversations::room_id.eq(uid.to_string()))
        .load(conn)
        .optional()?;

    Ok(convo)
}

pub fn insert_new_conversation(
    conn: &mut SqliteConnection,
    new: NewConversation,
) -> Result<Conversation, DbError> {
    use crate::schema::conversations::dsl::*;
    let new_conversation = Conversation {
        id: Uuid::new_v4().to_string(),
        user_id: new.user_id,
        room_id: new.room_id,
        content: new.message,
        created_at: iso_date(),
    };
    diesel::insert_into(conversations)
        .values(&new_conversation)
        .execute(conn)?;

    Ok(new_conversation)
}

pub mod conversations;
pub mod rooms;
pub mod users;
