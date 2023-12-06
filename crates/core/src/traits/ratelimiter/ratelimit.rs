use crate::{models::ratelimit_events::ratelimit::RatelimitEvent, Result};

#[async_trait]
pub trait AbstractRatelimitEvent: Sync + Send {
    async fn insert_ratelimit_event(&self, event: &RatelimitEvent) -> Result<()>;
}
