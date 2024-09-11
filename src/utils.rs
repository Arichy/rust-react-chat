use actix_session::Session;
use actix_web::{error, HttpRequest, HttpResponse};
use serde_json::json;
use uuid::Uuid;

use crate::ConnId;

pub fn get_user_id(session: &Session) -> Uuid {
    session.get("user_id").unwrap().unwrap()
}

pub fn get_conn_id(request: &HttpRequest) -> Result<ConnId, error::Error> {
    if let Some(conn_id) = request.headers().get("Conn-Id") {
        if let Ok(v) = conn_id.to_str() {
            let conn_id = v.parse::<ConnId>();
            if conn_id.is_err() {
                return Err(error::ErrorBadRequest(json!({
                      "message": "Invalid Conn-Id.",
                })));
            }
            return Ok(conn_id.unwrap());
        } else {
            return Err(error::ErrorBadRequest(json!({
                  "message": "Invalid Conn-Id.",
            })));
        }
    } else {
        return Err(error::ErrorBadRequest(json!({
              "message": "Conn-Id not found in header.",
        })));
    }
}
