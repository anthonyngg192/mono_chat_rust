use chat_core::{Database, DatabaseInfo};
use once_cell::sync::OnceCell;
use std::env;

static DB_CONN: OnceCell<Database> = OnceCell::new();

pub async fn connect() {
    match env::var("MONGODB") {
        Ok(uri) => {
            let database = DatabaseInfo::MongoDb(uri)
                .connect()
                .await
                .expect("Failed to connect to the database.");

            DB_CONN.set(database).expect("Setting `Database`");
        }
        Err(_) => println!("Failed to connect to the database."),
    }
}

pub fn get_db() -> &'static Database {
    DB_CONN.get().expect("Valid `Database`")
}
