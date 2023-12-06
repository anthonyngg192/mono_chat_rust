use std::collections::HashMap;

use num_enum::TryFromPrimitive;
use revolt_optional_struct::OptionalStruct;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{models::File, permissions::defn::OverrideField};

pub fn if_false(t: &bool) -> bool {
    !t
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, OptionalStruct, Default)]
#[optional_derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Default)]
#[optional_name = "PartialRole"]
#[opt_skip_serializing_none]
#[opt_some_priority]
pub struct Role {
    pub name: String,
    pub permissions: OverrideField,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colour: Option<String>,

    #[serde(skip_serializing_if = "if_false", default)]
    pub hoist: bool,

    #[serde(default)]
    pub rank: i64,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Default)]

pub struct Category {
    #[validate(length(min = 1, max = 32))]
    pub id: String,

    #[validate(length(min = 1, max = 32))]
    pub title: String,

    pub channels: Vec<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct SystemMessageChannels {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_joined: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_left: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_kicked: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_banned: Option<String>,
}

#[derive(Debug, PartialEq, Eq, TryFromPrimitive, Clone, Copy)]
#[repr(i32)]
pub enum ServerFlag {
    Verified = 1,
    Official = 2,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, OptionalStruct, Default)]
#[optional_derive(Serialize, Deserialize, JsonSchema, Debug, Default, Clone)]
#[optional_name = "PartialServer"]
#[opt_skip_serializing_none]
#[opt_some_priority]
pub struct Server {
    #[serde(rename = "_id")]
    pub id: String,

    pub owner: String,
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    pub channels: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_messages: Option<SystemMessageChannels>,

    #[serde(
        default = "HashMap::<String, Role>::new",
        skip_serializing_if = "HashMap::<String, Role>::is_empty"
    )]
    pub roles: HashMap<String, Role>,

    pub default_permissions: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<File>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<File>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<i32>,

    #[serde(skip_serializing_if = "if_false", default)]
    pub nsfw: bool,

    #[serde(skip_serializing_if = "if_false", default)]
    pub analytics: bool,

    #[serde(skip_serializing_if = "if_false", default)]
    pub discoverable: bool,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, PartialEq, Eq, Clone)]
pub enum FieldsServer {
    Description,
    Categories,
    SystemMessages,
    Icon,
    Banner,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, PartialEq, Eq, Clone)]
pub enum FieldsRole {
    Colour,
}
