use chat_core::{
    models::{bot::FetchBotResponse, User},
    util::reference::Reference,
    Database, Error, Result,
};
use rocket::{serde::json::Json, State};

#[openapi(tag = "Bots")]
#[get("/<bot>")]
pub async fn fetch_bot(
    db: &State<Database>,
    user: User,
    bot: Reference,
) -> Result<Json<FetchBotResponse>> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let bot = bot.as_bot(db).await?;
    if bot.owner != user.id {
        return Err(Error::NotFound);
    }

    Ok(Json(FetchBotResponse {
        user: db.fetch_user(&bot.id).await?.into(db, None).await,
        bot,
    }))
}
