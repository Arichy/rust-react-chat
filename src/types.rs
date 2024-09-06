use actix_web::error::ErrorInternalServerError;
use diesel::{
    r2d2::{self, ConnectionManager},
    SqliteConnection,
};

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;
