use chat_core::{
    models::{bot::OwnedBotsResponse, user::User},
    Database, Result,
};
use futures::future::join_all;
use rocket::{serde::json::Json, State};

#[openapi(tag = "Bots")]
#[get("/@me")]
pub async fn fetch_owned_bots(db: &State<Database>, user: User) -> Result<Json<OwnedBotsResponse>> {
    let mut bots = db.fetch_bots_by_user(&user.id).await?;
    let user_ids = bots
        .iter()
        .map(|x| x.id.to_owned())
        .collect::<Vec<String>>();

    let mut users = db.fetch_users(&user_ids).await?;

    // Ensure the lists match up exactly.
    bots.sort_by(|a, b| a.id.cmp(&b.id));
    users.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(Json(OwnedBotsResponse {
        users: join_all(users.into_iter().map(|user| user.into_self())).await,
        bots: bots.into_iter().collect(),
    }))
}
