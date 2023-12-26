use num_enum::TryFromPrimitive;
use once_cell::sync::Lazy;
use regex::Regex;
use revolt_optional_struct::OptionalStruct;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Utility function to check if a boolean value is false
pub fn if_false(t: &bool) -> bool {
    !t
}

use crate::models::media::attachment::File;
pub static RE_USERNAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\p{L}|[\d_.-])+$").unwrap());

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq, Default)]
pub enum RelationshipStatus {
    #[default]
    None,
    User,
    Friend,
    Outgoing,
    Incoming,
    Blocked,
    BlockedOther,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct Relationship {
    #[serde(rename = "_id")]
    pub user_id: String,
    pub status: RelationshipStatus,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
pub enum Presence {
    Online,
    Idle,
    Focus,
    Busy,
    Invisible,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Validate, Default)]
pub struct UserStatus {
    #[validate(length(min = 1, max = 128))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence: Option<Presence>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Default)]
pub struct UserProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<File>,
}

/// User badge bitfield
#[derive(Debug, PartialEq, Eq, TryFromPrimitive, Copy, Clone)]
#[repr(i32)]
pub enum UserBadges {
    Developer = 1,
    Translator = 2,
    Supporter = 4,
    ResponsibleDisclosure = 8,
    Founder = 16,
    PlatformModeration = 32,
    ActiveSupporter = 64,
    Paw = 128,
    EarlyAdopter = 256,
    ReservedRelevantJokeBadge1 = 512,
    ReservedRelevantJokeBadge2 = 1024,
}

#[derive(OptionalStruct, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[optional_derive(Serialize, Deserialize, Debug, Default, Clone)]
#[optional_name = "PartialUser"]
#[opt_skip_serializing_none]
#[opt_some_priority]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub username: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<File>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub relations: Vec<Relationship>,

    #[serde(skip_serializing_if = "if_zero_u32", default)]
    pub badges: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<UserStatus>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<UserProfile>,

    #[serde(skip_serializing_if = "if_zero_u32", default)]
    pub flags: u32,

    #[serde(skip_serializing_if = "if_false", default)]
    pub privileged: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot: Option<BotInformation>,

    pub online: bool,

    pub relationship: RelationshipStatus,
}

#[derive(Debug, PartialEq, Eq, TryFromPrimitive, Copy, Clone)]
#[repr(i32)]
pub enum UserFlags {
    Suspended = 1,
    Deleted = 2,
    Banned = 4,
    Spam = 8,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct BotInformation {
    pub owner: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, PartialEq, Eq, Clone)]
pub enum FieldsUser {
    Avatar,
    StatusText,
    StatusPresence,
    ProfileContent,
    ProfileBackground,
}

pub enum UserHint {
    Any,
    Bot,
    User,
}

pub fn if_zero_u32(t: &u32) -> bool {
    t == &0
}
