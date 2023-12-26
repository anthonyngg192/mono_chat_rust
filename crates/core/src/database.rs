use crate::util::config::sys_config::config;

use crate::{r#impl::MongoDb, AbstractDatabase};
use std::ops::Deref;

#[derive(Clone, Debug)]
pub enum Database {
    MongoDb(MongoDb),
}

pub enum DatabaseInfo {
    Auto,
    MongoDb { uri: String, database_name: String },
    MongoDbFromClient(::mongodb::Client, String),
}

impl DatabaseInfo {
    #[async_recursion]
    pub async fn connect(self) -> Result<Database, String> {
        let config = config().await;
        Ok(match self {
            DatabaseInfo::Auto => {
                if !config.database.mongodb.is_empty() {
                    DatabaseInfo::MongoDb {
                        uri: config.database.mongodb,
                        database_name: "revolt".to_string(),
                    }
                    .connect()
                    .await?
                } else {
                    unreachable!("must specify REFERENCE or MONGODB")
                }
            }
            DatabaseInfo::MongoDb { uri, database_name } => {
                let client = ::mongodb::Client::with_uri_str(uri)
                    .await
                    .map_err(|_| "Failed to init db connection.".to_string())?;

                Database::MongoDb(MongoDb(client, database_name))
            }
            DatabaseInfo::MongoDbFromClient(client, database_name) => {
                Database::MongoDb(MongoDb(client, database_name))
            }
        })
    }
}

impl Deref for Database {
    type Target = dyn AbstractDatabase;

    fn deref(&self) -> &Self::Target {
        match self {
            Database::MongoDb(mongo) => mongo,
        }
    }
}
