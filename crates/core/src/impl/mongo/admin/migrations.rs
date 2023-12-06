use log::info;

use super::super::MongoDb;
use crate::{AbstractMigrations, Result};
mod init;
mod scripts;

#[async_trait]
impl AbstractMigrations for MongoDb {
    async fn migrate_database(&self) -> Result<()> {
        info!("Migrating the database");

        let list = self
            .list_database_names(None, None)
            .await
            .expect("Failed to fetch data");

        if list.iter().any(|x| x == "rust_chatting") {
            scripts::migrate_database(self).await;
        } else {
            init::create_database(self).await;
        }
        Ok(())
    }
}
