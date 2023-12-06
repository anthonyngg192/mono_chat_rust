use std::collections::HashSet;

use crate::{
    database::Database,
    models::Channel,
    permissions::{
        defn::{
            Override, Permission, PermissionValue, Permissions, Perms, ALLOW_IN_TIMEOUT,
            DEFAULT_PERMISSION_DIRECT_MESSAGE, DEFAULT_PERMISSION_SAVED_MESSAGES,
            DEFAULT_PERMISSION_VIEW_ONLY,
        },
        PermissionCalculator,
    },
    util::result::Result,
};

use super::super::Permission::GrantAllSafe;

impl PermissionCalculator<'_> {
    pub async fn calc(&mut self, db: &Database) -> Result<Perms> {
        if self.perspective.privileged {
            return Ok(Permissions([GrantAllSafe as u64]));
        }

        let value = if self.channel.has() {
            calculate_channel_permission(self, db).await?
        } else if self.server.has() {
            calculate_server_permission(self, db).await?
        } else {
            panic!("Expected PermissionCalculator.(user|server) to exists");
        }
        .into();

        self.cached_permission = Some(value);
        Ok(Permissions([value]))
    }
}

async fn calculate_server_permission(
    data: &mut PermissionCalculator<'_>,
    db: &Database,
) -> Result<PermissionValue> {
    let server = data.server.get().unwrap();

    if data.perspective.id == server.owner {
        return Ok((Permission::GrantAllSafe as u64).into());
    }

    if !data.member.has() {
        data.member
            .set(db.fetch_member(&server.id, &data.perspective.id).await?);
    }

    let member = data.member.get().expect("Member should be present by now.");

    let mut permissions: PermissionValue = server.default_permissions.into();

    let member_roles: HashSet<&String> = member.roles.iter().collect();

    if !member_roles.is_empty() {
        let mut roles = server
            .roles
            .iter()
            .filter(|(id, _)| member_roles.contains(id))
            .map(|(_, role)| {
                let v: Override = role.permissions.into();
                (role.rank, v)
            })
            .collect::<Vec<(i64, Override)>>();

        roles.sort_by(|a, b| b.0.cmp(&a.0));

        // 5. Apply allows and denies from roles.
        for (_, v) in roles {
            permissions.apply(v);
        }
    }

    // 5. Revoke permissions if member is timed out.
    if member.in_timeout() {
        permissions.restrict(*ALLOW_IN_TIMEOUT);
    }

    Ok(permissions)
}

async fn calculate_channel_permission(
    data: &mut PermissionCalculator<'_>,
    db: &Database,
) -> Result<PermissionValue> {
    let server_id = match data.channel.get().unwrap() {
        Channel::TextChannel { server, .. } | Channel::VoiceChannel { server, .. } => Some(server),
        _ => None,
    };

    let mut permissions = if let Some(server) = server_id {
        if !data.server.has() {
            data.server.set(db.fetch_server(server).await?);
        }

        calculate_server_permission(data, db).await?
    } else {
        0_u64.into()
    };

    // Borrow the channel now and continue as normal.
    let channel = data.channel.get().unwrap();

    let value: PermissionValue = match channel {
        Channel::SavedMessages { user, .. } => {
            if user == &data.perspective.id {
                DEFAULT_PERMISSION_SAVED_MESSAGES.into()
            } else {
                0_u64.into()
            }
        }
        Channel::DirectMessage { recipients, .. } => {
            // 2. Ensure we are a recipient.
            if recipients.contains(&data.perspective.id) {
                // 3. Fetch user.
                let other_user = recipients
                    .iter()
                    .find(|x| x != &&data.perspective.id)
                    .unwrap();

                let user = db.fetch_user(other_user).await?;
                data.user.set(user);

                // 4. Calculate user permissions.
                let perms = data.calc_user(db).await;

                // 5. Check if the user can send messages.
                if perms.get_send_message() {
                    (*DEFAULT_PERMISSION_DIRECT_MESSAGE).into()
                } else {
                    (*DEFAULT_PERMISSION_VIEW_ONLY).into()
                }
            } else {
                0_u64.into()
            }
        }
        Channel::Group {
            owner,
            permissions,
            recipients,
            ..
        } => {
            // 2. Check if user is owner.
            if &data.perspective.id == owner {
                (Permission::GrantAllSafe as u64).into()
            } else {
                // 3. Check that we are actually in the group.
                if recipients.contains(&data.perspective.id) {
                    // 4. Pull out group permissions.
                    permissions
                        .map(|x| x as u64)
                        .unwrap_or(*DEFAULT_PERMISSION_DIRECT_MESSAGE)
                        .into()
                } else {
                    0_u64.into()
                }
            }
        }
        Channel::TextChannel {
            default_permissions,
            role_permissions,
            ..
        }
        | Channel::VoiceChannel {
            default_permissions,
            role_permissions,
            ..
        } => {
            if let Some(member) = data.member.get() {
                let server = data.server.get().unwrap();

                if server.owner == member.id.user {
                    return Ok((Permission::GrantAllSafe as u64).into());
                }

                if let Some(default) = default_permissions {
                    permissions.apply((*default).into());
                }

                let member_roles: Vec<_> = member.roles.iter().collect();

                if !member_roles.is_empty() {
                    let mut roles = role_permissions
                        .iter()
                        .filter(|(id, _)| member_roles.contains(id))
                        .filter_map(|(id, permission)| {
                            server.roles.get(id).map(|role| {
                                let v: Override = (*permission).into();
                                (role.rank, v)
                            })
                        })
                        .collect::<Vec<(i64, Override)>>();

                    roles.sort_by(|a, b| b.0.cmp(&a.0));

                    // 5. Apply allows and denies from roles.
                    for (_, v) in roles {
                        permissions.apply(v);
                    }
                }

                // 5. Revoke permissions if member is timed out.
                if member.in_timeout() {
                    permissions.restrict(*ALLOW_IN_TIMEOUT);
                }

                permissions
            } else {
                (Permission::GrantAllSafe as u64).into()
            }
        }
    };
    Ok(value)
}
