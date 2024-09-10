use diesel::{RunQueryDsl, SqliteConnection};
use uuid::Uuid;

use crate::{db::iso_date, models::Conversation};

use super::DbError;

pub fn create_conversation(
    conn: &mut SqliteConnection,
    message: String,
    room_id: String,
    user_id: String,
) -> Result<Conversation, DbError> {
    use crate::schema::conversations;

    let new_conversation = Conversation {
        id: Uuid::new_v4().to_string(),
        room_id,
        user_id,
        message,
        created_at: iso_date(),
    };

    diesel::insert_into(conversations::table)
        .values(&new_conversation)
        .execute(conn)?;

    Ok(new_conversation)
}
