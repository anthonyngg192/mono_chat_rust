use chat_core::models::{Emoji, User};
use chat_core::{perms, Db, Ref, Result};

use rocket::serde::json::Json;

#[openapi(tag = "Server Customisation")]
#[get("/<target>/emojis")]
pub async fn list_emoji(db: &Db, user: User, target: Ref) -> Result<Json<Vec<Emoji>>> {
    let server = target.as_server(db).await?;
    perms(&user).server(&server).calc(db).await?;

    db.fetch_emoji_by_parent_id(&server.id).await.map(Json)
}
