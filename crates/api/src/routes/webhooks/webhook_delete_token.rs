use chat_core::Result;
use chat_core::{util::reference::Reference, Database};
use rocket::State;
use rocket_empty::EmptyResponse;

#[openapi(tag = "Webhooks")]
#[delete("/<webhook_id>/<token>")]
pub async fn webhook_delete_token(
    db: &State<Database>,
    webhook_id: Reference,
    token: String,
) -> Result<EmptyResponse> {
    let webhook = webhook_id.as_webhook(db).await?;
    webhook.assert_token(&token)?;
    webhook.delete(db).await.map(|_| EmptyResponse)
}
