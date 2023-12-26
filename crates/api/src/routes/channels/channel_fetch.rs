use chat_core::{
    models::{Channel, User},
    permissions::{
        defn::ChannelPermission,
        r#impl::permission::{calculate_channel_permissions, DatabasePermissionQuery},
    },
    util::reference::Reference,
    Database, Result,
};
use rocket::{serde::json::Json, State};

#[openapi(tag = "Channel Information")]
#[get("/<target>")]
pub async fn fetch_channel(
    db: &State<Database>,
    user: User,
    target: Reference,
) -> Result<Json<Channel>> {
    let channel = target.as_channel(db).await?;

    let mut query = DatabasePermissionQuery::new(db, &user).channel(&channel);
    calculate_channel_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::ViewChannel)?;

    Ok(Json(channel))
}
