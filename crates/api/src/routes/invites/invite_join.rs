use chat_core::{
    models::{Channel, Invite, Server, User},
    variables::delta::MAX_SERVER_COUNT,
    Db, Error, Ref, Result,
};

use rocket::serde::json::Json;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
#[serde(tag = "type")]
pub enum InviteJoinResponse {
    Server {
        channels: Vec<Channel>,
        server: Server,
    },
}

#[openapi(tag = "Invites")]
#[post("/<target>")]
pub async fn req(db: &Db, user: User, target: Ref) -> Result<Json<InviteJoinResponse>> {
    if user.bot.is_some() {
        return Err(Error::IsBot);
    }

    if !user.can_acquire_server(db).await? {
        return Err(Error::TooManyServers {
            max: *MAX_SERVER_COUNT,
        });
    }

    let invite = target.as_invite(db).await?;
    match &invite {
        Invite::Server { server, .. } => {
            let server = db.fetch_server(server).await?;
            let channels = server.create_member(db, user, None).await?;
            Ok(Json(InviteJoinResponse::Server { channels, server }))
        }
        _ => unreachable!(),
    }
}
