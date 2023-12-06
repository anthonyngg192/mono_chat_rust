use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RatelimitEventType {
    DiscriminatorChange,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RatelimitEvent {
    #[serde(rename = "_id")]
    pub id: String,
    pub target_id: String,
    pub event_type: RatelimitEventType,
}
