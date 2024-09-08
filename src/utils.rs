use actix_session::Session;
use uuid::Uuid;

pub fn get_user_id(session: &Session) -> Uuid {
    session.get("user_id").unwrap().unwrap()
}
