use std::collections::HashSet;

use chat_core::{
    models::{
        server_member::{FieldsMember, PartialMember},
        File, Member, User,
    },
    permissions::defn::ChannelPermission,
    perms, Db, Error, Ref, Result, Timestamp,
};

use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataMemberEdit {
    #[validate(length(min = 1, max = 32))]
    nickname: Option<String>,

    avatar: Option<String>,
    roles: Option<Vec<String>>,
    timeout: Option<Timestamp>,

    #[validate(length(min = 1))]
    remove: Option<Vec<FieldsMember>>,
}

#[openapi(tag = "Server Members")]
#[patch("/<server>/members/<target>", data = "<data>")]
pub async fn req(
    db: &Db,
    user: User,
    server: Ref,
    target: Ref,
    data: Json<DataMemberEdit>,
) -> Result<Json<Member>> {
    let data = data.into_inner();
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    let mut server = server.as_server(db).await?;
    let mut member = target.as_member(db, &server.id).await?;
    let mut permissions = perms(&user).server(&server);

    let mut required = vec![];

    if data.nickname.is_some()
        || data
            .remove
            .as_ref()
            .map(|x| x.contains(&FieldsMember::Nickname))
            .unwrap_or_default()
    {
        if user.id == member.id.user {
            required.push(ChannelPermission::ChangeNickname);
        } else {
            required.push(ChannelPermission::ManageNicknames);
        }
    }

    if data.avatar.is_some()
        || data
            .remove
            .as_ref()
            .map(|x| x.contains(&FieldsMember::Avatar))
            .unwrap_or_default()
    {
        if user.id == member.id.user {
            required.push(ChannelPermission::ChangeAvatar);
        } else {
            return Err(Error::InvalidOperation);
        }
    }

    if data.roles.is_some()
        || data
            .remove
            .as_ref()
            .map(|x| x.contains(&FieldsMember::Roles))
            .unwrap_or_default()
    {
        required.push(ChannelPermission::AssignRoles);
    }

    if data.timeout.is_some()
        || data
            .remove
            .as_ref()
            .map(|x| x.contains(&FieldsMember::Timeout))
            .unwrap_or_default()
    {
        required.push(ChannelPermission::TimeoutMembers);
    }

    for permission in required {
        permissions.throw_permission(db, permission).await?;
    }

    let our_ranking = permissions.get_member_rank().unwrap_or(i64::MIN);

    // Check that we have ChannelPermissions to act against this member
    if member.id.user != user.id
        && member.get_ranking(permissions.server.get().unwrap()) <= our_ranking
    {
        return Err(Error::NotElevated);
    }

    // Check ChannelPermissions against roles in diff
    if let Some(roles) = &data.roles {
        let current_roles = member.roles.iter().collect::<HashSet<&String>>();

        let new_roles = roles.iter().collect::<HashSet<&String>>();
        let added_roles: Vec<&&String> = new_roles.difference(&current_roles).collect();

        for role_id in added_roles {
            if let Some(role) = server.roles.remove(*role_id) {
                if role.rank <= our_ranking {
                    return Err(Error::NotElevated);
                }
            } else {
                return Err(Error::InvalidRole);
            }
        }
    }

    // Apply edits to the member object
    let DataMemberEdit {
        nickname,
        avatar,
        roles,
        timeout,
        remove,
    } = data;

    let mut partial = PartialMember {
        nickname,
        roles,
        timeout,
        ..Default::default()
    };

    // 1. Remove fields from object
    if let Some(fields) = &remove {
        if fields.contains(&FieldsMember::Avatar) {
            if let Some(avatar) = &member.avatar {
                db.mark_attachment_as_deleted(&avatar.id).await?;
            }
        }
    }

    // 2. Apply new avatar
    if let Some(avatar) = avatar {
        partial.avatar = Some(File::use_avatar(db, &avatar, &user.id).await?);
    }

    member
        .update(db, partial, remove.unwrap_or_default())
        .await?;

    Ok(Json(member))
}
