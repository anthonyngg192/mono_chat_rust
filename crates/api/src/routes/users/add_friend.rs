use chat_core::models::User;
use chat_core::{Database, Error, Ref, Result};

use rocket::serde::json::Json;
use rocket::State;

#[openapi(tag = "Relationships")]
#[put("/<target>/friend")]
pub async fn req(db: &State<Database>, user: User, target: Ref) -> Result<Json<User>> {
    let mut target = target.as_user(db).await?;

    if user.bot.is_some() || target.bot.is_some() {
        return Err(Error::IsBot);
    }

    user.add_friend(db, &mut target).await?;
    Ok(Json(target.with_auto_perspective(db, &user).await))
}
