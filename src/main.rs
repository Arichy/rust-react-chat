#![allow(unused)]
use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_session::{
    config::PersistentSession, storage::CookieSessionStore, Session, SessionExt, SessionMiddleware,
};
use actix_web::{
    cookie::Key,
    dev::{Service, ServiceRequest, ServiceResponse},
    get, http, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer,
};
use actix_web_lab::web::spa;
use diesel::{
    prelude::*,
    r2d2::{self, ConnectionManager},
};
use env_logger::Env;
use middlewares::auth::Authentication;
use models::Conversation;
use routes::{create_auth_scope, create_conversation_scope, create_room_scope};
use server::ChatServer;
use tokio::{task::spawn, try_join};
use uuid::Uuid;

mod db;
mod routes;
mod services;

mod middlewares;
mod models;
mod schema;
mod server;
// mod session;

mod types;
mod utils;

pub type ConnId = usize;
pub type RoomId = String;
pub type Msg = String;
pub type UserId = String;

#[get("/hello")]
async fn hello(session: Session) -> String {
    println!("{:?}", session.entries());

    "world".to_string()
}

#[get("/auth")]
async fn auth(session: Session) -> String {
    println!("{:?}", session.entries());

    "world".to_string()
}

// #[actix_web::main]
#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    let conn_spec = "chat.db";
    let manager = ConnectionManager::<SqliteConnection>::new(conn_spec);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");
    let server_addr = "127.0.0.1";
    let server_port = 8080;

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let (chat_server, server_tx) = ChatServer::new(pool.clone());

    let chat_server = spawn(chat_server.run());

    let app = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://localhost:5173")
            .allowed_origin("http://localhost:8080")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        let auth_scope = create_auth_scope();
        let room_scope = create_room_scope();
        let conversation_scope = create_conversation_scope();

        let api_scope = web::scope("/api")
            .service(hello)
            .service(auth_scope)
            .service(room_scope)
            .service(conversation_scope);

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(server_tx.clone()))
            .wrap(Authentication)
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    .cookie_secure(false)
                    .session_lifecycle(
                        PersistentSession::default()
                            .session_ttl(actix_web::cookie::time::Duration::hours(12)),
                    )
                    .build(),
            )
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .service(web::resource("/ws").route(web::get().to(routes::ws::chat_ws)))
            .service(api_scope)
            .service(spa()
                .index_file("./static/index.html")
                .static_resources_mount("/")
                .static_resources_location("./static")
                .finish())
            .wrap(middleware::NormalizePath::trim())
    })
    .workers(2)
    .bind((server_addr, server_port))?
    .run();

    log::info!("Server running at http://{server_addr}:{server_port}");

    try_join!(app, async move { chat_server.await.unwrap() })?;

    Ok(())
}
