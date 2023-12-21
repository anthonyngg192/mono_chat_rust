use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Default)]
#[serde(tag = "type")]
pub enum Metadata {
    #[default]
    File,
    Text,
    Image {
        width: isize,
        height: isize,
    },
    Video {
        width: isize,
        height: isize,
    },
    Audio,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Default)]
pub struct File {
    #[serde(rename = "_id")]
    pub id: String,
    pub tag: String,
    pub filename: String,
    pub metadata: Metadata,
    pub content_type: String,
    pub size: isize,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reported: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_id: Option<String>,
}
