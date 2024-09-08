#![allow(unused)]
use actix::*;
use actix_cors::Cors;
use actix_files::Files;
use actix_session::{
    config::PersistentSession, storage::CookieSessionStore, Session, SessionExt, SessionMiddleware,
};
use actix_web::{
    cookie::Key,
    dev::{Service, ServiceRequest, ServiceResponse},
    get, http, middleware, web, App, Error, HttpResponse, HttpServer,
};
use diesel::{
    prelude::*,
    r2d2::{self, ConnectionManager},
};
use env_logger::Env;
use middlewares::auth::Authentication;
use routes::{create_auth_scope, create_room_scope};
use uuid::Uuid;

mod db;
mod routes;
mod services;

mod middlewares;
mod models;
mod schema;
mod server;
mod session;

mod types;
mod utils;

#[get("/hello")]
async fn hello(session: Session) -> String {
    println!("{:?}", session.entries());

    "world".to_string()
}

// async fn auth_middleware(
//     req: ServiceRequest,
//     srv: &web::Data<
//         dyn Service<ServiceRequest = ServiceRequest, Response = ServiceResponse, Error = Error>,
//     >,
// ) -> Result<ServiceResponse, Error> {
//     // Get session from request
//     let session = req.get_session();

//     // Get the path of the request
//     let path = req.path();

//     // Allow requests to "/api/auth" without authentication
//     if path.starts_with("/api/auth") {
//         return srv.call(req).await;
//     }

//     // Check if session has "user_id"
//     let user_id: Option<String> = session.get("user_id").unwrap_or(None);

//     match user_id {
//         Some(_uid) => {
//             // If user_id exists, continue to the next service
//             srv.call(req).await
//         }
//         None => {
//             // If user_id doesn't exist, return an error response
//             Ok(req.into_response(
//                 HttpResponse::Unauthorized()
//                     .json("User not authenticated")
//                     .into_body(),
//             ))
//         }
//     }
// }

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
