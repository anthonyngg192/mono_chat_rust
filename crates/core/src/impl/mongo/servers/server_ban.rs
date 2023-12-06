use bson::doc;

use crate::{
    models::{server_member::MemberCompositeKey, ServerBan},
    AbstractServerBan, Result,
};

use super::super::MongoDb;

static COL: &str = "server_bans";

#[async_trait]
impl AbstractServerBan for MongoDb {
    async fn fetch_ban(&self, server: &str, user: &str) -> Result<ServerBan> {
        self.find_one(
            COL,
            doc! {
                "_id.server":server,
                "_id.user":user
            },
        )
        .await
    }

    /// Fetch all bans in a server
    async fn fetch_bans(&self, server: &str) -> Result<Vec<ServerBan>> {
        self.find(
            COL,
            doc! {
                "_id.server":server
            },
        )
        .await
    }

    /// Insert new ban into database
    async fn insert_ban(&self, ban: &ServerBan) -> Result<()> {
        self.insert_one(COL, ban).await.map(|_| ())
    }

    /// Delete a ban from the database
    async fn delete_ban(&self, id: &MemberCompositeKey) -> Result<()> {
        self.delete_one(
            COL,
            doc! {
                "_id.server":&id.server,
                "_id.user":&id.user
            },
        )
        .await
        .map(|_| ())
    }
}
