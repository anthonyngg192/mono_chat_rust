use chat_core::models::channel::{DataEditWebhook, PartialWebhook, Webhook};
use chat_core::util::reference::Reference;
use chat_core::{Database, Error, Result};
use rocket::{serde::json::Json, State};
use validator::Validate;

#[openapi(tag = "Webhooks")]
#[patch("/<webhook_id>/<token>", data = "<data>")]
pub async fn webhook_edit_token(
    db: &State<Database>,
    webhook_id: Reference,
    token: String,
    data: Json<DataEditWebhook>,
) -> Result<Json<Webhook>> {
    let data = data.into_inner();
    let _ = data
        .validate()
        .map_err(|error| Error::FailedValidation { error });

    let mut webhook = webhook_id.as_webhook(db).await?;
    webhook.assert_token(&token)?;

    if data.name.is_none() && data.avatar.is_none() && data.remove.is_empty() {
        return Ok(Json(webhook));
    };

    let DataEditWebhook {
        name,
        avatar,
        permissions,
        remove,
    } = data;

    let mut partial = PartialWebhook {
        name,
        permissions,
        ..Default::default()
    };

    if let Some(avatar) = avatar {
        let file = db
            .find_and_use_attachment(&avatar, "avatars", "user", &webhook.id)
            .await?;

        partial.avatar = Some(file)
    }

    webhook
        .update(db, partial, remove.into_iter().collect())
        .await?;

    Ok(Json(webhook))
}
