use chat_core::{Database, DatabaseInfo};
use once_cell::sync::OnceCell;

static DB_CONN: OnceCell<Database> = OnceCell::new();

pub async fn connect() {
    let database = DatabaseInfo::Auto
        .connect()
        .await
        .expect("Failed to connect to the database.");

    DB_CONN.set(database).expect("Setting `Database`");
}

pub fn get_db() -> &'static Database {
    DB_CONN.get().expect("Valid `Database`")
}
