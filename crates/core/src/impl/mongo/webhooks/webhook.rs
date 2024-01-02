use crate::{
    models::channel::{FieldsWebhook, PartialWebhook, Webhook},
    r#impl::{mongo::IntoDocumentPath, MongoDb},
    AbstractWebhook, Error, Result,
};
use futures::StreamExt;

static COL: &str = "channel_webhooks";

#[async_trait]
impl AbstractWebhook for MongoDb {
    async fn insert_webhook(&self, webhook: &Webhook) -> Result<()> {
        self.insert_one(COL, webhook).await.map(|_| ())
    }

    async fn fetch_webhook(&self, webhook_id: &str) -> Result<Webhook> {
        self.find_one_by_id(COL, webhook_id).await
    }

    async fn fetch_webhooks_for_channel(&self, channel_id: &str) -> Result<Vec<Webhook>> {
        Ok(self
            .col::<Webhook>(COL)
            .find(
                doc! {
                    "channel_id": channel_id,
                },
                None,
            )
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find",
                with: COL,
            })?
            .filter_map(|s| async {
                if cfg!(debug_assertions) {
                    Some(s.unwrap())
                } else {
                    s.ok()
                }
            })
            .collect()
            .await)
    }

    async fn update_webhook(
        &self,
        webhook_id: &str,
        partial: &PartialWebhook,
        remove: &[FieldsWebhook],
    ) -> Result<()> {
        self.update_one_by_id(
            COL,
            webhook_id,
            partial,
            remove.iter().map(|x| x as &dyn IntoDocumentPath).collect(),
            None,
        )
        .await
        .map(|_| ())
    }

    async fn delete_webhook(&self, webhook_id: &str) -> Result<()> {
        self.delete_one_by_id(COL, webhook_id).await.map(|_| ())
    }
}

impl IntoDocumentPath for FieldsWebhook {
    fn as_path(&self) -> Option<&'static str> {
        Some(match self {
            FieldsWebhook::Avatar => "avatar",
        })
    }
}
