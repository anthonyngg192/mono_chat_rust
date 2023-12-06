use bson::{doc, Document};

use crate::{models::Emoji, AbstractEmoji, Error, Result};

use super::super::MongoDb;

static COL: &str = "emojis";

#[async_trait]
impl AbstractEmoji for MongoDb {
    async fn fetch_emoji(&self, id: &str) -> Result<Emoji> {
        self.find_one_by_id(COL, id).await
    }

    async fn fetch_emoji_by_parent_id(&self, parent_id: &str) -> Result<Vec<Emoji>> {
        self.find(
            COL,
            doc! {
                "parent.id":parent_id
            },
        )
        .await
    }

    async fn fetch_emoji_by_parent_ids(&self, parent_ids: &[String]) -> Result<Vec<Emoji>> {
        self.find(
            COL,
            doc! {
                "parent.id":{
                    "$in":parent_ids
                }
            },
        )
        .await
    }

    async fn insert_emoji(&self, emoji: &Emoji) -> Result<()> {
        self.insert_one(COL, emoji).await.map(|_| ())
    }

    async fn detach_emoji(&self, emoji: &Emoji) -> Result<()> {
        self.col::<Document>(COL)
            .update_one(
                doc! {
                    "_id":&emoji.id,
                },
                doc! {
                    "$set":{
                        "parent":{
                            "type":"Detached"
                        }
                    }
                },
                None,
            )
            .await
            .map(|_| ())
            .map_err(|_| Error::DatabaseError {
                operation: "update_one",
                with: "emojis",
            })
    }
}
