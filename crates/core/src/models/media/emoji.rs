use once_cell::sync::Lazy;
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

pub fn if_false(t: &bool) -> bool {
    !t
}

pub static RE_EMOJI: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-z0-9_]+$").unwrap());

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(tag = "type")]
pub enum EmojiParent {
    Server { id: String },
    Detached,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct Emoji {
    #[serde(rename = "_id")]
    pub id: String,
    pub parent: EmojiParent,
    pub creator_id: String,
    pub name: String,

    #[serde(skip_serializing_if = "if_false", default)]
    pub animated: bool,

    #[serde(skip_serializing_if = "if_false", default)]
    pub nsfw: bool,
}

#[derive(Serialize, Deserialize, Validate, JsonSchema)]
pub struct DataCreateEmoji {
    #[validate(length(min = 1, max = 32), regex = "RE_EMOJI")]
    pub name: String,
    pub parent: EmojiParent,

    #[serde(default)]
    pub nsfw: bool,
}
