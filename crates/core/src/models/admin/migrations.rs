use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MigrationInfo {
    #[serde(rename = "_id")]
    id: i32,
    revision: i32,
}
