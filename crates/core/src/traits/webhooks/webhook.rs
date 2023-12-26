use crate::models::channel::{FieldsWebhook, PartialWebhook, Webhook};
use crate::Result;

#[async_trait]
pub trait AbstractWebhook: Sync + Send {
    async fn insert_webhook(&self, webhook: &Webhook) -> Result<()>;
    async fn fetch_webhook(&self, webhook_id: &str) -> Result<Webhook>;
    async fn fetch_webhooks_for_channel(&self, channel_id: &str) -> Result<Vec<Webhook>>;
    async fn update_webhook(
        &self,
        webhook_id: &str,
        partial: &PartialWebhook,
        remove: &[FieldsWebhook],
    ) -> Result<()>;
    async fn delete_webhook(&self, webhook_id: &str) -> Result<()>;
}
