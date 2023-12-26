use std::{
    fmt,
    time::{Duration, SystemTime},
};

use ulid::Ulid;

use crate::{
    models::ratelimit_events::ratelimit::{RatelimitEvent, RatelimitEventType},
    query,
    ratelimiter::ratelimit::AbstractRatelimitEvent,
    Error, Result,
};

static COL: &str = "ratelimit_events";
use super::super::MongoDb;

#[async_trait]
impl AbstractRatelimitEvent for MongoDb {
    async fn insert_ratelimit_event(&self, event: &RatelimitEvent) -> Result<()> {
        query!(self, insert_one, COL, &event).map(|_| ())
    }

    async fn has_ratelimited(
        &self,
        target_id: &str,
        event_type: RatelimitEventType,
        period: Duration,
        count: usize,
    ) -> Result<bool> {
        self.col::<RatelimitEvent>(COL)
            .count_documents(
                doc! {
                    "_id": {
                        "$gte": Ulid::from_datetime(SystemTime::now() - period).to_string()
                    },
                    "target_id": target_id,
                    "event_type": event_type.to_string()
                },
                None,
            )
            .await
            .map(|c| c as usize >= count)
            .map_err(|_| Error::DatabaseError {
                operation: "count_documents",
                with: COL,
            })
    }
}

impl fmt::Display for RatelimitEventType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
