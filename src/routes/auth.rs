use crate::{
    db,
    models::{self, User},
    types::DbPool,
};
use actix_session::Session;
use actix_web::{body::BoxBody, get, post, web, Error, HttpRequest, HttpResponse};
use bcrypt::verify;
use diesel::result::DatabaseErrorKind;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize)]
struct SignData {
    username: String,
    password: String,
}

#[post["/signup"]]
pub async fn sign_up(
    pool: web::Data<DbPool>,
    form: web::Json<models::NewUser>,
    session: Session,
) -> Result<HttpResponse, Error> {
    let signin: bool = form.sign_in;
    let username = form.username.clone();
    let user = web::block(move || {
        let mut conn = pool.get()?;
        db::users::insert_new_user(&mut conn, &form.username, &form.password)
    })
    .await?
    .map_err(|err| {
        let error_msg = if let Some(diesel_error) = err.downcast_ref::<diesel::result::Error>() {
            match diesel_error {
                diesel::result::Error::DatabaseError(kind, _) => match kind {
                    DatabaseErrorKind::UniqueViolation => {
                        format!("Username {} already exists.", username)
                    }
                    _ => diesel_error.to_string(),
                },
                _ => diesel_error.to_string(),
            }
        } else {
            err.to_string()
        };

        actix_web::error::ErrorUnprocessableEntity(json!({
            "success": false,
            "message": error_msg
        }))
    })?;

    if signin {
        session.insert("user_id", user.id).unwrap();
    }

    Ok(HttpResponse::Ok().json(json!({
        "success": true,
    })))
}

#[post("/signin")]
pub async fn sign_in(
    pool: web::Data<DbPool>,
    session: Session,
    signin_data: web::Json<SignData>,
) -> Result<HttpResponse, Error> {
    let SignData { username, password } = signin_data.0;

    let username_clone = username.clone();

    let user = web::block(move || {
        let mut conn = pool.get()?;
        db::users::find_user_by_username(&mut conn, username_clone)
    })
    .await?
    .map_err(actix_web::error::ErrorInternalServerError)?;

    if let Some(user) = user {
        if verify(password, &user.password).unwrap() {
            session.insert("user_id", user.id.clone()).unwrap();
            Ok(HttpResponse::Ok().json(user))
        } else {
            let res = HttpResponse::Unauthorized().body(
                json!({
                    "message": format!("Wrong password for username: {}", &username)
                })
                .to_string(),
            );
            Ok(res)
        }
    } else {
        let res = HttpResponse::NotFound().body(
            json!({
                "message": format!("No user found with username: {}", &username)
            })
            .to_string(),
        );
        Ok(res)
    }
}

#[post("/logout")]
pub async fn log_out(session: Session) -> HttpResponse {
    let user_id: Option<Uuid> = session.get("user_id").unwrap_or(None);

    match user_id {
        Some(_) => {
            session.purge();
            HttpResponse::Ok().json(json!({}))
        }
        None => HttpResponse::Unauthorized().json(json!({
            "message": "You're not signed in."
        })),
    }
}

#[get("/user")]
pub async fn get_current_user(
    pool: web::Data<DbPool>,
    session: Session,
) -> Result<HttpResponse, Error> {
    let user_id = session.get::<Uuid>("user_id").unwrap().unwrap();

    let user = web::block(move || {
        let mut conn = pool.get()?;
        db::users::find_user_by_uid(&mut conn, user_id)
    })
    .await?
    .map_err(actix_web::error::ErrorInternalServerError)?;

    if user.is_none() {
        Ok(HttpResponse::NotFound().json(json!({
            "message": format!("User {} does not exist.", user_id),
        })))
    } else {
        Ok(HttpResponse::Ok().json(user))
    }
}
