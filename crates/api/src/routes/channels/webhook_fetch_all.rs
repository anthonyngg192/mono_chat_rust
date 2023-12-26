use chat_core::{
    models::{channel::Webhook, User},
    permissions::{
        defn::ChannelPermission,
        r#impl::permission::{calculate_channel_permissions, DatabasePermissionQuery},
    },
    util::reference::Reference,
    Database, Result,
};
use rocket::{serde::json::Json, State};

#[openapi(tag = "Webhooks")]
#[get("/<channel_id>/webhooks")]
pub async fn req(
    db: &State<Database>,
    user: User,
    channel_id: Reference,
) -> Result<Json<Vec<Webhook>>> {
    let channel = channel_id.as_channel(db).await?;

    let mut query = DatabasePermissionQuery::new(db, &user).channel(&channel);
    calculate_channel_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::ViewChannel)?;

    Ok(Json(
        db.fetch_webhooks_for_channel(channel.id())
            .await?
            .into_iter()
            .collect::<Vec<Webhook>>(),
    ))
}
