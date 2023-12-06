use crate::{
    models::ratelimit_events::ratelimit::RatelimitEvent, query,
    ratelimiter::ratelimit::AbstractRatelimitEvent, Result,
};

static COL: &str = "ratelimit_events";
use super::super::MongoDb;

#[async_trait]
impl AbstractRatelimitEvent for MongoDb {
    async fn insert_ratelimit_event(&self, event: &RatelimitEvent) -> Result<()> {
        query!(self, insert_one, COL, &event).map(|_| ())
    }
}
