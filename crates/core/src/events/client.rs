use authifier::AuthifierEvent;
use serde::{Deserialize, Serialize};

use crate::{
    models::{
        channel::{FieldsChannel, FieldsWebhook, PartialChannel, PartialWebhook, Webhook},
        message::{AppendMessage, PartialMessage},
        server::{FieldsRole, FieldsServer, PartialRole, PartialServer},
        server_member::{FieldsMember, MemberCompositeKey, PartialMember},
        user::{FieldsUser, PartialUser},
        Channel, Emoji, Member, Message, Report, Server, User, UserSettings,
    },
    Error,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "error")]
pub enum WebSocketError {
    LabelMe,
    InternalError { at: String },
    InvalidSession,
    OnboardingNotFinished,
    AlreadyAuthenticated,
    MalformedData { msg: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Ping {
    Binary(Vec<u8>),
    Number(usize),
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum ErrorEvent {
    Error(WebSocketError),
    APIError(Error),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum EventV1 {
    Bulk {
        v: Vec<EventV1>,
    },

    Authenticated,

    Ready {
        users: Vec<User>,
        servers: Vec<Server>,
        channels: Vec<Channel>,
        members: Vec<Member>,
        emojis: Option<Vec<Emoji>>,
    },

    Pong {
        data: Ping,
    },

    Message(Message),

    MessageUpdate {
        id: String,
        channel: String,
        data: PartialMessage,
    },

    MessageAppend {
        id: String,
        channel: String,
        append: AppendMessage,
    },

    MessageDelete {
        id: String,
        channel: String,
    },

    MessageReact {
        id: String,
        channel_id: String,
        user_id: String,
        emoji_id: String,
    },

    MessageUnreact {
        id: String,
        channel_id: String,
        user_id: String,
        emoji_id: String,
    },

    MessageRemoveReaction {
        id: String,
        channel_id: String,
        emoji_id: String,
    },

    BulkMessageDelete {
        channel: String,
        ids: Vec<String>,
    },

    ChannelCreate(Channel),

    ChannelUpdate {
        id: String,
        data: PartialChannel,
        clear: Vec<FieldsChannel>,
    },

    ChannelDelete {
        id: String,
    },

    ChannelGroupJoin {
        id: String,
        user: String,
    },

    ChannelGroupLeave {
        id: String,
        user: String,
    },

    ChannelStartTyping {
        id: String,
        user: String,
    },

    ChannelStopTyping {
        id: String,
        user: String,
    },

    ChannelAck {
        id: String,
        user: String,
        message_id: String,
    },

    ServerCreate {
        id: String,
        server: Server,
        channels: Vec<Channel>,
    },

    ServerUpdate {
        id: String,
        data: PartialServer,
        clear: Vec<FieldsServer>,
    },

    ServerDelete {
        id: String,
    },

    ServerMemberUpdate {
        id: MemberCompositeKey,
        data: PartialMember,
        clear: Vec<FieldsMember>,
    },

    ServerMemberJoin {
        id: String,
        user: String,
    },

    ServerMemberLeave {
        id: String,
        user: String,
    },

    ServerRoleUpdate {
        id: String,
        role_id: String,
        data: PartialRole,
        clear: Vec<FieldsRole>,
    },

    ServerRoleDelete {
        id: String,
        role_id: String,
    },

    UserUpdate {
        id: String,
        data: PartialUser,
        clear: Vec<FieldsUser>,
    },

    UserRelationship {
        id: String,
        user: User,
    },

    UserSettingsUpdate {
        id: String,
        update: UserSettings,
    },

    UserPlatformWipe {
        user_id: String,
        flags: i32,
    },

    EmojiCreate(Emoji),

    EmojiDelete {
        id: String,
    },

    ReportCreate(Report),

    WebhookCreate(Webhook),

    WebhookUpdate {
        id: String,
        data: PartialWebhook,
        remove: Vec<FieldsWebhook>,
    },

    WebhookDelete {
        id: String,
    },

    Auth(AuthifierEvent),
}
