// use std::collections::HashMap;

use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{models::File, permissions::defn::OverrideField};

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
