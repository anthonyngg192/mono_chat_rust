use chat_core::{Database, Ref, Result};

use rocket::{serde::json::Json, State};
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct FlagResponse {
    flags: u32,
}

#[openapi(tag = "User Information")]
#[get("/<target>/flags")]
pub async fn fetch_user_flags(db: &State<Database>, target: Ref) -> Result<Json<FlagResponse>> {
    let flags = if let Ok(target) = target.as_user(db).await {
        target.flags
    } else {
        0
    };

    Ok(Json(FlagResponse { flags }))
}
