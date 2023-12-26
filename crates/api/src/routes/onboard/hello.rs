use chat_core::{authifier::models::Session, models::User};
use rocket::serde::json::Json;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct DataHello {
    onboarding: bool,
}

#[openapi(tag = "Onboarding")]
#[get("/hello")]
pub async fn req(_session: Session, user: Option<User>) -> Json<DataHello> {
    Json(DataHello {
        onboarding: user.is_none(),
    })
}
