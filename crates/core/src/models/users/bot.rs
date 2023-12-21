use num_enum::TryFromPrimitive;
use revolt_optional_struct::OptionalStruct;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::models::User;
pub fn if_false(t: &bool) -> bool {
    !t
}

#[derive(Debug, PartialEq, Eq, TryFromPrimitive, Clone, Copy)]
#[repr(i32)]
pub enum BogFlags {
    Verified = 1,
    Official = 2,
}

#[derive(Serialize, Deserialize, Debug, Clone, OptionalStruct, Default)]
#[optional_derive(Serialize, Deserialize, JsonSchema, Debug, Default, Clone)]
#[optional_name = "PartialBot"]
#[opt_skip_serializing_none]
#[opt_some_priority]
pub struct Bot {
    #[serde(rename = "_id")]
    pub id: String,

    pub owner: String,
    pub token: String,
    pub public: bool,

    #[serde(skip_serializing_if = "if_false", default)]
    pub analytics: bool,

    #[serde(skip_serializing_if = "if_false", default)]
    pub discoverable: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub interactions_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms_of_service_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy_policy_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<i32>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, PartialEq, Eq)]
pub enum FieldsBot {
    Token,
    InteractionsUrl,
}

#[repr(u32)]
pub enum BotFlags {
    Verified = 1,
    Official = 2,
}

pub struct PublicBot {
    #[cfg_attr(feature = "serde", serde(rename = "_id"))]
    pub id: String,

    pub username: String,

    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "String::is_empty", default)
    )]
    pub avatar: String,

    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "String::is_empty", default)
    )]
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FetchBotResponse {
    pub bot: Bot,
    pub user: User,
}

#[derive(Default)]
#[cfg_attr(feature = "validator", derive(validator::Validate))]
pub struct DataCreateBot {
    #[cfg_attr(
        feature = "validator",
        validate(length(min = 2, max = 32), regex = "super::RE_USERNAME")
    )]
    pub name: String,
}

#[derive(Default, Serialize)]
#[cfg_attr(feature = "validator", derive(validator::Validate))]
pub struct DataEditBot {
    #[cfg_attr(
        feature = "validator",
        validate(length(min = 2, max = 32), regex = "super::RE_USERNAME")
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    pub public: Option<bool>,

    pub analytics: Option<bool>,

    #[cfg_attr(feature = "validator", validate(length(min = 1, max = 2048)))]
    pub interactions_url: Option<String>,

    #[cfg_attr(feature = "validator", validate(length(min = 1)))]
    pub remove: Option<Vec<FieldsBot>>,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum InviteBotDestination {
    Server { server: String },
    Group { group: String },
}

pub struct OwnedBotsResponse {
    pub bots: Vec<Bot>,
    pub users: Vec<User>,
}
