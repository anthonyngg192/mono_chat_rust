use serde::{Deserialize, Serialize};

use crate::models::{Channel, Message, Server, User};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "_type")]
pub enum SnapshotContent {
    Message {
        #[serde(rename = "_prior_context", default)]
        prior_context: Vec<Message>,

        #[serde(rename = "_leading_context", default)]
        leading_context: Vec<Message>,

        #[serde(flatten)]
        message: Message,
    },
    Server(Server),
    User(User),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Snapshot {
    #[serde(rename = "_id")]
    pub id: String,

    pub report_id: String,
    pub content: SnapshotContent,
}

#[derive(Serialize, Debug)]
pub struct SnapshotWithContext {
    #[serde(flatten)]
    pub snapshot: Snapshot,

    #[serde(rename = "_users", skip_serializing_if = "Vec::is_empty")]
    pub users: Vec<User>,

    #[serde(rename = "_channels", skip_serializing_if = "Vec::is_empty")]
    pub channels: Vec<Channel>,

    #[serde(rename = "_server", skip_serializing_if = "Option::is_none")]
    pub server: Option<Server>,
}
