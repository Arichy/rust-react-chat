use crate::schema::*;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

// db models
#[derive(
    Debug, Clone, PartialEq, Serialize, Deserialize, Queryable, Selectable, Identifiable, Insertable,
)]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    #[serde(skip_serializing)]
    pub created_at: String,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    Queryable,
    Identifiable,
    Associations,
    Insertable,
    Selectable,
)]
#[diesel(belongs_to(Room))]
#[diesel(belongs_to(User))]
pub struct Conversation {
    pub id: String,
    pub room_id: String,
    pub user_id: String,
    pub message: String,
    pub created_at: String,
}

#[derive(
    Debug, Clone, PartialEq, Serialize, Deserialize, Selectable, Queryable, Identifiable, Insertable,
)]
#[diesel(table_name = rooms)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub last_message: String,
    pub created_at: String,
    pub owner_id: String,
}

#[derive(Identifiable, Selectable, Insertable, Queryable, Associations, Debug, Clone)]
#[diesel(belongs_to(Room))]
#[diesel(belongs_to(User))]
#[diesel(table_name = rooms_users)]
#[diesel(primary_key(room_id, user_id))]
pub struct RoomUser {
    pub room_id: String,
    pub user_id: String,
}

// business models
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
pub struct ListRoomResponse {
    pub room: Room,
    pub users: Vec<User>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomResponse {
    pub room: Room,
    pub users: Vec<User>,
    pub conversations: Vec<Conversation>,
}
