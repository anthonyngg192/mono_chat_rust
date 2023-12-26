use chat_core::models::User;
use chat_core::{Database, Error, Result};

use rocket::serde::json::Json;
use rocket::State;

#[openapi(tag = "Relationships")]
#[delete("/<target>/friend")]
pub async fn req(db: &State<Database>, user: User, target: String) -> Result<Json<User>> {
    let mut target = db.fetch_user(&target).await?;

    if user.bot.is_some() || target.bot.is_some() {
        return Err(Error::IsBot);
    }

    user.remove_friend(db, &mut target).await?;
    Ok(Json(target.with_auto_perspective(db, &user).await))
}
