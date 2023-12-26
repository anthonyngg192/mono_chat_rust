use chat_core::{
    authifier::{
        models::{Session, WebPushSubscription},
        Authifier,
    },
    EmptyResponse, Error, Result,
};

use rocket::{serde::json::Json, State};

#[openapi(tag = "Web Push")]
#[post("/subscribe", data = "<data>")]
pub async fn req(
    authifier: &State<Authifier>,
    mut session: Session,
    data: Json<WebPushSubscription>,
) -> Result<EmptyResponse> {
    session.subscription = Some(data.into_inner());
    session
        .save(authifier)
        .await
        .map(|_| EmptyResponse)
        .map_err(|_| Error::DatabaseError {
            operation: "save",
            with: "session",
        })
}
