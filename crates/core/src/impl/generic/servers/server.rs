use std::collections::{HashMap, HashSet};

use iso8601_timestamp::Timestamp;
use ulid::Ulid;

use crate::{
    database::Database,
    events::client::EventV1,
    models::{
        channel::{DataCreateServerChannel, LegacyServerChannelType},
        message::SystemMessage,
        server::{
            DataCreateServer, FieldsRole, FieldsServer, PartialRole, PartialServer, Role,
            SystemMessageChannels,
        },
        server_member::{MemberCompositeKey, RemovalIntention},
        Channel, Member, Server, ServerBan, User,
    },
    permissions::{
        defn::{ChannelPermission, OverrideField},
        perms,
    },
    Error, Result, DEFAULT_PERMISSION_SERVER,
};

impl Server {
    pub fn remove(&mut self, field: &FieldsServer) {
        match field {
            FieldsServer::Description => self.description = None,
            FieldsServer::Categories => self.categories = None,
            FieldsServer::SystemMessages => self.system_messages = None,
            FieldsServer::Icon => self.icon = None,
            FieldsServer::Banner => self.banner = None,
        }
    }

    pub async fn create(
        &self,
        db: &Database,
        data: DataCreateServer,
        owner: &User,
        create_default_channels: bool,
    ) -> Result<(Server, Vec<Channel>)> {
        let mut server = Server {
            id: ulid::Ulid::new().to_string(),
            owner: owner.id.to_string(),
            name: data.name,
            description: data.description,
            channels: vec![],
            nsfw: data.nsfw.unwrap_or(false),
            default_permissions: *DEFAULT_PERMISSION_SERVER as i64,

            analytics: false,
            banner: None,
            categories: None,
            discoverable: false,
            flags: None,
            icon: None,
            roles: HashMap::new(),
            system_messages: None,
        };

        let channels: Vec<Channel> = if create_default_channels {
            vec![
                Channel::create_server_channel(
                    db,
                    &mut server,
                    DataCreateServerChannel {
                        channel_type: LegacyServerChannelType::Text,
                        name: "General".to_string(),
                        description: None,
                    },
                    false,
                )
                .await?,
            ]
        } else {
            vec![]
        };

        server.channels = channels.iter().map(|c| c.id().to_string()).collect();
        db.insert_server(&server).await?;
        Ok((server, channels))
    }

    pub async fn update<'a>(
        &mut self,
        db: &Database,
        partial: PartialServer,
        remove: Vec<FieldsServer>,
    ) -> Result<()> {
        for field in &remove {
            self.remove(field);
        }

        self.apply_options(partial.clone());

        db.update_server(&self.id, &partial, remove.clone()).await?;

        EventV1::ServerUpdate {
            id: self.id.clone(),
            data: partial,
            clear: remove,
        }
        .p(self.id.clone())
        .await;

        Ok(())
    }

    pub async fn set_role_permission(
        &mut self,
        db: &Database,
        role_id: &str,
        permissions: OverrideField,
    ) -> Result<()> {
        if let Some(role) = self.roles.get_mut(role_id) {
            role.update(
                db,
                &self.id,
                role_id,
                PartialRole {
                    permissions: Some(permissions),
                    ..Default::default()
                },
                vec![],
            )
            .await?;

            Ok(())
        } else {
            Err(Error::NotFound)
        }
    }

    pub async fn delete(self, db: &Database) -> Result<()> {
        EventV1::ServerDelete {
            id: self.id.clone(),
        }
        .p(self.id.clone())
        .await;

        db.delete_server(&self).await
    }

    pub async fn create_member(
        &self,
        db: &Database,
        user: User,
        channels: Option<Vec<Channel>>,
    ) -> Result<Vec<Channel>> {
        if db.fetch_ban(&self.id, &user.id).await.is_ok() {
            return Err(Error::Banned);
        }

        let member = Member {
            id: MemberCompositeKey {
                server: self.id.clone(),
                user: user.id.clone(),
            },
            joined_at: Timestamp::now_utc(),
            nickname: None,
            avatar: None,
            roles: vec![],
            timeout: None,
        };

        db.insert_member(&member).await?;

        let should_fetch = channels.is_none();
        let mut channels = channels.unwrap_or_default();

        if should_fetch {
            let perm = perms(&user).server(self).member(&member);
            let existing_channels = db.fetch_channels(&self.channels).await?;
            for channel in existing_channels {
                if perm
                    .clone()
                    .channel(&channel)
                    .has_permission(db, ChannelPermission::ViewChannel)
                    .await?
                {
                    channels.push(channel);
                }
            }
        }

        EventV1::ServerMemberJoin {
            id: self.id.clone(),
            user: user.id.clone(),
        }
        .p(self.id.clone())
        .await;

        EventV1::ServerCreate {
            id: self.id.clone(),
            server: self.clone(),
            channels: channels.clone(),
        }
        .private(user.id.clone())
        .await;

        if let Some(id) = self
            .system_messages
            .as_ref()
            .and_then(|x| x.user_joined.as_ref())
        {
            SystemMessage::UserJoined {
                id: user.id.clone(),
            }
            .into_message(id.to_string())
            .create_no_web_push(db, id, false)
            .await
            .ok();
        }

        Ok(channels)
    }

    pub async fn remove_member(
        &self,
        db: &Database,
        member: Member,
        intention: RemovalIntention,
        silent: bool,
    ) -> Result<()> {
        db.delete_member(&member.id).await?;

        EventV1::ServerMemberLeave {
            id: self.id.to_string(),
            user: member.id.user.clone(),
        }
        .p(member.id.server)
        .await;

        if !silent {
            if let Some(id) = self.system_messages.as_ref().and_then(|x| match intention {
                RemovalIntention::Leave => x.user_left.as_ref(),
                RemovalIntention::Kick => x.user_kicked.as_ref(),
                RemovalIntention::Ban => x.user_banned.as_ref(),
            }) {
                match intention {
                    RemovalIntention::Leave => SystemMessage::UserLeft { id: member.id.user },
                    RemovalIntention::Kick => SystemMessage::UserKicked { id: member.id.user },
                    RemovalIntention::Ban => SystemMessage::UserBanned { id: member.id.user },
                }
                .into_message(id.to_string())
                .create_no_web_push(db, id, false)
                .await
                .ok();
            }
        }
        Ok(())
    }

    pub async fn ban_user(
        self,
        db: &Database,
        id: MemberCompositeKey,
        reason: Option<String>,
    ) -> Result<ServerBan> {
        let ban = ServerBan { id, reason };
        db.insert_ban(&ban).await?;
        Ok(ban)
    }

    pub async fn ban_member(
        self,
        db: &Database,
        member: Member,
        reason: Option<String>,
    ) -> Result<ServerBan> {
        self.remove_member(db, member.clone(), RemovalIntention::Ban, false)
            .await?;

        self.ban_user(db, member.id, reason).await
    }
}

impl Role {
    pub fn remove(&mut self, field: &FieldsRole) {
        match field {
            FieldsRole::Colour => self.colour = None,
        }
    }

    pub fn into_optional(self) -> PartialRole {
        PartialRole {
            name: Some(self.name),
            permissions: Some(self.permissions),
            colour: self.colour,
            hoist: Some(self.hoist),
            rank: Some(self.rank),
        }
    }

    pub async fn create(&self, db: &Database, server_id: &str) -> Result<String> {
        let role_id = Ulid::new().to_string();
        db.insert_role(server_id, &role_id, self).await?;

        EventV1::ServerRoleUpdate {
            id: server_id.to_string(),
            role_id: role_id.to_string(),
            data: self.clone().into_optional(),
            clear: vec![],
        }
        .p(server_id.to_string())
        .await;
        Ok(role_id)
    }

    pub async fn update<'a>(
        &mut self,
        db: &Database,
        server_id: &str,
        role_id: &str,
        partial: PartialRole,
        remove: Vec<FieldsRole>,
    ) -> Result<()> {
        for field in &remove {
            self.remove(field);
        }

        self.apply_options(partial.clone());

        db.update_role(server_id, role_id, &partial, remove.clone())
            .await?;

        EventV1::ServerRoleUpdate {
            id: server_id.to_string(),
            role_id: role_id.to_string(),
            data: partial,
            clear: vec![],
        }
        .p(server_id.to_string())
        .await;

        Ok(())
    }

    /// Delete a role
    pub async fn delete(self, db: &Database, server_id: &str, role_id: &str) -> Result<()> {
        EventV1::ServerRoleDelete {
            id: server_id.to_string(),
            role_id: role_id.to_string(),
        }
        .p(server_id.to_string())
        .await;

        db.delete_role(server_id, role_id).await
    }
}

impl SystemMessageChannels {
    pub fn into_channel_ids(self) -> HashSet<String> {
        let mut ids = HashSet::new();

        if let Some(id) = self.user_joined {
            ids.insert(id);
        }

        if let Some(id) = self.user_left {
            ids.insert(id);
        }

        if let Some(id) = self.user_kicked {
            ids.insert(id);
        }

        if let Some(id) = self.user_banned {
            ids.insert(id);
        }

        ids
    }
}
