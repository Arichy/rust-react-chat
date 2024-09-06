#![allow(unused)]
use actix::*;
use actix_cors::Cors;
use actix_files::Files;
use actix_session::{
    config::PersistentSession, storage::CookieSessionStore, Session, SessionMiddleware,
};
use actix_web::{cookie::Key, dev::Service as _, get, http, middleware, web, App, HttpServer};
use diesel::{
    prelude::*,
    r2d2::{self, ConnectionManager},
};
use env_logger::Env;
use routes::{create_auth_scope, create_room_scope};

mod db;
mod models;
mod routes;
mod schema;
mod server;
mod session;

#[get("/hello")]
async fn hello(session: Session) -> String {
    println!("{:?}", session.entries());

    "world".to_string()
}

mod types;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server = server::ChatServer::new().start();
    let conn_spec = "chat.db";
    let manager = ConnectionManager::<SqliteConnection>::new(conn_spec);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");
    let server_addr = "127.0.0.1";
    let server_port = 8080;

    env_logger::init_from_env(Env::default().default_filter_or("info"));

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

        let api_scope = web::scope("/api")
            .service(hello)
            .service(auth_scope)
            .service(room_scope);

        App::new()
            .app_data(web::Data::new(server.clone()))
            .app_data(web::Data::new(pool.clone()))
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
            // .wrap_fn(|req, srv| {
            //     println!("origin: {:?}", req.headers().get("Origin").unwrap());
            //     println!("host: {:?}", req.headers().get("Host").unwrap());
            //     srv.call(req)
            // })
            .wrap(middleware::Logger::default())
            .service(web::resource("/").to(routes::index))
            .route("/ws", web::get().to(routes::chat_server))
            .service(api_scope)
            .service(Files::new("/", "./static"))
    })
    .workers(2)
    .bind((server_addr, server_port))?
    .run();

    println!("Server running at http://{server_addr}:{server_port}/");
    app.await
}
