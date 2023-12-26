use chat_core::{models::Emoji, util::reference::Reference, Database, Result};
use rocket::{serde::json::Json, State};

#[openapi(tag = "Emojis")]
#[get("/emoji/<emoji_id>")]
pub async fn fetch_emoji(db: &State<Database>, emoji_id: Reference) -> Result<Json<Emoji>> {
    emoji_id.as_emoji(db).await.map(Json)
}
