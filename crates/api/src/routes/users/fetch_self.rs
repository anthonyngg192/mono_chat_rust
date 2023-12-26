use chat_core::models::User;
use chat_core::Result;

use rocket::serde::json::Json;

#[openapi(tag = "User Information")]
#[get("/@me")]
pub async fn req(user: User) -> Result<Json<User>> {
    Ok(Json(user.foreign()))
}
