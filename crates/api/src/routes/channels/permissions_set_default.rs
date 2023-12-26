use chat_core::{
    models::{channel::PartialChannel, Channel, User},
    permissions::defn::ChannelPermission,
    permissions::defn::Override,
    perms, Db, Error, Ref, Result,
};
use rocket::serde::json::Json;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum DataDefaultChannelPermissions {
    Value { permissions: u64 },
    Field { permissions: Override },
}

#[openapi(tag = "Channel Permissions")]
#[put("/<target>/permissions/default", data = "<data>", rank = 1)]
pub async fn req(
    db: &Db,
    user: User,
    target: Ref,
    data: Json<DataDefaultChannelPermissions>,
) -> Result<Json<Channel>> {
    let data = data.into_inner();

    let mut channel = target.as_channel(db).await?;
    let mut perm = perms(&user).channel(&channel);
    perm.throw_permission_and_view_channel(db, ChannelPermission::ManagePermissions)
        .await?;

    match &channel {
        Channel::Group { .. } => {
            if let DataDefaultChannelPermissions::Value { permissions } = data {
                channel
                    .update(
                        db,
                        PartialChannel {
                            permission: Some(permissions as i64),
                            ..Default::default()
                        },
                        vec![],
                    )
                    .await?;
            } else {
                return Err(Error::InvalidOperation);
            }
        }
        Channel::TextChannel {
            default_permissions,
            ..
        }
        | Channel::VoiceChannel {
            default_permissions,
            ..
        } => {
            if let DataDefaultChannelPermissions::Field { permissions } = data {
                perm.throw_permission_override(
                    db,
                    default_permissions.map(|x| x.into()),
                    permissions,
                )
                .await?;

                channel
                    .update(
                        db,
                        PartialChannel {
                            default_permissions: Some(permissions.into()),
                            ..Default::default()
                        },
                        vec![],
                    )
                    .await?;
            } else {
                return Err(Error::InvalidOperation);
            }
        }
        _ => return Err(Error::InvalidOperation),
    }
    Ok(Json(channel))
}
