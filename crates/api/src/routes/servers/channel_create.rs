use chat_core::{
    models::{channel::DataCreateServerChannel, Channel, User},
    permissions::{
        defn::ChannelPermission,
        r#impl::permission::{calculate_server_permissions, DatabasePermissionQuery},
    },
    util::reference::Reference,
    Database, Error, Result,
};

use rocket::serde::json::Json;
use rocket::State;
use validator::Validate;

#[openapi(tag = "Server Information")]
#[post("/<server>/channels", data = "<data>")]
pub async fn create_server_channel(
    db: &State<Database>,
    user: User,
    server: Reference,
    data: Json<DataCreateServerChannel>,
) -> Result<Json<Channel>> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let mut server = server.as_server(db).await?;
    let mut query = DatabasePermissionQuery::new(db, &user).server(&server);
    calculate_server_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::ManageChannel)?;

    Channel::create_server_channel(db, &mut server, data, true)
        .await
        .map(Json)
}
