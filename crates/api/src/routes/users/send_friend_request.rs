use chat_core::models::User;
use chat_core::{Database, Error, Result};

use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct DataSendFriendRequest {
    username: String,
}

#[openapi(tag = "Relationships")]
#[post("/friend", data = "<data>")]
pub async fn req(
    db: &State<Database>,
    user: User,
    data: Json<DataSendFriendRequest>,
) -> Result<Json<User>> {
    if let Some((username, _)) = data.username.split_once('#') {
        let mut target = db.fetch_user_by_username(username).await?;

        if user.bot.is_some() || target.bot.is_some() {
            return Err(Error::IsBot);
        }

        user.add_friend(db, &mut target).await?;
        Ok(Json(target.with_auto_perspective(db, &user).await))
    } else {
        Err(Error::InvalidProperty)
    }
}
