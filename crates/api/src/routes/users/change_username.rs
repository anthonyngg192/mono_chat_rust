use chat_core::{authifier::models::Account, models::User, Database, Error, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};
use validator::Validate;

pub static RE_USERNAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\p{L}|[\d_.-])+$").unwrap());

#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataChangeUsername {
    #[validate(length(min = 2, max = 32), regex = "RE_USERNAME")]
    username: String,

    #[validate(length(min = 8, max = 1024))]
    password: String,
}

#[openapi(tag = "User Information")]
#[patch("/@me/username", data = "<data>")]
pub async fn req(
    db: &State<Database>,
    account: Account,
    mut user: User,
    data: Json<DataChangeUsername>,
) -> Result<Json<User>> {
    let data = data.into_inner();
    let _ = data
        .validate()
        .map_err(|error| Error::FailedValidation { error });

    account
        .verify_password(&data.password)
        .map_err(|_| Error::InvalidCredentials)?;

    user.update_username(db, data.username).await?;
    Ok(Json(user.foreign()))
}
