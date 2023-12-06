use crate::models::message::{AppendMessage, Message, MessageQuery, PartialMessage};
use crate::Result;

#[async_trait]
pub trait AbstractMessage: Sync + Send {
    async fn fetch_message(&self, id: &str) -> Result<Message>;
    async fn insert_message(&self, message: &Message) -> Result<()>;
    async fn update_message(&self, id: &str, message: &PartialMessage) -> Result<()>;
    async fn append_message(&self, id: &str, append: &AppendMessage) -> Result<()>;
    async fn delete_message(&self, id: &str) -> Result<()>;
    async fn delete_messages(&self, channel: &str, ids: Vec<String>) -> Result<()>;
    async fn fetch_messages(&self, query: MessageQuery) -> Result<Vec<Message>>;
    async fn add_reaction(&self, id: &str, emoji: &str, user: &str) -> Result<()>;
    async fn remove_reaction(&self, id: &str, emoji: &str, user: &str) -> Result<()>;
    async fn clear_reaction(&self, id: &str, emoji: &str) -> Result<()>;
}
