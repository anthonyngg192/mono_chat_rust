use chat_core::{
    models::{ChannelUnread, User},
    Db, Result,
};

use rocket::serde::json::Json;

#[openapi(tag = "Sync")]
#[get("/unreads")]
pub async fn req(db: &Db, user: User) -> Result<Json<Vec<ChannelUnread>>> {
    db.fetch_unreads(&user.id).await.map(Json)
}
