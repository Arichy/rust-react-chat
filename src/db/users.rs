use crate::{db::iso_date, models::User};
use bcrypt::hash;
use diesel::prelude::*;
use uuid::Uuid;

use super::DbError;

pub fn find_user_by_uid(conn: &mut SqliteConnection, uid: Uuid) -> Result<Option<User>, DbError> {
    use crate::schema::users::dsl::*;

    let user = users
        .filter(id.eq(uid.to_string()))
        .first::<User>(conn)
        .optional()?;
    Ok(user)
}

pub fn find_user_by_username(
    conn: &mut SqliteConnection,
    un: String,
) -> Result<Option<User>, DbError> {
    use crate::schema::users::dsl::*;
    let user = users
        .filter(username.eq(un))
        .first::<User>(conn)
        .optional()?;

    Ok(user)
}

pub fn insert_new_user(conn: &mut SqliteConnection, un: &str, pw: &str) -> Result<User, DbError> {
    use crate::schema::users::dsl::*;

    let hashed_password = hash(pw, 4).unwrap();

    let new_user = User {
        id: Uuid::new_v4().to_string(),
        username: un.to_owned(),
        password: hashed_password,
        created_at: iso_date(),
    };
    diesel::insert_into(users).values(&new_user).execute(conn)?;

    Ok(new_user)
}
