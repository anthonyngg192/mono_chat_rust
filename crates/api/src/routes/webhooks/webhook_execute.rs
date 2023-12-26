use chat_core::{
    models::{
        message::{DataMessageSend, MessageAuthor},
        Message,
    },
    permissions::{defn::ChannelPermission, r#impl::PermissionValue},
    util::{idempotency::IdempotencyKey, reference::Reference},
    Database,
};

use chat_core::{Error, Result};
use rocket::{serde::json::Json, State};

use validator::Validate;

#[openapi(tag = "Webhooks")]
#[post("/<webhook_id>/<token>", data = "<data>")]
pub async fn webhook_execute(
    db: &State<Database>,
    webhook_id: Reference,
    token: String,
    data: Json<DataMessageSend>,
    idempotency: IdempotencyKey,
) -> Result<Json<Message>> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let webhook = webhook_id.as_webhook(db).await?;
    webhook.assert_token(&token)?;

    let permissions: PermissionValue = webhook.permissions.into();
    permissions.throw_if_lacking_channel_permission(ChannelPermission::SendMessage)?;

    if data.attachments.as_ref().map_or(false, |v| !v.is_empty()) {
        permissions.throw_if_lacking_channel_permission(ChannelPermission::UploadFiles)?;
    }

    if data.embeds.as_ref().map_or(false, |v| !v.is_empty()) {
        permissions.throw_if_lacking_channel_permission(ChannelPermission::SendEmbeds)?;
    }

    if data.masquerade.is_some() {
        permissions.throw_if_lacking_channel_permission(ChannelPermission::Masquerade)?;
    }

    if data.interactions.is_some() {
        permissions.throw_if_lacking_channel_permission(ChannelPermission::React)?;
    }

    let channel = db.fetch_channel(&webhook.channel_id).await?;

    Ok(Json(
        Message::create_from_api(
            db,
            channel,
            data,
            MessageAuthor::Webhook(&webhook),
            idempotency,
            true,
        )
        .await?,
    ))
}
