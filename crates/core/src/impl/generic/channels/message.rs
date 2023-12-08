use std::collections::HashSet;

use serde_json::json;
use ulid::Ulid;

use crate::{
    database::Database,
    events::client::EventV1,
    models::{
        message::{
            AppendMessage, BulkMessageResponse, DataMessageSend, Interactions, MessageAuthor,
            PartialMessage, Reply, SendableEmbed, SystemMessage, RE_MENTION,
        },
        Channel, Emoji, File, Message, User,
    },
    permissions::{defn::ChannelPermission, PermissionCalculator},
    presence::presence_filter_online,
    tasks::{self, ack::AckEvent},
    types::{
        january::{Embed, Text},
        push::PushNotification,
    },
    util::idempotency::IdempotencyKey,
    Error, Result,
};

impl Message {
    pub async fn create_no_web_push(
        &mut self,
        db: &Database,
        channel: &str,
        is_direct_dm: bool,
    ) -> Result<()> {
        db.insert_message(self).await?;

        EventV1::Message(self.clone()).p(channel.to_string()).await;

        crate::tasks::last_message_id::queue(
            channel.to_string(),
            self.id.to_string(),
            is_direct_dm,
        )
        .await;

        if let Some(mentions) = &self.mentions {
            for user in mentions {
                crate::tasks::ack::queue(
                    channel.to_string(),
                    user.to_string(),
                    AckEvent::AddMention {
                        ids: vec![self.id.to_string()],
                    },
                )
                .await;
            }
        }

        Ok(())
    }

    pub async fn append(
        db: &Database,
        id: String,
        channel: String,
        append: AppendMessage,
    ) -> Result<()> {
        db.append_message(&id, &append).await?;

        EventV1::MessageAppend {
            id,
            channel: channel.to_string(),
            append,
        }
        .p(channel)
        .await;

        Ok(())
    }

    pub async fn create(
        &mut self,
        db: &Database,
        channel: &Channel,
        sender: Option<&User>,
    ) -> Result<()> {
        self.create_no_web_push(db, channel.id(), channel.is_direct_dm())
            .await?;

        crate::tasks::web_push::queue(
            {
                let mut target_ids = vec![];
                match &channel {
                    Channel::DirectMessage { recipients, .. }
                    | Channel::Group { recipients, .. } => {
                        target_ids = (&recipients.iter().cloned().collect::<HashSet<String>>()
                            - &presence_filter_online(recipients).await)
                            .into_iter()
                            .collect::<Vec<String>>();
                    }
                    Channel::TextChannel { .. } => {
                        if let Some(mentions) = &self.mentions {
                            target_ids.append(&mut mentions.clone());
                        }
                    }
                    _ => {}
                };
                target_ids
            },
            json!(PushNotification::new(self.clone(), sender, channel.id())).to_string(),
        )
        .await;

        Ok(())
    }

    pub async fn update(&mut self, db: &Database, partial: PartialMessage) -> Result<()> {
        self.apply_options(partial.clone());
        let _ = db.update_message(&self.id, &partial).await;
        EventV1::MessageUpdate {
            id: self.id.clone(),
            channel: self.channel.clone(),
            data: partial,
        }
        .p(self.channel.clone())
        .await;
        Ok(())
    }

    pub async fn delete(self, db: &Database) -> Result<()> {
        let file_ids: Vec<String> = self
            .attachments
            .map(|files| files.iter().map(|file| file.id.to_string()).collect())
            .unwrap_or_default();

        if !file_ids.is_empty() {
            let _ = db.mark_attachments_as_deleted(&file_ids).await;
        }

        let _ = db.delete_message(&self.id).await;

        EventV1::MessageDelete {
            id: self.id,
            channel: self.channel.clone(),
        }
        .p(self.channel)
        .await;
        Ok(())
    }

    pub async fn bulk_delete(db: &Database, channel: &str, ids: Vec<String>) -> Result<()> {
        db.delete_messages(channel, ids.clone()).await?;
        EventV1::BulkMessageDelete {
            channel: channel.to_string(),
            ids,
        }
        .p(channel.to_string())
        .await;
        Ok(())
    }

    pub fn validate_sum(
        content: &Option<String>,
        embeds: &Option<Vec<SendableEmbed>>,
    ) -> Result<()> {
        let mut running_total = 0;
        if let Some(content) = content {
            running_total += content.len();
        }

        if let Some(embeds) = embeds {
            for embed in embeds {
                if let Some(desc) = &embed.description {
                    running_total += desc.len();
                }
            }
        }

        if running_total <= 2000 {
            Ok(())
        } else {
            Err(Error::PayloadTooLarge)
        }
    }

    pub async fn add_reaction(&self, db: &Database, user: &User, emoji: &str) -> Result<()> {
        if self.reactions.len() >= 20 {
            return Err(Error::InvalidOperation);
        }

        if !self.interactions.can_use(emoji) {
            return Err(Error::InvalidOperation);
        }

        if !Emoji::can_use(db, emoji).await? {
            return Err(Error::InvalidOperation);
        }

        EventV1::MessageReact {
            id: self.id.to_string(),
            channel_id: self.channel.to_string(),
            user_id: user.id.to_string(),
            emoji_id: emoji.to_string(),
        }
        .p(self.channel.to_string())
        .await;

        db.add_reaction(&self.id, emoji, &user.id).await
    }

    pub async fn remove_reaction(&self, db: &Database, user: &str, emoji: &str) -> Result<()> {
        let empty = if let Some(users) = self.reactions.get(emoji) {
            if !users.contains(user) {
                return Err(Error::NotFound);
            }

            users.len() == 1
        } else {
            return Err(Error::NotFound);
        };

        EventV1::MessageUnreact {
            id: self.id.to_string(),
            channel_id: self.channel.to_string(),
            user_id: user.to_string(),
            emoji_id: emoji.to_string(),
        }
        .p(self.channel.to_string())
        .await;

        if empty {
            db.clear_reaction(&self.id, emoji).await
        } else {
            db.remove_reaction(&self.id, emoji, user).await
        }
    }

    pub async fn clear_reaction(&self, db: &Database, emoji: &str) -> Result<()> {
        EventV1::MessageRemoveReaction {
            id: self.id.to_string(),
            channel_id: self.channel.to_string(),
            emoji_id: emoji.to_string(),
        }
        .p(self.channel.to_string())
        .await;

        db.clear_reaction(&self.id, emoji).await
    }

    pub async fn send_without_notifications(
        &mut self,
        db: &Database,
        is_dm: bool,
        generate_embeds: bool,
    ) -> Result<()> {
        db.insert_message(self).await?;

        EventV1::Message(self.clone())
            .p(self.channel.to_string())
            .await;

        tasks::last_message_id::queue(self.channel.to_string(), self.id.to_string(), is_dm).await;

        if let Some(mentions) = &self.mentions {
            for user in mentions {
                tasks::ack::queue(
                    self.channel.to_string(),
                    user.to_string(),
                    AckEvent::AddMention {
                        ids: vec![self.id.to_string()],
                    },
                )
                .await;
            }
        }

        if generate_embeds {
            if let Some(content) = &self.content {
                tasks::process_embeds::queue(
                    self.channel.to_string(),
                    self.id.to_string(),
                    content.clone(),
                )
                .await;
            }
        }

        Ok(())
    }

    pub async fn create_from_api(
        db: &Database,
        channel: Channel,
        data: DataMessageSend,
        author: MessageAuthor<'_>,
        mut idempotency: IdempotencyKey,
        generate_embeds: bool,
    ) -> Result<Message> {
        idempotency
            .consume_nonce(data.nonce)
            .await
            .map_err(|_| Error::InvalidOperation)?;

        if (data.content.as_ref().map_or(true, |v| v.is_empty()))
            && (data.attachments.as_ref().map_or(true, |v| v.is_empty()))
            && (data.embeds.as_ref().map_or(true, |v| v.is_empty()))
        {
            return Err(Error::EmptyMessage);
        }

        if let Some(interactions) = &data.interactions {
            if interactions.restrict_reactions {
                let disallowed = if let Some(list) = &interactions.reactions {
                    list.is_empty()
                } else {
                    true
                };

                if disallowed {
                    return Err(Error::InvalidProperty);
                }
            }
        }

        let (author_id, webhook) = match &author {
            MessageAuthor::User(user) => (user.id.clone(), None),
            // MessageAuthor::Webhook(webhook) => (webhook.id.clone(), Some((*webhook).clone())),
            MessageAuthor::System { .. } => ("00000000000000000000000000".to_string(), None),
        };

        let message_id = Ulid::new().to_string();
        let mut message = Message {
            id: message_id.clone(),
            channel: channel.id().to_string(),
            masquerade: data.masquerade.map(|masquerade| masquerade.into()),
            interactions: data
                .interactions
                .map(|interactions| interactions.into())
                .unwrap_or_default(),
            author: author_id,
            ..Default::default()
        };

        let mut mentions = HashSet::new();
        if let Some(content) = &data.content {
            for capture in RE_MENTION.captures_iter(content) {
                if let Some(mention) = capture.get(1) {
                    mentions.insert(mention.as_str().to_string());
                }
            }
        }

        let mut replies = HashSet::new();
        if let Some(entries) = data.replies {
            for Reply { id, mention } in entries {
                let message = db.fetch_message(&id).await?;

                if mention {
                    mentions.insert(message.author.to_owned());
                }

                replies.insert(message.id);
            }
        }

        if !mentions.is_empty() {
            message.mentions.replace(mentions.into_iter().collect());
        }

        if !replies.is_empty() {
            message
                .replies
                .replace(replies.into_iter().collect::<Vec<String>>());
        }

        let mut attachments = vec![];

        for attachment_id in data.attachments.as_deref().unwrap_or_default() {
            attachments.push(
                db.find_and_use_attachment(attachment_id, "attachments", "message", &message_id)
                    .await?,
            );
        }

        if !attachments.is_empty() {
            message.attachments.replace(attachments);
        }

        for sendable_embed in data.embeds.unwrap_or_default() {
            message.attach_sendable_embed(db, sendable_embed).await?;
        }

        message.content = data.content;

        message.nonce = Some(idempotency.into_key());

        message.send(db, author, &channel, generate_embeds).await?;

        Ok(message)
    }

    pub async fn send(
        &mut self,
        db: &Database,
        author: MessageAuthor<'_>,
        channel: &Channel,
        generate_embeds: bool,
    ) -> Result<()> {
        self.send_without_notifications(
            db,
            matches!(channel, Channel::DirectMessage { .. }),
            generate_embeds,
        )
        .await?;

        // Push out Web Push notifications
        crate::tasks::web_push::queue(
            {
                match channel {
                    Channel::DirectMessage { recipients, .. }
                    | Channel::Group { recipients, .. } => recipients.clone(),
                    Channel::TextChannel { .. } => self.mentions.clone().unwrap_or_default(),
                    _ => vec![],
                }
            },
            PushNotification::from(self.clone().into(), Some(author), &channel.id()).await,
        )
        .await;

        Ok(())
    }

    pub async fn attach_sendable_embed(
        &mut self,
        db: &Database,
        embed: SendableEmbed,
    ) -> Result<()> {
        let media: Option<File> = if let id = embed.media {
            Some(
                db.find_and_use_attachment(&id, "attachments", "message", &self.id)
                    .await?
                    .into(),
            )
        } else {
            None
        };

        let embed = Embed::Text(Text {
            icon_url: embed.icon_url,
            url: embed.url,
            title: embed.title,
            description: embed.description,
            media,
            colour: embed.colour,
        });

        if let Some(embeds) = &mut self.embeds {
            embeds.push(embed);
        } else {
            self.embeds = Some(vec![embed]);
        }

        Ok(())
    }
}

pub trait IntoUsers {
    fn get_user_ids(&self) -> Vec<String>;
}

impl IntoUsers for Message {
    fn get_user_ids(&self) -> Vec<String> {
        let mut ids = vec![self.author.clone()];

        if let Some(msg) = &self.system {
            match msg {
                SystemMessage::UserAdded { id, by, .. }
                | SystemMessage::UserRemove { id, by, .. } => {
                    ids.push(id.clone());
                    ids.push(by.clone());
                }
                SystemMessage::UserJoined { id, .. }
                | SystemMessage::UserLeft { id, .. }
                | SystemMessage::UserKicked { id, .. }
                | SystemMessage::UserBanned { id, .. } => ids.push(id.clone()),
                SystemMessage::ChannelRenamed { by, .. }
                | SystemMessage::ChannelDescriptionChanged { by, .. }
                | SystemMessage::ChannelIconChanged { by, .. } => ids.push(by.clone()),
                _ => {}
            }
        }

        ids
    }
}

impl IntoUsers for Vec<Message> {
    fn get_user_ids(&self) -> Vec<String> {
        let mut ids = vec![];
        for message in self {
            ids.append(&mut message.get_user_ids());
        }

        ids
    }
}

impl SystemMessage {
    pub fn into_message(self, channel: String) -> Message {
        Message {
            id: Ulid::new().to_string(),
            channel,
            author: "00000000000000000000000000".to_string(),
            system: Some(self),

            ..Default::default()
        }
    }
}

impl From<SystemMessage> for String {
    fn from(s: SystemMessage) -> String {
        match s {
            SystemMessage::Text { content } => content,
            SystemMessage::UserAdded { .. } => "User added to the channel.".to_string(),
            SystemMessage::UserRemove { .. } => "User removed from the channel.".to_string(),
            SystemMessage::UserJoined { .. } => "User joined the channel.".to_string(),
            SystemMessage::UserLeft { .. } => "User left the channel.".to_string(),
            SystemMessage::UserKicked { .. } => "User kicked from the channel.".to_string(),
            SystemMessage::UserBanned { .. } => "User banned from the channel.".to_string(),
            SystemMessage::ChannelRenamed { .. } => "Channel renamed.".to_string(),
            SystemMessage::ChannelDescriptionChanged { .. } => {
                "Channel description changed.".to_string()
            }
            SystemMessage::ChannelIconChanged { .. } => "Channel icon changed.".to_string(),
            SystemMessage::ChannelOwnershipChanged { .. } => {
                "Channel ownership changed.".to_string()
            }
        }
    }
}

impl Interactions {
    pub async fn validate(
        &self,
        db: &Database,
        permissions: &mut PermissionCalculator<'_>,
    ) -> Result<()> {
        if let Some(reactions) = &self.reactions {
            permissions
                .throw_permission(db, ChannelPermission::React)
                .await?;

            if reactions.len() > 20 {
                return Err(Error::InvalidOperation);
            }

            for reaction in reactions {
                if !Emoji::can_use(db, reaction).await? {
                    return Err(Error::InvalidOperation);
                }
            }
        }

        Ok(())
    }

    pub fn can_use(&self, emoji: &str) -> bool {
        if self.restrict_reactions {
            if let Some(reactions) = &self.reactions {
                reactions.contains(emoji)
            } else {
                false
            }
        } else {
            true
        }
    }

    pub fn is_default(&self) -> bool {
        !self.restrict_reactions && self.reactions.is_none()
    }
}

impl SendableEmbed {
    pub async fn into_embed(self, db: &Database, message_id: String) -> Result<Embed> {
        let media = if let Some(id) = self.media {
            Some(
                db.find_and_use_attachment(&id, "attachments", "message", &message_id)
                    .await?,
            )
        } else {
            None
        };

        Ok(Embed::Text(Text {
            icon_url: self.icon_url,
            url: self.url,
            title: self.title,
            description: self.description,
            media,
            colour: self.colour,
        }))
    }
}

impl BulkMessageResponse {
    pub async fn transform(
        db: &Database,
        channel: Option<&Channel>,
        messages: Vec<Message>,
        include_users: Option<bool>,
    ) -> Result<BulkMessageResponse> {
        if let Some(true) = include_users {
            let user_ids = messages.get_user_ids();
            let users = User::fetch_foreign_users(db, &user_ids).await?;

            Ok(match channel {
                Some(Channel::TextChannel { server, .. })
                | Some(Channel::VoiceChannel { server, .. }) => {
                    BulkMessageResponse::MessagesAndUsers {
                        messages,
                        users,
                        members: Some(db.fetch_members(server, &user_ids).await?),
                    }
                }
                _ => BulkMessageResponse::MessagesAndUsers {
                    messages,
                    users,
                    members: None,
                },
            })
        } else {
            Ok(BulkMessageResponse::JustMessage(messages))
        }
    }
}

impl<'a> MessageAuthor<'a> {
    pub fn id(&self) -> &str {
        match self {
            MessageAuthor::User(user) => &user.id,
            // MessageAuthor::Webhook(webhook) => &webhook.id,
            MessageAuthor::System { .. } => "00000000000000000000000000",
        }
    }

    pub fn avatar(&self) -> Option<&str> {
        match self {
            MessageAuthor::User(user) => user.avatar.as_ref().map(|file| file.id.as_str()),
            // MessageAuthor::Webhook(webhook) => webhook.avatar.as_ref().map(|file| file.id.as_str()),
            MessageAuthor::System { avatar, .. } => *avatar,
        }
    }

    pub fn username(&self) -> &str {
        match self {
            MessageAuthor::User(user) => &user.username,
            // MessageAuthor::Webhook(webhook) => &webhook.name,
            MessageAuthor::System { username, .. } => username,
        }
    }
}
