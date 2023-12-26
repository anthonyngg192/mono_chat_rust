use chat_core::{
    models::User,
    permissions::{
        defn::ChannelPermission,
        r#impl::permission::{calculate_channel_permissions, DatabasePermissionQuery},
    },
    util::reference::Reference,
    Database, Result,
};
use rocket::State;
use rocket_empty::EmptyResponse;

#[openapi(tag = "Webhooks")]
#[delete("/<webhook_id>")]
pub async fn webhook_delete(
    db: &State<Database>,
    user: User,
    webhook_id: Reference,
) -> Result<EmptyResponse> {
    let webhook = webhook_id.as_webhook(db).await?;
    let channel = db.fetch_channel(&webhook.channel_id).await?;

    let mut query = DatabasePermissionQuery::new(db, &user).channel(&channel);
    calculate_channel_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::ManageWebhooks)?;

    webhook.delete(db).await.map(|_| EmptyResponse)
}
