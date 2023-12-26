use chat_core::{
    models::{User, UserSettings},
    Db, Result,
};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct OptionsFetchSettings {
    keys: Vec<String>,
}

#[openapi(tag = "Sync")]
#[post("/settings/fetch", data = "<options>")]
pub async fn req(
    db: &Db,
    user: User,
    options: Json<OptionsFetchSettings>,
) -> Result<Json<UserSettings>> {
    db.fetch_user_settings(&user.id, &options.into_inner().keys)
        .await
        .map(Json)
}
