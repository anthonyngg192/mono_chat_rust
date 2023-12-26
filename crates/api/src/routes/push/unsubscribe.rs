use chat_core::{
    authifier::{models::Session, Authifier},
    EmptyResponse, Error, Result,
};

use rocket::State;

#[openapi(tag = "Web Push")]
#[post("/unsubscribe")]
pub async fn req(authifier: &State<Authifier>, mut session: Session) -> Result<EmptyResponse> {
    session.subscription = None;
    session
        .save(authifier)
        .await
        .map(|_| EmptyResponse)
        .map_err(|_| Error::DatabaseError {
            operation: "save",
            with: "session",
        })
}
