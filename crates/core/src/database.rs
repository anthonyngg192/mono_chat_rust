use crate::{r#impl::MongoDb, AbstractDatabase};
use std::ops::Deref;

#[derive(Clone, Debug)]
pub enum Database {
    MongoDb(MongoDb),
}

pub enum DatabaseInfo {
    MongoDb(String),
    MongoDbFromClient(mongodb::Client),
}

impl DatabaseInfo {
    #[async_recursion]
    pub async fn connect(self) -> Result<Database, String> {
        Ok(match self {
            DatabaseInfo::MongoDb(uri) => {
                let client = mongodb::Client::with_uri_str(uri)
                    .await
                    .map_err(|_| "Failed to init db connection.".to_string())?;

                Database::MongoDb(MongoDb(client))
            }
            DatabaseInfo::MongoDbFromClient(client) => Database::MongoDb(MongoDb(client)),
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

impl From<Database> for authifier::Database {
    fn from(val: Database) -> Self {
        match val {
            Database::MongoDb(MongoDb(client)) => authifier::Database::MongoDb(
                authifier::database::MongoDb(client.database("revolt")),
            ),
        }
    }
}
