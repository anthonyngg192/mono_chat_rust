use crate::models::channel_unread::ChannelUnread;
use crate::Result;

#[async_trait]
pub trait AbstractChannelUnread: Sync + Send {
    async fn acknowledge_message(&self, channel: &str, user: &str, message: &str) -> Result<()>;
    async fn acknowledge_channels(&self, user: &str, channels: &[String]) -> Result<()>;
    async fn add_mention_to_unread<'a>(
        &self,
        channel: &str,
        user: &str,
        ids: &[String],
    ) -> Result<()>;
    async fn fetch_unreads(&self, user: &str) -> Result<Vec<ChannelUnread>>;
}
