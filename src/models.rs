use crate::schema::*;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    #[serde(skip_serializing)]
    pub created_at: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Queryable, Insertable)]
pub struct Conversation {
    pub id: String,
    pub room_id: String,
    pub user_id: String,
    pub content: String,
    pub created_at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub last_message: String,
    pub participant_ids: String,
    pub created_at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub sign_in: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewConversation {
    pub user_id: String,
    pub room_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomResponse {
    pub room: Room,
    pub users: Vec<User>,
}
