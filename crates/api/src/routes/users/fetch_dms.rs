use chat_core::{
    models::{Channel, User},
    Database, Result,
};

use rocket::{serde::json::Json, State};

#[openapi(tag = "Direct Messaging")]
#[get("/dms")]
pub async fn req(db: &State<Database>, user: User) -> Result<Json<Vec<Channel>>> {
    db.find_direct_messages(&user.id).await.map(Json)
}
