use chat_core::models::User;
use chat_core::{Database, Result};

use rocket::serde::json::Json;
use rocket::State;

#[openapi(tag = "Relationships")]
#[put("/<target>/block")]
pub async fn req(db: &State<Database>, user: User, target: String) -> Result<Json<User>> {
    let mut target = db.fetch_user(&target).await?;
    user.block_user(db, &mut target).await?;
    Ok(Json(target.with_auto_perspective(db, &user).await))
}
