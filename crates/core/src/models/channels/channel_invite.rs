use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(tag = "type")]
pub enum Invite {
    Server {
        #[serde(rename = "_id")]
        code: String,
        server: String,
        creator: String,
        channel: String,
    },
    Group {
        #[serde(rename = "_id")]
        code: String,
        creator: String,
        channel: String,
    },
}
