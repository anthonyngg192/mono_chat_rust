use chat_core::{
    models::{Message, User},
    permissions::defn::ChannelPermission,
    perms, Db, Error, Ref, Result,
};

#[derive(Validate, Deserialize, JsonSchema)]
pub struct OptionsBulkDelete {
    /// Message IDs
    #[validate(length(min = 1, max = 100))]
    ids: Vec<String>,
}

use chrono::Utc;
use rocket::serde::json::Json;
use rocket_empty::EmptyResponse;
use serde::Deserialize;
use validator::Validate;

#[openapi(tag = "Messaging")]
#[delete("/<target>/messages/bulk", data = "<options>", rank = 1)]
pub async fn req(
    db: &Db,
    user: User,
    target: Ref,
    options: Json<OptionsBulkDelete>,
) -> Result<EmptyResponse> {
    let options = options.into_inner();
    options
        .validate()
        .map_err(|error| Error::FailedValidation { error })?;

    for id in &options.ids {
        if ulid::Ulid::from_string(id)
            .map_err(|_| Error::InvalidOperation)?
            .datetime()
            .signed_duration_since(Utc::now())
            .num_days()
            .abs()
            > 7
        {
            return Err(Error::InvalidOperation);
        }
    }

    perms(&user)
        .channel(&target.as_channel(db).await?)
        .throw_permission(db, ChannelPermission::ManageMessages)
        .await?;

    Message::bulk_delete(db, &target.id, options.ids)
        .await
        .map(|_| EmptyResponse)
}
