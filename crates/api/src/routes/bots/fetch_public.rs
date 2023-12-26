use chat_core::{
    models::{bot::PublicBot, User},
    util::reference::Reference,
    Database, Error, Result,
};
use rocket::{serde::json::Json, State};

#[openapi(tag = "Bots")]
#[get("/<target>/invite")]
pub async fn fetch_public_bot(
    db: &State<Database>,
    user: Option<User>,
    target: Reference,
) -> Result<Json<PublicBot>> {
    let bot = db.fetch_bot(&target.id).await?;
    if !bot.public && user.map_or(true, |x| x.id != bot.owner) {
        return Err(Error::NotFound);
    }

    let user = db.fetch_user(&bot.id).await?;
    Ok(Json(bot.into_public_bot(user)))
}
