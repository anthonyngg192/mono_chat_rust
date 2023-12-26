use chat_core::{
    models::{channel::DataCreateGroup, user::RelationshipStatus, Channel, User},
    Database, Error, Result,
};
use rocket::{serde::json::Json, State};
use validator::Validate;

#[openapi(tag = "Groups")]
#[post("/create", data = "<data>")]
pub async fn create_group(
    db: &State<Database>,
    user: User,
    data: Json<DataCreateGroup>,
) -> Result<Json<Channel>> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    for target in &data.users {
        match user.relationship_with(target) {
            RelationshipStatus::Friend | RelationshipStatus::User => {}
            _ => {
                return Err(Error::NotFriends);
            }
        }
    }

    Ok(Json(Channel::create_group(db, data, user.id).await?))
}
