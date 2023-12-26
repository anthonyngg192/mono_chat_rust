use crate::{Error, Result};

use super::defn::{ChannelPermission, Override, UserPermission};

pub mod permission;
pub mod user;

#[derive(Clone, Debug)]
pub struct PermissionValue(u64);

impl PermissionValue {
    pub fn apply(&mut self, v: Override) {
        self.allow(v.allow);
        self.revoke(v.deny);
    }

    pub fn allow(&mut self, v: u64) {
        self.0 |= v;
    }

    pub fn revoke(&mut self, v: u64) {
        self.0 &= !v;
    }

    pub fn revoke_all(&mut self) {
        self.0 = 0;
    }

    pub fn restrict(&mut self, v: u64) {
        self.0 &= v;
    }

    pub fn has(&self, v: u64) -> bool {
        (self.0 & v) == v
    }

    pub fn has_user_permission(&self, permission: UserPermission) -> bool {
        self.has(permission as u64)
    }

    pub fn has_channel_permission(&self, permission: ChannelPermission) -> bool {
        self.has(permission as u64)
    }

    pub fn throw_if_lacking_channel_permission(&self, permission: ChannelPermission) -> Result<()> {
        if self.has_channel_permission(permission) {
            Ok(())
        } else {
            Err(Error::MissingPermission { permission })
        }
    }
}

impl From<i64> for PermissionValue {
    fn from(v: i64) -> Self {
        Self(v as u64)
    }
}

impl From<u64> for PermissionValue {
    fn from(v: u64) -> Self {
        Self(v)
    }
}

impl From<PermissionValue> for u64 {
    fn from(v: PermissionValue) -> Self {
        v.0
    }
}
