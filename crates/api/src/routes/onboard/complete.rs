use chat_core::{authifier::models::Session, models::User, Database, Error, Result};
use once_cell::sync::Lazy;
use regex::Regex;

use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};
use validator::Validate;

pub static RE_USERNAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\p{L}|[\d_.-])+$").unwrap());

#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataOnboard {
    #[validate(length(min = 2, max = 32), regex = "RE_USERNAME")]
    username: String,
}

#[openapi(tag = "Onboarding")]
#[post("/complete", data = "<data>")]
pub async fn req(
    db: &State<Database>,
    session: Session,
    user: Option<User>,
    data: Json<DataOnboard>,
) -> Result<Json<User>> {
    if user.is_some() {
        return Err(Error::AlreadyOnboard);
    }

    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    Ok(Json(
        User::create(db, data.username, session.user_id, None)
            .await?
            .into_self()
            .await,
    ))
}
