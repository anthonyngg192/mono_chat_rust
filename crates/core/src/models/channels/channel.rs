// use std::collections::HashMap;

use std::collections::{HashMap, HashSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;
extern crate serde;

use crate::{models::File, permissions::defn::OverrideField};

pub enum ChannelType {
    SavedMessages,
    DirectMessage,
    Group,
    ServerChannel,
    Unknown,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(tag = "channel_type")]
pub enum Channel {
    SavedMessages {
        #[serde(rename = "_id")]
        id: String,
        user: String,
    },

    DirectMessage {
        #[serde(rename = "_id")]
        id: String,
        active: bool,

        //receivers
        recipients: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        last_message_id: Option<String>,
    },

    Group {
        #[serde(rename = "_id")]
        id: String,
        name: String,
        owner: String,

        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,

        //receivers
        recipients: Vec<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        icon: Option<File>,

        #[serde(skip_serializing_if = "Option::is_none")]
        last_message_id: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        permission: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        permissions: Option<i64>,
    },

    TextChannel {
        #[serde(rename = "_id")]
        id: String,

        server: String,
        name: String,

        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        icon: Option<File>,

        #[serde(skip_serializing_if = "Option::is_none")]
        last_message_id: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        default_permissions: Option<OverrideField>,

        #[serde(
            default = "HashMap::<String, OverrideField>::new",
            skip_serializing_if = "HashMap::<String, OverrideField>::is_empty"
        )]
        role_permissions: HashMap<String, OverrideField>,
    },

    VoiceChannel {
        #[serde(rename = "_id")]
        id: String,

        server: String,
        name: String,

        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        icon: Option<File>,

        #[serde(skip_serializing_if = "Option::is_none")]
        default_permissions: Option<OverrideField>,

        #[serde(
            default = "HashMap::<String, OverrideField>::new",
            skip_serializing_if = "HashMap::<String, OverrideField>::is_empty"
        )]
        role_permissions: HashMap<String, OverrideField>,
    },
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, PartialEq, Eq, Clone)]
pub enum FieldsChannel {
    Description,
    Icon,
    DefaultsPermission,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Default, Clone)]
pub struct PartialChannel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<File>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_permissions: Option<HashMap<String, OverrideField>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_permissions: Option<OverrideField>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum LegacyServerChannelType {
    Text,
    Voice,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "data_server_channel")]
#[cfg_attr(feature = "validator", derive(validator::Validate))]
pub struct DataCreateServerChannel {
    #[serde(rename = "type")]
    pub channel_type: LegacyServerChannelType,

    #[cfg_attr(feature = "validator", validate(length(min = 1, max = 32)))]
    pub name: String,

    #[cfg_attr(feature = "validator", validate(length(min = 0, max = 1024)))]
    pub description: Option<String>,
}

#[derive(Validate, Serialize, Deserialize, Clone)]
pub struct DataEditChannel {
    #[cfg_attr(feature = "validator", validate(length(min = 1, max = 32)))]
    pub name: Option<String>,

    #[cfg_attr(feature = "validator", validate(length(min = 0, max = 1024)))]
    pub description: Option<String>,

    pub owner: Option<String>,

    #[cfg_attr(feature = "validator", validate(length(min = 1, max = 128)))]
    pub icon: Option<String>,

    pub nsfw: Option<bool>,

    pub archived: Option<bool>,

    #[cfg_attr(feature = "serde", serde(default))]
    pub remove: Option<Vec<FieldsChannel>>,
}

#[derive(Validate, Serialize, Deserialize, Clone, Default)]
pub struct DataCreateGroup {
    #[cfg_attr(feature = "validator", validate(length(min = 1, max = 32)))]
    pub name: String,

    #[cfg_attr(feature = "validator", validate(length(min = 0, max = 1024)))]
    pub description: Option<String>,

    #[cfg_attr(feature = "validator", validate(length(min = 1, max = 128)))]
    pub icon: Option<String>,

    #[cfg_attr(feature = "validator", validate(length(min = 0, max = 49)))]
    #[serde(default)]
    pub users: HashSet<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,
}
