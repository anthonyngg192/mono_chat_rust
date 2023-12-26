use chat_core::{
    models::{
        bot::{Bot, DataEditBot, PartialBot},
        User,
    },
    util::reference::Reference,
    Database, Error, Result,
};
use rocket::serde::json::Json;
use rocket::State;
use validator::Validate;

#[openapi(tag = "Bots")]
#[patch("/<target>", data = "<data>")]
pub async fn edit_bot(
    db: &State<Database>,
    user: User,
    target: Reference,
    data: Json<DataEditBot>,
) -> Result<Json<Bot>> {
    let data = data.into_inner();
    let _ = data
        .validate()
        .map_err(|error| Error::FailedValidation { error });
    let mut bot = target.as_bot(db).await?;
    if bot.owner != user.id {
        return Err(Error::NotFound);
    }

    if let Some(name) = data.name {
        let mut user = db.fetch_user(&bot.id).await?;
        user.update_username(db, name).await?;
    }

    if data.public.is_none()
        && data.analytics.is_none()
        && data.interactions_url.is_none()
        && data.remove.is_none()
    {
        return Ok(Json(bot));
    }

    let DataEditBot {
        public,
        analytics,
        interactions_url,
        remove,
        ..
    } = data;

    let partial = PartialBot {
        public,
        analytics,
        interactions_url,
        ..Default::default()
    };

    bot.update(
        db,
        partial,
        remove.unwrap_or_default().into_iter().collect(),
    )
    .await?;

    Ok(Json(bot))
}
