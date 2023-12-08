use crate::{
    database::Database,
    events::client::EventV1,
    models::{
        message::SystemMessage,
        server_member::{FieldsMember, MemberCompositeKey, PartialMember},
        Channel, Member, Server, User,
    },
    permissions::r#impl::permission::{calculate_channel_permissions, DatabasePermissionQuery},
    ChannelPermission, Error, Result,
};
use iso8601_timestamp::Timestamp;

impl Default for Member {
    fn default() -> Self {
        Self {
            id: Default::default(),
            joined_at: Timestamp::now_utc(),
            nickname: None,
            avatar: None,
            roles: vec![],
            timeout: None,
        }
    }
}

impl Member {
    pub fn remove(&mut self, field: &FieldsMember) {
        match field {
            FieldsMember::Avatar => self.avatar = None,
            FieldsMember::Nickname => self.nickname = None,
            FieldsMember::Roles => self.roles.clear(),
            FieldsMember::Timeout => self.timeout = None,
        }
    }

    pub fn in_timeout(&self) -> bool {
        if let Some(timeout) = self.timeout {
            *timeout > *Timestamp::now_utc()
        } else {
            false
        }
    }

    pub async fn update<'a>(
        &mut self,
        db: &Database,
        partial: PartialMember,
        remove: Vec<FieldsMember>,
    ) -> Result<()> {
        for field in &remove {
            self.remove(field);
        }

        self.apply_options(partial.clone());

        db.update_member(&self.id, &partial, remove.clone()).await?;

        EventV1::ServerMemberUpdate {
            id: self.id.clone(),
            data: partial,
            clear: remove,
        }
        .p(self.id.server.clone())
        .await;

        Ok(())
    }

    pub fn get_ranking(&self, server: &Server) -> i64 {
        let mut value = i64::MAX;
        for role in &self.roles {
            if let Some(role) = server.roles.get(role) {
                if role.rank < value {
                    value = role.rank;
                }
            }
        }

        value
    }

    pub async fn create(
        db: &Database,
        server: &Server,
        user: &User,
        channels: Option<Vec<Channel>>,
    ) -> Result<Vec<Channel>> {
        if db.fetch_ban(&server.id, &user.id).await.is_ok() {
            return Err(Error::Banned);
        }

        if db.fetch_member(&server.id, &user.id).await.is_ok() {
            return Err(Error::AlreadyInServer);
        }

        let member = Member {
            id: MemberCompositeKey {
                server: server.id.to_string(),
                user: user.id.to_string(),
            },
            joined_at: Timestamp::now_utc(),
            ..Default::default()
        };

        db.insert_member(&member).await?;

        let should_fetch = channels.is_none();
        let mut channels = channels.unwrap_or_default();

        if should_fetch {
            let query = DatabasePermissionQuery::new(db, user).server(server);
            let existing_channels = db.fetch_channels(&server.channels).await?;

            for channel in existing_channels {
                let mut channel_query = query.clone().channel(&channel);

                if calculate_channel_permissions(&mut channel_query)
                    .await
                    .has_channel_permission(ChannelPermission::ViewChannel)
                {
                    channels.push(channel);
                }
            }
        }

        // let emojis = db.fetch_emoji_by_parent_id(&server.id).await?;

        EventV1::ServerMemberJoin {
            id: server.id.clone(),
            user: user.id.clone(),
        }
        .p(server.id.clone())
        .await;

        EventV1::ServerCreate {
            id: server.id.clone(),
            server: server.clone(),
            channels: channels.clone(),
        }
        .private(user.id.clone())
        .await;

        if let Some(id) = server
            .system_messages
            .as_ref()
            .and_then(|x| x.user_joined.as_ref())
        {
            SystemMessage::UserJoined {
                id: user.id.clone(),
            }
            .into_message(id.to_string())
            .send_without_notifications(db, false, false)
            .await
            .ok();
        }

        Ok(channels)
    }
}
