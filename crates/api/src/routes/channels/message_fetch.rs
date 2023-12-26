use chat_core::{
    models::{Message, User},
    permissions::{
        defn::ChannelPermission,
        r#impl::permission::{calculate_channel_permissions, DatabasePermissionQuery},
    },
    util::reference::Reference,
    Database, Error, Result,
};
use rocket::{serde::json::Json, State};

#[openapi(tag = "Messaging")]
#[get("/<target>/messages/<msg>")]
pub async fn req(
    db: &State<Database>,
    user: User,
    target: Reference,
    msg: Reference,
) -> Result<Json<Message>> {
    let channel = target.as_channel(db).await?;

    let mut query = DatabasePermissionQuery::new(db, &user).channel(&channel);
    calculate_channel_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::ViewChannel)?;

    let message = msg.as_message(db).await?;
    if message.channel != channel.id() {
        return Err(Error::NotFound);
    }

    Ok(Json(message))
}
