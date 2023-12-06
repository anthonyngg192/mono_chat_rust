use crate::models::Emoji;
use crate::Result;

#[async_trait]
pub trait AbstractEmoji: Sync + Send {
    async fn fetch_emoji(&self, id: &str) -> Result<Emoji>;
    async fn fetch_emoji_by_parent_id(&self, parent_id: &str) -> Result<Vec<Emoji>>;
    async fn fetch_emoji_by_parent_ids(&self, parent_ids: &[String]) -> Result<Vec<Emoji>>;
    async fn insert_emoji(&self, emoji: &Emoji) -> Result<()>;
    async fn detach_emoji(&self, emoji: &Emoji) -> Result<()>;
}
