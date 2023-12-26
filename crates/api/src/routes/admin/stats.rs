use chat_core::{models::stats::Stats, Db, Result};
use rocket::serde::json::Json;

#[openapi(tag = "Admin")]
#[get("/stats")]
pub async fn stats(db: &Db) -> Result<Json<Stats>> {
    Ok(Json(db.generate_stats().await?))
}
