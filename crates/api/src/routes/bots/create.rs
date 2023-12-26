use chat_core::models::bot::DataCreateBot;
use chat_core::models::{Bot, User};
use chat_core::{Database, Error, Result};
use rocket::serde::json::Json;
use rocket::State;
use validator::Validate;

#[openapi(tag = "Bots")]
#[post("/create", data = "<info>")]
pub async fn create_bot(
    db: &State<Database>,
    user: User,
    info: Json<DataCreateBot>,
) -> Result<Json<Bot>> {
    let info = info.into_inner();
    let _ = info
        .validate()
        .map_err(|error| Error::FailedValidation { error });

    let bot = Bot::create(db, info.name, &user, None).await?;
    Ok(Json(bot))
}
