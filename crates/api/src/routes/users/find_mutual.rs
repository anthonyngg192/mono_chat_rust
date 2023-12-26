use chat_core::models::User;
use chat_core::{perms, Database, Error, Ref, Result};

use rocket::serde::json::Json;
use rocket::State;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct MutualResponse {
    users: Vec<String>,
    servers: Vec<String>,
}

#[openapi(tag = "Relationships")]
#[get("/<target>/mutual")]
pub async fn req(db: &State<Database>, user: User, target: Ref) -> Result<Json<MutualResponse>> {
    if target.id == user.id {
        return Err(Error::InvalidOperation);
    }

    let target = target.as_user(db).await?;

    if perms(&user)
        .user(&target)
        .calc_user(db)
        .await
        .get_view_profile()
    {
        Ok(Json(MutualResponse {
            users: db.fetch_mutual_user_ids(&user.id, &target.id).await?,
            servers: db.fetch_mutual_server_ids(&user.id, &target.id).await?,
        }))
    } else {
        Err(Error::NotFound)
    }
}
