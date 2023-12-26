pub mod permission;
mod r#trait;
mod user;

use bson::Bson;
use serde::{Deserialize, Serialize};

pub use permission::*;
pub use r#trait::*;
pub use user::*;

#[derive(Deserialize, JsonSchema, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
pub struct Override {
    pub allow: u64,
    pub deny: u64,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Copy, Default)]
pub struct OverrideField {
    a: i64,
    d: i64,
}

impl Override {
    pub fn allows(&self) -> u64 {
        self.allow
    }

    pub fn denies(&self) -> u64 {
        self.deny
    }
}

impl From<Override> for OverrideField {
    fn from(v: Override) -> Self {
        Self {
            a: v.allow as i64,
            d: v.deny as i64,
        }
    }
}

impl From<OverrideField> for Override {
    fn from(v: OverrideField) -> Self {
        Self {
            allow: v.a as u64,
            deny: v.d as u64,
        }
    }
}

impl From<OverrideField> for Bson {
    fn from(v: OverrideField) -> Self {
        Self::Document(bson::to_document(&v).unwrap())
    }
}
