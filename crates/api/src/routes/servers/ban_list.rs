use chat_core::models::{File, ServerBan, User};
use chat_core::permissions::defn::ChannelPermission;
use chat_core::{perms, Db, Ref, Result};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
struct BannedUser {
    #[serde(rename = "_id")]
    pub id: String,

    pub username: String,
    pub avatar: Option<File>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct BanListResult {
    users: Vec<BannedUser>,
    bans: Vec<ServerBan>,
}

impl From<User> for BannedUser {
    fn from(user: User) -> Self {
        BannedUser {
            id: user.id,
            username: user.username,
            avatar: user.avatar,
        }
    }
}

#[openapi(tag = "Server Members")]
#[get("/<target>/bans")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<Json<BanListResult>> {
    let server = target.as_server(db).await?;
    perms(&user)
        .server(&server)
        .throw_permission(db, ChannelPermission::BanMembers)
        .await?;

    let bans = db.fetch_bans(&server.id).await?;
    let users = db
        .fetch_users(
            &bans
                .iter()
                .map(|x| &x.id.user)
                .cloned()
                .collect::<Vec<String>>(),
        )
        .await?
        .into_iter()
        .map(|x| BannedUser {
            id: x.id,
            username: x.username,
            avatar: x.avatar,
        })
        .collect();

    Ok(Json(BanListResult { users, bans }))
}
