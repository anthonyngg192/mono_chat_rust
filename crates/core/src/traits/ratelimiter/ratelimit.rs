use crate::{
    models::ratelimit_events::ratelimit::{RatelimitEvent, RatelimitEventType},
    Result,
};
use std::time::Duration;

#[async_trait]
pub trait AbstractRatelimitEvent: Sync + Send {
    async fn insert_ratelimit_event(&self, event: &RatelimitEvent) -> Result<()>;

    async fn has_ratelimited(
        &self,
        target_id: &str,
        event_type: RatelimitEventType,
        period: Duration,
        count: usize,
    ) -> Result<bool>;
}
