use ulid::Ulid;

use crate::{
    models::ratelimit_events::ratelimit::{RatelimitEvent, RatelimitEventType},
    Database, Result,
};

impl RatelimitEvent {
    pub async fn create(
        db: &Database,
        target_id: String,
        event_type: RatelimitEventType,
    ) -> Result<()> {
        db.insert_ratelimit_event(&RatelimitEvent {
            id: Ulid::new().to_string(),
            target_id,
            event_type,
        })
        .await
    }
}
