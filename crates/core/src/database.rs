use crate::{
    r#impl::MongoDb,
    variables::delta::{MONGODB, MONGODB_DATABASE_NAME},
    AbstractDatabase,
};
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
        Ok(match self {
            DatabaseInfo::Auto => {
                DatabaseInfo::MongoDb {
                    uri: MONGODB.to_string(),
                    database_name: MONGODB_DATABASE_NAME.to_string(),
                }
                .connect()
                .await?
            }
            DatabaseInfo::MongoDb { uri, database_name } => {
                let client = ::mongodb::Client::with_uri_str(uri)
                    .await
                    .map_err(|_err| "Failed to init db connection.".to_string())?;

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
