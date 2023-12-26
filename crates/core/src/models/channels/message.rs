use indexmap::{IndexMap, IndexSet};
use iso8601_timestamp::Timestamp;
use once_cell::sync::Lazy;
use regex::Regex;
use revolt_optional_struct::OptionalStruct;
use rocket::FromFormField;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    models::{File, Member, User},
    types::january::Embed,
};

use super::channel::{MessageWebhook, Webhook};

pub static RE_COLOUR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(?:[a-z ]+|var\(--[a-z\d-]+\)|rgba?\([\d, ]+\)|#[a-f0-9]+|(repeating-)?(linear|conic|radial)-gradient\(([a-z ]+|var\(--[a-z\d-]+\)|rgba?\([\d, ]+\)|#[a-f0-9]+|\d+deg)([ ]+(\d{1,3}%|0))?(,[ ]*([a-z ]+|var\(--[a-z\d-]+\)|rgba?\([\d, ]+\)|#[a-f0-9]+)([ ]+(\d{1,3}%|0))?)+\))$").unwrap()
});

pub fn if_false(t: &bool) -> bool {
    !t
}

#[derive(Clone, Debug)]
pub struct Rely {
    pub id: String,
    pub mention: bool,
}

#[derive(Validate, Serialize, Deserialize, JsonSchema, Clone, Debug, Default)]
pub struct SendableEmbed {
    #[validate(length(min = 1, max = 128))]
    pub icon_url: Option<String>,
    pub url: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub title: Option<String>,
    #[validate(length(min = 1, max = 2000))]
    pub description: Option<String>,
    pub media: Option<String>,
    #[validate(length(min = 1, max = 200), regex = "RE_COLOUR")]
    pub colour: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
#[serde(tag = "Type")]
pub enum SystemMessage {
    #[serde(rename = "text")]
    Text { content: String },

    #[serde(rename = "user_added")]
    UserAdded { id: String, by: String },

    #[serde(rename = "user_remove")]
    UserRemove { id: String, by: String },

    #[serde(rename = "user_joined")]
    UserJoined { id: String },

    #[serde(rename = "user_left")]
    UserLeft { id: String },

    #[serde(rename = "user_kicked")]
    UserKicked { id: String },

    #[serde(rename = "user_banned")]
    UserBanned { id: String },

    #[serde(rename = "channel_renamed")]
    ChannelRenamed { name: String, by: String },

    #[serde(rename = "channel_description_changed")]
    ChannelDescriptionChanged { by: String },

    #[serde(rename = "channel_icon_changed")]
    ChannelIconChanged { by: String },

    #[serde(rename = "channel_ownership_changed")]
    ChannelOwnershipChanged { from: String, to: String },
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone, Validate)]
pub struct Masquerade {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(min = 1, max = 32))]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(min = 1, max = 256))]
    pub avatar: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(min = 1, max = 128), regex = "RE_COLOUR")]
    pub colour: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Validate, Default, JsonSchema)]
pub struct Interactions {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub reactions: Option<IndexSet<String>>,

    #[serde(skip_serializing_if = "if_false", default)]
    pub restrict_reactions: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, OptionalStruct, Default)]
#[optional_derive(Serialize, Deserialize, Debug, Default, Clone)]
#[optional_name = "PartialMessage"]
#[opt_some_priority]
pub struct Message {
    #[serde(rename = "_id")]
    pub id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    pub channel: String,
    pub author: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<MessageWebhook>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemMessage>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<File>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub edited: Option<Timestamp>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub embeds: Option<Vec<Embed>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mentions: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub replies: Option<Vec<String>>,

    #[serde(skip_serializing_if = "IndexMap::is_empty", default)]
    pub reactions: IndexMap<String, IndexSet<String>>,

    #[serde(skip_serializing_if = "Interactions::is_default", default)]
    pub interactions: Interactions,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub masquerade: Option<Masquerade>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Default)]
#[cfg_attr(feature = "rocket_impl", derive(FromFormField))]
pub enum MessageSort {
    #[default]
    Relevance,
    Latest,
    Oldest,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum MessageTimePeriod {
    Relative {
        nearby: String,
    },
    Absolute {
        before: Option<String>,
        after: Option<String>,
        sort: Option<MessageSort>,
    },
}

#[derive(Serialize, Deserialize, JsonSchema, Default)]
pub struct MessageFilter {
    pub channel: Option<String>,
    pub author: Option<String>,
    pub query: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct MessageQuery {
    pub limit: Option<i64>,
    #[serde(flatten)]
    pub filter: MessageFilter,
    #[serde(flatten)]
    pub time_period: MessageTimePeriod,
}

#[derive(Serialize, Deserialize)]
pub enum BulkMessageResponse {
    JustMessage(Vec<Message>),

    MessagesAndUsers {
        messages: Vec<Message>,
        users: Vec<User>,

        #[serde(skip_serializing_if = "Option::is_none")]
        members: Option<Vec<Member>>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppendMessage {
    pub embeds: Option<Vec<Embed>>,
}

#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataMessageSend {
    #[validate(length(min = 1, max = 64))]
    pub nonce: Option<String>,

    #[validate(length(min = 0, max = 2000))]
    pub content: Option<String>,

    pub attachments: Option<Vec<String>>,

    pub replies: Option<Vec<Reply>>,

    #[validate]
    pub embeds: Option<Vec<SendableEmbed>>,

    #[validate]
    pub masquerade: Option<Masquerade>,
    pub interactions: Option<Interactions>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct Reply {
    pub id: String,
    pub mention: bool,
}

pub enum MessageAuthor<'a> {
    User(&'a User),
    Webhook(&'a Webhook),
    System {
        username: &'a str,
        avatar: Option<&'a str>,
    },
}

pub static RE_MENTION: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"<@([0-9A-HJKMNP-TV-Z]{26})>").unwrap());
