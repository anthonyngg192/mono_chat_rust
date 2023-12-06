use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct ChannelCompositeKey {
    pub channel: String,
    pub user: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct ChannelUnread {
    #[serde(rename = "_id")]
    pub id: ChannelCompositeKey,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mentions: Option<Vec<String>>,
}
