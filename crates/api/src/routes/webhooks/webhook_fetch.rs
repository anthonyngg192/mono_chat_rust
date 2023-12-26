use chat_core::models::channel::{ResponseWebhook, Webhook};
use chat_core::permissions::defn::ChannelPermission;
use chat_core::permissions::r#impl::permission::{
    calculate_channel_permissions, DatabasePermissionQuery,
};
use chat_core::{models::User, util::reference::Reference, Database};

use chat_core::Result;
use rocket::{serde::json::Json, State};

#[openapi(tag = "Webhooks")]
#[get("/<webhook_id>")]
pub async fn webhook_fetch(
    db: &State<Database>,
    webhook_id: Reference,
    user: User,
) -> Result<Json<ResponseWebhook>> {
    let webhook = webhook_id.as_webhook(db).await?;
    let channel = db.fetch_channel(&webhook.channel_id).await?;

    let mut query = DatabasePermissionQuery::new(db, &user).channel(&channel);
    calculate_channel_permissions(&mut query)
        .await
        .throw_if_lacking_channel_permission(ChannelPermission::ViewChannel)?;

    Ok(Json(std::convert::Into::<Webhook>::into(webhook).into()))
}
