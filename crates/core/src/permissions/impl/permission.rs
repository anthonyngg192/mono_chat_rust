use std::{borrow::Cow, collections::HashSet};

use crate::{
    database::Database,
    models::{channel::ChannelType, user::RelationshipStatus, Channel, Member, Server, User},
    permissions::{
        defn::{
            ChannelPermission, Override, PermissionQuery, Permissions, Perms, UserPermission,
            ALLOW_IN_TIMEOUT, DEFAULT_PERMISSION_DIRECT_MESSAGE, DEFAULT_PERMISSION_SAVED_MESSAGES,
            DEFAULT_PERMISSION_VIEW_ONLY,
        },
        PermissionCalculator,
    },
    util::result::Result,
};

use super::{super::ChannelPermission::GrantAllSafe, PermissionValue};
#[derive(Clone)]
pub struct DatabasePermissionQuery<'a> {
    #[allow(dead_code)]
    database: &'a Database,

    perspective: &'a User,
    user: Option<Cow<'a, User>>,
    channel: Option<Cow<'a, Channel>>,
    server: Option<Cow<'a, Server>>,
    member: Option<Cow<'a, Member>>,

    cached_user_permission: Option<PermissionValue>,
    cached_mutual_connection: Option<bool>,
    cached_permission: Option<u64>,
}

#[async_trait]
impl PermissionQuery for DatabasePermissionQuery<'_> {
    async fn are_we_privileged(&mut self) -> bool {
        self.perspective.privileged
    }

    async fn are_we_a_bot(&mut self) -> bool {
        self.perspective.bot.is_some()
    }

    async fn are_the_users_same(&mut self) -> bool {
        if let Some(other_user) = &self.user {
            self.perspective.id == other_user.id
        } else {
            false
        }
    }

    async fn user_relationship(&mut self) -> RelationshipStatus {
        if let Some(other_user) = &self.user {
            if self.perspective.id == other_user.id {
                return RelationshipStatus::User;
            }

            let relations = &self.perspective.relations;
            for entry in relations {
                if entry.user_id == other_user.id {
                    return match entry.status {
                        RelationshipStatus::None => RelationshipStatus::None,
                        RelationshipStatus::User => RelationshipStatus::User,
                        RelationshipStatus::Friend => RelationshipStatus::Friend,
                        RelationshipStatus::Outgoing => RelationshipStatus::Outgoing,
                        RelationshipStatus::Incoming => RelationshipStatus::Incoming,
                        RelationshipStatus::Blocked => RelationshipStatus::Blocked,
                        RelationshipStatus::BlockedOther => RelationshipStatus::BlockedOther,
                    };
                }
            }
        }

        RelationshipStatus::None
    }

    async fn user_is_bot(&mut self) -> bool {
        if let Some(other_user) = &self.user {
            other_user.bot.is_some()
        } else {
            false
        }
    }

    async fn have_mutual_connection(&mut self) -> bool {
        if let Some(value) = self.cached_mutual_connection {
            value
        } else if let Some(user) = &self.user {
            let value = self
                .perspective
                .has_mutual_connection(self.database, &user.id)
                .await
                .unwrap_or_default();

            self.cached_mutual_connection = Some(value);
            matches!(value, true)
        } else {
            false
        }
    }

    async fn are_we_server_owner(&mut self) -> bool {
        if let Some(server) = &self.server {
            server.owner == self.perspective.id
        } else {
            false
        }
    }

    async fn are_we_a_member(&mut self) -> bool {
        if let Some(server) = &self.server {
            if self.member.is_some() {
                true
            } else if let Ok(member) = self
                .database
                .fetch_member(&server.id, &self.perspective.id)
                .await
            {
                self.member = Some(Cow::Owned(member));
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    async fn get_default_server_permissions(&mut self) -> u64 {
        if let Some(server) = &self.server {
            server.default_permissions as u64
        } else {
            0
        }
    }

    async fn get_our_server_role_overrides(&mut self) -> Vec<Override> {
        if let Some(server) = &self.server {
            let member_roles = self
                .member
                .as_ref()
                .map(|member| member.roles.clone())
                .unwrap_or_default();

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
            roles.into_iter().map(|(_, v)| v).collect()
        } else {
            vec![]
        }
    }

    async fn are_we_timed_out(&mut self) -> bool {
        if let Some(member) = &self.member {
            member.in_timeout()
        } else {
            false
        }
    }

    async fn get_channel_type(&mut self) -> ChannelType {
        if let Some(channel) = &self.channel {
            match channel {
                Cow::Borrowed(Channel::DirectMessage { .. })
                | Cow::Owned(Channel::DirectMessage { .. }) => ChannelType::DirectMessage,
                Cow::Borrowed(Channel::Group { .. }) | Cow::Owned(Channel::Group { .. }) => {
                    ChannelType::Group
                }
                Cow::Borrowed(Channel::SavedMessages { .. })
                | Cow::Owned(Channel::SavedMessages { .. }) => ChannelType::SavedMessages,
                Cow::Borrowed(Channel::TextChannel { .. })
                | Cow::Owned(Channel::TextChannel { .. })
                | Cow::Borrowed(Channel::VoiceChannel { .. })
                | Cow::Owned(Channel::VoiceChannel { .. }) => ChannelType::ServerChannel,
            }
        } else {
            ChannelType::Unknown
        }
    }

    async fn get_default_channel_permissions(&mut self) -> Override {
        if let Some(channel) = &self.channel {
            match channel {
                Cow::Borrowed(Channel::Group { permissions, .. })
                | Cow::Owned(Channel::Group { permissions, .. }) => Override {
                    allow: permissions.unwrap_or_default() as u64,
                    deny: 0,
                },
                Cow::Borrowed(Channel::TextChannel {
                    default_permissions,
                    ..
                })
                | Cow::Owned(Channel::TextChannel {
                    default_permissions,
                    ..
                })
                | Cow::Borrowed(Channel::VoiceChannel {
                    default_permissions,
                    ..
                })
                | Cow::Owned(Channel::VoiceChannel {
                    default_permissions,
                    ..
                }) => default_permissions.unwrap_or_default().into(),
                _ => Default::default(),
            }
        } else {
            Default::default()
        }
    }

    async fn get_our_channel_role_overrides(&mut self) -> Vec<Override> {
        if let Some(channel) = &self.channel {
            match channel {
                Cow::Borrowed(Channel::TextChannel {
                    role_permissions, ..
                })
                | Cow::Owned(Channel::TextChannel {
                    role_permissions, ..
                })
                | Cow::Borrowed(Channel::VoiceChannel {
                    role_permissions, ..
                })
                | Cow::Owned(Channel::VoiceChannel {
                    role_permissions, ..
                }) => {
                    if let Some(server) = &self.server {
                        let member_roles = self
                            .member
                            .as_ref()
                            .map(|member| member.roles.clone())
                            .unwrap_or_default();

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
                        roles.into_iter().map(|(_, v)| v).collect()
                    } else {
                        vec![]
                    }
                }
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    async fn do_we_own_the_channel(&mut self) -> bool {
        if let Some(channel) = &self.channel {
            match channel {
                Cow::Borrowed(Channel::Group { owner, .. })
                | Cow::Owned(Channel::Group { owner, .. }) => owner == &self.perspective.id,
                Cow::Borrowed(Channel::SavedMessages { user, .. })
                | Cow::Owned(Channel::SavedMessages { user, .. }) => user == &self.perspective.id,
                _ => false,
            }
        } else {
            false
        }
    }

    async fn are_we_part_of_the_channel(&mut self) -> bool {
        if let Some(channel) = &self.channel {
            match channel {
                Cow::Borrowed(Channel::DirectMessage { recipients, .. })
                | Cow::Owned(Channel::DirectMessage { recipients, .. })
                | Cow::Borrowed(Channel::Group { recipients, .. })
                | Cow::Owned(Channel::Group { recipients, .. }) => {
                    recipients.contains(&self.perspective.id)
                }
                _ => false,
            }
        } else {
            false
        }
    }

    async fn set_recipient_as_user(&mut self) {
        if let Some(channel) = &self.channel {
            match channel {
                Cow::Borrowed(Channel::DirectMessage { recipients, .. })
                | Cow::Owned(Channel::DirectMessage { recipients, .. }) => {
                    let recipient_id = recipients
                        .iter()
                        .find(|recipient| recipient != &&self.perspective.id)
                        .expect("Missing recipient for DM");

                    if let Ok(user) = self.database.fetch_user(recipient_id).await {
                        self.user.replace(Cow::Owned(user));
                    }
                }
                _ => unimplemented!(),
            }
        }
    }

    async fn set_server_from_channel(&mut self) {
        if let Some(channel) = &self.channel {
            match channel {
                Cow::Borrowed(Channel::TextChannel { server, .. })
                | Cow::Owned(Channel::TextChannel { server, .. })
                | Cow::Borrowed(Channel::VoiceChannel { server, .. })
                | Cow::Owned(Channel::VoiceChannel { server, .. }) => {
                    if let Some(known_server) =
                        if let Some(Cow::Borrowed(known_server)) = self.server {
                            Some(known_server)
                        } else if let Some(Cow::Owned(ref known_server)) = self.server {
                            Some(known_server)
                        } else {
                            None
                        }
                    {
                        if server == &known_server.id {
                            // Already cached, return early.
                            return;
                        }
                    }

                    if let Ok(server) = self.database.fetch_server(server).await {
                        self.server.replace(Cow::Owned(server));
                    }
                }
                _ => unimplemented!(),
            }
        }
    }
}

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

impl<'a> DatabasePermissionQuery<'a> {
    pub fn new(database: &'a Database, perspective: &'a User) -> DatabasePermissionQuery<'a> {
        DatabasePermissionQuery {
            database,
            perspective,
            user: None,
            channel: None,
            server: None,
            member: None,

            cached_mutual_connection: None,
            cached_user_permission: None,
            cached_permission: None,
        }
    }

    pub async fn calc_user(mut self) -> DatabasePermissionQuery<'a> {
        if self.cached_user_permission.is_some() {
            return self;
        }

        if self.user.is_none() {
            panic!("Expected `PermissionCalculator.user to exist.");
        }

        DatabasePermissionQuery {
            cached_user_permission: Some(calculate_user_permissions(&mut self).await),
            ..self
        }
    }

    pub async fn calc(self) -> DatabasePermissionQuery<'a> {
        if self.cached_permission.is_some() {
            return self;
        }

        self
    }

    pub fn user(self, user: &'a User) -> DatabasePermissionQuery {
        DatabasePermissionQuery {
            user: Some(Cow::Borrowed(user)),
            ..self
        }
    }

    pub fn channel(self, channel: &'a Channel) -> DatabasePermissionQuery {
        DatabasePermissionQuery {
            channel: Some(Cow::Borrowed(channel)),
            ..self
        }
    }

    pub fn server(self, server: &'a Server) -> DatabasePermissionQuery {
        DatabasePermissionQuery {
            server: Some(Cow::Borrowed(server)),
            ..self
        }
    }

    pub fn member(self, member: &'a Member) -> DatabasePermissionQuery {
        DatabasePermissionQuery {
            member: Some(Cow::Borrowed(member)),
            ..self
        }
    }
}

pub async fn calculate_server_permission(
    data: &mut PermissionCalculator<'_>,
    db: &Database,
) -> Result<PermissionValue> {
    let server = data.server.get().unwrap();

    if data.perspective.id == server.owner {
        return Ok((ChannelPermission::GrantAllSafe as u64).into());
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

        for (_, v) in roles {
            permissions.apply(v);
        }
    }

    if member.in_timeout() {
        permissions.restrict(*ALLOW_IN_TIMEOUT);
    }

    Ok(permissions)
}

pub async fn calculate_channel_permission(
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
                (ChannelPermission::GrantAllSafe as u64).into()
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
                    return Ok((ChannelPermission::GrantAllSafe as u64).into());
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
                (ChannelPermission::GrantAllSafe as u64).into()
            }
        }
    };
    Ok(value)
}

pub async fn calculate_user_permissions<P: PermissionQuery>(query: &mut P) -> PermissionValue {
    if query.are_we_privileged().await {
        return u64::MAX.into();
    }

    if query.are_the_users_same().await {
        return u64::MAX.into();
    }

    let mut permissions = 0_u64;
    match query.user_relationship().await {
        RelationshipStatus::Friend => return u64::MAX.into(),
        RelationshipStatus::Blocked | RelationshipStatus::BlockedOther => {
            return (UserPermission::Access as u64).into()
        }
        RelationshipStatus::Incoming | RelationshipStatus::Outgoing => {
            permissions = UserPermission::Access as u64;
        }
        _ => {}
    }

    if query.have_mutual_connection().await {
        permissions = UserPermission::Access as u64 + UserPermission::ViewProfile as u64;

        if query.user_is_bot().await || query.are_we_a_bot().await {
            permissions += UserPermission::SendMessage as u64;
        }

        permissions.into()
    } else {
        permissions.into()
    }
}

pub async fn calculate_server_permissions<P: PermissionQuery>(query: &mut P) -> PermissionValue {
    if query.are_we_privileged().await || query.are_we_server_owner().await {
        return ChannelPermission::GrantAllSafe.into();
    }

    if !query.are_we_a_member().await {
        return 0_u64.into();
    }

    let mut permissions: PermissionValue = query.get_default_server_permissions().await.into();

    for role_override in query.get_our_server_role_overrides().await {
        permissions.apply(role_override);
    }

    if query.are_we_timed_out().await {
        permissions.restrict(*ALLOW_IN_TIMEOUT);
    }

    permissions
}

pub async fn calculate_channel_permissions<P: PermissionQuery>(query: &mut P) -> PermissionValue {
    if query.are_we_privileged().await {
        return ChannelPermission::GrantAllSafe.into();
    }

    match query.get_channel_type().await {
        ChannelType::SavedMessages => {
            if query.do_we_own_the_channel().await {
                DEFAULT_PERMISSION_SAVED_MESSAGES.into()
            } else {
                0_u64.into()
            }
        }
        ChannelType::DirectMessage => {
            if query.are_we_part_of_the_channel().await {
                query.set_recipient_as_user().await;

                let permissions = calculate_user_permissions(query).await;
                if permissions.has_user_permission(UserPermission::SendMessage) {
                    (*DEFAULT_PERMISSION_DIRECT_MESSAGE).into()
                } else {
                    (*DEFAULT_PERMISSION_VIEW_ONLY).into()
                }
            } else {
                0_u64.into()
            }
        }
        ChannelType::Group => {
            if query.do_we_own_the_channel().await {
                ChannelPermission::GrantAllSafe.into()
            } else if query.are_we_part_of_the_channel().await {
                (*DEFAULT_PERMISSION_VIEW_ONLY
                    | query.get_default_channel_permissions().await.allow)
                    .into()
            } else {
                0_u64.into()
            }
        }
        ChannelType::ServerChannel => {
            query.set_server_from_channel().await;

            if query.are_we_server_owner().await {
                ChannelPermission::GrantAllSafe.into()
            } else if query.are_we_a_member().await {
                let mut permissions = calculate_server_permissions(query).await;
                permissions.apply(query.get_default_channel_permissions().await);

                for role_override in query.get_our_channel_role_overrides().await {
                    permissions.apply(role_override);
                }

                if query.are_we_timed_out().await {
                    permissions.restrict(*ALLOW_IN_TIMEOUT);
                }

                if !permissions.has_channel_permission(ChannelPermission::ViewChannel) {
                    permissions.revoke_all();
                }

                permissions
            } else {
                0_u64.into()
            }
        }
        ChannelType::Unknown => 0_u64.into(),
    }
}
