use serde::{Deserialize, Serialize};

use super::server_member::MemberCompositeKey;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ServerBan {
    #[serde(rename = "_id")]
    pub id: MemberCompositeKey,

    pub reason: Option<String>,
}
