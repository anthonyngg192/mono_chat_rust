use std::collections::HashMap;

use crate::{
    database::Database,
    events::client::EventV1,
    models::{
        channel::{
            DataCreateServerChannel, FieldsChannel, LegacyServerChannelType, PartialChannel,
        },
        message::SystemMessage,
        server::PartialServer,
        Channel, Server,
    },
    permissions::defn::OverrideField,
    tasks::ack::AckEvent,
    Error, Result,
};

impl Channel {
    pub fn id(&'_ self) -> &'_ str {
        match self {
            Channel::DirectMessage { id, .. }
            | Channel::Group { id, .. }
            | Channel::SavedMessages { id, .. }
            | Channel::TextChannel { id, .. }
            | Channel::VoiceChannel { id, .. } => id,
        }
    }

    pub fn as_id(self) -> String {
        match self {
            Channel::DirectMessage { id, .. }
            | Channel::Group { id, .. }
            | Channel::SavedMessages { id, .. }
            | Channel::TextChannel { id, .. }
            | Channel::VoiceChannel { id, .. } => id,
        }
    }

    pub fn is_direct_dm(&self) -> bool {
        matches!(self, Channel::DirectMessage { .. })
    }

    pub async fn create(&self, db: &Database) -> Result<()> {
        db.insert_channel(self).await?;

        Ok(())
    }

    pub fn remove(&mut self, field: &FieldsChannel) {
        match field {
            FieldsChannel::Description => match self {
                Self::Group { description, .. }
                | Self::TextChannel { description, .. }
                | Self::VoiceChannel { description, .. } => {
                    description.take();
                }
                _ => {}
            },
            FieldsChannel::Icon => match self {
                Self::Group { icon, .. }
                | Self::TextChannel { icon, .. }
                | Self::VoiceChannel { icon, .. } => {
                    icon.take();
                }
                _ => {}
            },
            FieldsChannel::DefaultsPermission => match self {
                Self::TextChannel {
                    default_permissions,
                    ..
                }
                | Self::VoiceChannel {
                    default_permissions,
                    ..
                } => {
                    default_permissions.take();
                }
                _ => {}
            },
        }
    }

    pub fn apply_options(&mut self, partial: PartialChannel) {
        // ! FIXME: maybe flatten channel object?
        match self {
            Self::DirectMessage { active, .. } => {
                if let Some(v) = partial.active {
                    *active = v;
                }
            }
            Self::Group {
                name,
                owner,
                description,
                icon,
                permissions,
                ..
            } => {
                if let Some(v) = partial.name {
                    *name = v;
                }

                if let Some(v) = partial.owner {
                    *owner = v;
                }

                if let Some(v) = partial.description {
                    description.replace(v);
                }

                if let Some(v) = partial.icon {
                    icon.replace(v);
                }

                if let Some(v) = partial.permission {
                    permissions.replace(v);
                }
            }
            Self::TextChannel {
                name,
                description,
                icon,
                default_permissions,
                role_permissions,
                ..
            }
            | Self::VoiceChannel {
                name,
                description,
                icon,
                default_permissions,
                role_permissions,
                ..
            } => {
                if let Some(v) = partial.name {
                    *name = v;
                }

                if let Some(v) = partial.description {
                    description.replace(v);
                }

                if let Some(v) = partial.icon {
                    icon.replace(v);
                }

                if let Some(v) = partial.role_permissions {
                    *role_permissions = v;
                }

                if let Some(v) = partial.default_permissions {
                    default_permissions.replace(v);
                }
            }
            _ => {}
        }
    }

    pub async fn update<'a>(
        &mut self,
        db: &Database,
        partial: PartialChannel,
        remove: Vec<FieldsChannel>,
    ) -> Result<()> {
        for field in &remove {
            self.remove(field);
        }

        self.apply_options(partial.clone());

        let id = self.id().to_string();
        db.update_channel(&id, &partial, remove.clone()).await?;

        EventV1::ChannelUpdate {
            id: id.clone(),
            data: partial,
            clear: remove,
        }
        .p(match self {
            Self::TextChannel { server, .. } | Self::VoiceChannel { server, .. } => server.clone(),
            _ => id,
        })
        .await;

        Ok(())
    }

    /// Delete a channel
    pub async fn delete(self, db: &Database) -> Result<()> {
        let id = self.id().to_string();
        EventV1::ChannelDelete { id: id.clone() }.p(id).await;
        db.delete_channel(&self).await
    }

    /// Acknowledge a message
    pub async fn ack(&self, user: &str, message: &str) -> Result<()> {
        EventV1::ChannelAck {
            id: self.id().to_string(),
            user: user.to_string(),
            message_id: message.to_string(),
        }
        .private(user.to_string())
        .await;

        crate::tasks::ack::queue(
            self.id().to_string(),
            user.to_string(),
            AckEvent::AckMessage {
                id: message.to_string(),
            },
        )
        .await;

        Ok(())
    }

    /// Add user to a group
    pub async fn add_user_to_group(&mut self, db: &Database, user: &str, by: &str) -> Result<()> {
        if let Channel::Group { recipients, .. } = self {
            let user = user.to_string();
            if recipients.contains(&user) {
                return Err(Error::AlreadyInGroup);
            }

            recipients.push(user);
        }

        match &self {
            Channel::Group { id, .. } => {
                db.add_user_to_group(id, user).await?;

                EventV1::ChannelGroupJoin {
                    id: id.to_string(),
                    user: user.to_string(),
                }
                .p(id.to_string())
                .await;

                EventV1::ChannelCreate(self.clone())
                    .private(user.to_string())
                    .await;

                SystemMessage::UserAdded {
                    id: user.to_string(),
                    by: by.to_string(),
                }
                .into_message(id.to_string())
                .create(db, self, None)
                .await
                .ok();

                Ok(())
            }
            _ => Err(Error::InvalidOperation),
        }
    }

    pub async fn set_role_permission(
        &mut self,
        db: &Database,
        role: &str,
        permissions: OverrideField,
    ) -> Result<()> {
        match self {
            Channel::TextChannel {
                id,
                server,
                role_permissions,
                ..
            }
            | Channel::VoiceChannel {
                id,
                server,
                role_permissions,
                ..
            } => {
                let _ = db.set_channel_role_permission(id, role, permissions).await;
                role_permissions.insert(role.to_string(), permissions);

                EventV1::ChannelUpdate {
                    id: id.clone(),
                    data: PartialChannel {
                        role_permissions: Some(role_permissions.clone()),
                        ..Default::default()
                    },
                    clear: vec![],
                }
                .p(server.clone())
                .await;

                Ok(())
            }
            _ => Err(Error::InvalidOperation),
        }
    }

    pub async fn create_server_channel(
        db: &Database,
        server: &mut Server,
        data: DataCreateServerChannel,
        update_server: bool,
    ) -> Result<Channel> {
        // let config = config().await;
        // if server.channels.len() > config.features.limits.default.server_channels {
        //     return Err(create_error!(TooManyChannels {
        //         max: config.features.limits.default.server_channels,
        //     }));
        // };

        let id = ulid::Ulid::new().to_string();
        let channel = match data.channel_type {
            LegacyServerChannelType::Text => Channel::TextChannel {
                id: id.clone(),
                server: server.id.to_owned(),
                name: data.name,
                description: data.description,
                icon: None,
                last_message_id: None,
                default_permissions: None,
                role_permissions: HashMap::new(),
            },
            LegacyServerChannelType::Voice => Channel::VoiceChannel {
                id: id.clone(),
                server: server.id.to_owned(),
                name: data.name,
                description: data.description,
                icon: None,
                default_permissions: None,
                role_permissions: HashMap::new(),
            },
        };

        db.insert_channel(&channel).await?;

        if update_server {
            server
                .update(
                    db,
                    PartialServer {
                        channels: Some([server.channels.clone(), [id].into()].concat()),
                        ..Default::default()
                    },
                    vec![],
                )
                .await?;

            EventV1::ChannelCreate(channel.clone())
                .p(server.id.clone())
                .await;
        }

        Ok(channel)
    }
}
