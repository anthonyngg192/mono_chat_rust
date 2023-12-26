use chat_core::models::channel::Webhook;
use chat_core::Result;
use chat_core::{util::reference::Reference, Database};
use rocket::{serde::json::Json, State};

#[openapi(tag = "Webhooks")]
#[get("/<webhook_id>/<token>")]
pub async fn webhook_fetch_token(
    db: &State<Database>,
    webhook_id: Reference,
    token: String,
) -> Result<Json<Webhook>> {
    let webhook = webhook_id.as_webhook(db).await?;
    webhook.assert_token(&token)?;
    Ok(Json(webhook))
}
