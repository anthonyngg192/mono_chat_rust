use num_enum::TryFromPrimitive;
use revolt_optional_struct::OptionalStruct;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Utility function to check if a boolean value is false
pub fn if_false(t: &bool) -> bool {
    !t
}

use crate::models::media::attachment::File;

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
pub enum RelationshipStatus {
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
    pub id: String,
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
pub enum Badges {
    /// Revolt Developer
    Developer = 1,
    /// Helped translate Revolt
    Translator = 2,
    /// Monetarily supported Revolt
    Supporter = 4,
    /// Responsibly disclosed a security issue
    ResponsibleDisclosure = 8,
    /// Revolt Founder
    Founder = 16,
    /// Platform moderator
    PlatformModeration = 32,
    /// Active monetary supporter
    ActiveSupporter = 64,
    /// 🦊🦝
    Paw = 128,
    /// Joined as one of the first 1000 users in 2021
    EarlyAdopter = 256,
    /// Amogus
    ReservedRelevantJokeBadge1 = 512,
    /// Low resolution troll face
    ReservedRelevantJokeBadge2 = 1024,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, OptionalStruct, Default)]
#[optional_derive(Serialize, Deserialize, Debug, Default, Clone)]
#[optional_name = "PartialUser"]
#[opt_skip_serializing_none]
#[opt_some_priority]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,

    pub username: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<File>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub relations: Option<Vec<Relationship>>,

    /// Bitfield of user badges
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badges: Option<i32>,
    /// User's current status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<UserStatus>,
    /// User's profile page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<UserProfile>,

    /// Enum of user flags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<i32>,
    /// Whether this user is privileged
    #[serde(skip_serializing_if = "if_false", default)]
    pub privileged: bool,
    /// Bot information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot: Option<BotInformation>,

    // ? Entries below should never be pushed to the database
    /// Current session user's relationship with this user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship: Option<RelationshipStatus>,
    /// Whether this user is currently online
    #[serde(skip_serializing_if = "Option::is_none")]
    pub online: Option<bool>,
}

/// User flag enum
#[derive(Debug, PartialEq, Eq, TryFromPrimitive, Copy, Clone)]
#[repr(i32)]
pub enum Flags {
    /// User has been suspended from the platform
    Suspended = 1,
    /// User has deleted their account
    Deleted = 2,
    /// User was banned off the platform
    Banned = 4,
    /// User was marked as spam and removed from platform
    Spam = 8,
}

/// Bot information for if the user is a bot
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct BotInformation {
    /// Id of the owner of this bot
    pub owner: String,
}

/// Optional fields on user object
#[derive(Serialize, Deserialize, JsonSchema, Debug, PartialEq, Eq, Clone)]
pub enum FieldsUser {
    Avatar,
    StatusText,
    StatusPresence,
    ProfileContent,
    ProfileBackground,
}

/// Enumeration providing a hint to the type of user we are handling
pub enum UserHint {
    /// Could be either a user or a bot
    Any,
    /// Only match bots
    Bot,
    /// Only match users
    User,
}
