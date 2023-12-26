use crate::models::User;
use num_enum::TryFromPrimitive;
use once_cell::sync::Lazy;
use regex::Regex;
use revolt_optional_struct::OptionalStruct;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

pub static RE_USERNAME: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\p{L}|[\d_.-])+$").unwrap());

pub fn if_false(t: &bool) -> bool {
    !t
}

#[derive(Debug, PartialEq, Eq, TryFromPrimitive, Clone, Copy)]
#[repr(i32)]
pub enum BogFlags {
    Verified = 1,
    Official = 2,
}

#[derive(Serialize, Deserialize, Debug, Clone, OptionalStruct, Default, JsonSchema)]
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

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct PublicBot {
    #[serde(rename = "_id")]
    pub id: String,

    pub username: String,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub avatar: String,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct FetchBotResponse {
    pub bot: Bot,
    pub user: User,
}

#[derive(Serialize, Deserialize, Default, Validate, JsonSchema)]
pub struct DataCreateBot {
    #[validate(length(min = 1, max = 128), regex = "RE_USERNAME")]
    pub name: String,
}

#[derive(Deserialize, Default, Serialize, Validate, JsonSchema)]
pub struct DataEditBot {
    #[validate(length(min = 1, max = 32), regex = "RE_USERNAME")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    pub public: Option<bool>,

    pub analytics: Option<bool>,

    #[validate(length(min = 1, max = 2048))]
    pub interactions_url: Option<String>,

    #[validate(length(min = 1))]
    pub remove: Option<Vec<FieldsBot>>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
#[serde(untagged)]
pub enum InviteBotDestination {
    Server { server: String },
    Group { group: String },
}
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct OwnedBotsResponse {
    pub bots: Vec<Bot>,
    pub users: Vec<User>,
}
