use chat_core::{
    models::{
        server::{CreateServerLegacyResponse, DataCreateServer},
        Member, Server, User,
    },
    Database, Error, Result,
};

use rocket::serde::json::Json;
use rocket::State;
use validator::Validate;

#[openapi(tag = "Server Information")]
#[post("/create", data = "<data>")]
pub async fn create_server(
    db: &State<Database>,
    user: User,
    data: Json<DataCreateServer>,
) -> Result<Json<CreateServerLegacyResponse>> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let data = data.into_inner();
    let _ = data
        .validate()
        .map_err(|error| Error::FailedValidation { error });

    user.can_acquire_server(db).await?;

    let (server, channels) = Server::create(db, data, &user, true).await?;
    let channels = Member::create(db, &server, &user, Some(channels)).await?;

    Ok(Json(CreateServerLegacyResponse {
        server,
        channels: channels.into_iter().collect(),
    }))
}
