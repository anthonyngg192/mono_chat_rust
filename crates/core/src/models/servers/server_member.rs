use iso8601_timestamp::Timestamp;
use revolt_optional_struct::OptionalStruct;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{auto_derived, models::File};

#[derive(Serialize, Deserialize, Debug, Clone, OptionalStruct)]
#[optional_derive(Serialize, Deserialize, JsonSchema, Debug, Default, Clone)]
#[optional_name = "PartialMember"]
#[opt_skip_serializing_none]
#[opt_some_priority]
pub struct Member {
    #[serde(rename = "_id")]
    pub id: MemberCompositeKey,

    pub joined_at: Timestamp,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<File>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub roles: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<Timestamp>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Default)]
pub struct MemberCompositeKey {
    pub server: String,
    pub user: String,
}

auto_derived!(
    pub enum FieldsMember {
        Nickname,
        Avatar,
        Roles,
        Timeout,
    }

    pub enum RemovalIntention {
        Leave,
        Kick,
        Ban,
    }
);
