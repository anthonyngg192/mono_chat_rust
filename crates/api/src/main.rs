use std::{net::Ipv4Addr, str::FromStr};

use async_std::channel::unbounded;
use authifier::{Authifier, AuthifierEvent};
use chat_core::{events::client::EventV1, r#impl::MongoDb, Database, DatabaseInfo};
use rocket::data::ToByteUnit;
use rocket::{Build, Rocket};
use rocket_cors::{AllowedOrigins, CorsOptions};
use rocket_prometheus::PrometheusMetrics;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate revolt_rocket_okapi;

#[macro_use]
extern crate serde_json;

pub mod routes;
pub mod util;

pub async fn web() -> Rocket<Build> {
    let db = DatabaseInfo::Auto.connect().await.unwrap();
    db.migrate_database().await.unwrap();

    let legacy_db = DatabaseInfo::Auto.connect().await.unwrap();

    let (sender, receiver) = unbounded();

    let authifier = Authifier {
        database: match db.clone() {
            Database::MongoDb(MongoDb(client, _)) => authifier::Database::MongoDb(
                authifier::database::MongoDb(client.database("rust_demo")),
            ),
        },
        config: chat_core::util::authifier::config(),
        event_channel: Some(sender),
    };

    async_std::task::spawn(async move {
        while let Ok(event) = receiver.recv().await {
            match &event {
                AuthifierEvent::CreateSession { .. } | AuthifierEvent::CreateAccount { .. } => {
                    EventV1::Auth(event).global().await
                }
                AuthifierEvent::DeleteSession { user_id, .. }
                | AuthifierEvent::DeleteAllSessions { user_id, .. } => {
                    let id = user_id.to_string();
                    EventV1::Auth(event).private(id).await
                }
            }
        }
    });

    async_std::task::spawn(chat_core::tasks::start_workers(
        db.clone(),
        authifier.database.clone(),
    ));
    async_std::task::spawn(chat_core::tasks::start_workers(
        legacy_db.clone(),
        authifier.database.clone(),
    ));

    let cors = CorsOptions {
        allowed_origins: AllowedOrigins::All,
        allowed_methods: [
            "Get", "Put", "Post", "Delete", "Options", "Head", "Trace", "Connect", "Patch",
        ]
        .iter()
        .map(|s| FromStr::from_str(s).unwrap())
        .collect(),
        ..Default::default()
    }
    .to_cors()
    .expect("Failed to create CORS.");

    // Configure Swagger
    let swagger = revolt_rocket_okapi::swagger_ui::make_swagger_ui(
        &revolt_rocket_okapi::swagger_ui::SwaggerUIConfig {
            url: "../openapi.json".to_owned(),
            ..Default::default()
        },
    )
    .into();

    let rocket = rocket::build();
    let prometheus = PrometheusMetrics::new();

    routes::mount(rocket)
        .attach(prometheus.clone())
        .mount("/metrics", prometheus)
        .mount("/", rocket_cors::catch_all_options_routes())
        .mount("/", util::ratelimiter::routes())
        .mount("/swagger/", swagger)
        .manage(authifier)
        .manage(db)
        // .manage(legacy_db)
        .manage(cors.clone())
        .attach(util::ratelimiter::RatelimitFairing)
        .attach(cors)
        .configure(rocket::Config {
            limits: rocket::data::Limits::default().limit("string", 5.megabytes()),
            address: Ipv4Addr::new(0, 0, 0, 0).into(),
            ..Default::default()
        })
}

#[launch]
async fn rocket() -> _ {
    chat_core::configure!();
    chat_core::variables::delta::preflight_checks();
    web().await
}
