#[macro_use]
extern crate lazy_static;

use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use log::info;
use util::variables::JANUARY_PUBLIC_URL;

pub mod routes;
pub mod structs;
pub mod util;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    env_logger::init_from_env(env_logger::Env::default().filter_or("RUST_LOG", "info"));

    info!("Starting January server.");
    let addr = JANUARY_PUBLIC_URL.to_string();
    info!("Server working on {}.", addr);

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .route("/", web::get().to(routes::info::get))
            .route("/embed", web::get().to(routes::embed::get))
            .route("/proxy", web::get().to(routes::proxy::get))
    })
    .bind(JANUARY_PUBLIC_URL.clone())?
    .run()
    .await
}
