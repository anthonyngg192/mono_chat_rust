use chat_core::models::{Channel, File, Invite};
use chat_core::{Db, Ref, Result};
use rocket::serde::json::Json;
use serde::Serialize;

#[allow(clippy::large_enum_variant)]
#[derive(Serialize, JsonSchema)]
#[serde(tag = "type")]
pub enum InviteResponse {
    Server {
        code: String,
        server_id: String,
        server_name: String,

        #[serde(skip_serializing_if = "Option::is_none")]
        server_icon: Option<File>,

        #[serde(skip_serializing_if = "Option::is_none")]
        server_banner: Option<File>,

        #[serde(skip_serializing_if = "Option::is_none")]
        server_flags: Option<i32>,

        channel_id: String,
        channel_name: String,

        #[serde(skip_serializing_if = "Option::is_none")]
        channel_description: Option<String>,

        user_name: String,

        #[serde(skip_serializing_if = "Option::is_none")]
        user_avatar: Option<File>,

        member_count: i64,
    },
    Group {
        code: String,
        channel_id: String,
        channel_name: String,

        #[serde(skip_serializing_if = "Option::is_none")]
        channel_description: Option<String>,
        user_name: String,

        #[serde(skip_serializing_if = "Option::is_none")]
        user_avatar: Option<File>,
    },
}

#[openapi(tag = "Invites")]
#[get("/<target>")]
pub async fn req(db: &Db, target: Ref) -> Result<Json<InviteResponse>> {
    Ok(Json(match target.as_invite(db).await? {
        Invite::Server {
            creator, channel, ..
        } => {
            let channel = db.fetch_channel(&channel).await?;
            let user = db.fetch_user(&creator).await?;

            match channel {
                Channel::TextChannel {
                    id,
                    server,
                    name,
                    description,
                    ..
                }
                | Channel::VoiceChannel {
                    id,
                    server,
                    name,
                    description,
                    ..
                } => {
                    let server = db.fetch_server(&server).await?;
                    InviteResponse::Server {
                        code: target.id,
                        member_count: db.fetch_member_count(&server.id).await? as i64,
                        server_id: server.id,
                        server_name: server.name,
                        server_icon: server.icon,
                        server_banner: server.banner,
                        server_flags: server.flags,
                        channel_id: id,
                        channel_name: name,
                        channel_description: description,
                        user_name: user.username,
                        user_avatar: user.avatar,
                    }
                }
                _ => unreachable!(),
            }
        }
        Invite::Group {
            channel, creator, ..
        } => {
            let channel = db.fetch_channel(&channel).await?;
            let user = db.fetch_user(&creator).await?;

            match channel {
                Channel::Group {
                    id,
                    name,
                    description,
                    ..
                } => InviteResponse::Group {
                    code: target.id,
                    channel_id: id,
                    channel_name: name,
                    channel_description: description,
                    user_name: user.username,
                    user_avatar: user.avatar,
                },
                _ => unreachable!(),
            }
        }
    }))
}
