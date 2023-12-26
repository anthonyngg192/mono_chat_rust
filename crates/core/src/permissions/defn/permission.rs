use num_enum::TryFromPrimitive;
use once_cell::sync::Lazy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    ops::{self, Add},
};

use crate::permissions::r#impl::PermissionValue;

#[derive(
    Serialize, Deserialize, Debug, PartialEq, Eq, TryFromPrimitive, Copy, Clone, JsonSchema,
)]
#[repr(u64)]
pub enum ChannelPermission {
    ManageChannel = 1 << 0,
    ManageServer = 1 << 1,
    ManagePermissions = 1 << 2,
    ManageRole = 1 << 3,
    ManageCustomisation = 1 << 4,
    KickMembers = 1 << 6,
    BanMembers = 1 << 7,
    TimeoutMembers = 1 << 8,
    AssignRoles = 1 << 9,
    ChangeNickname = 1 << 10,
    ManageNicknames = 1 << 11,
    ChangeAvatar = 1 << 12,
    RemoveAvatars = 1 << 13,
    ViewChannel = 1 << 20,
    ReadMessageHistory = 1 << 21,
    SendMessage = 1 << 22,
    ManageMessages = 1 << 23,
    ManageWebhooks = 1 << 24,
    InviteOthers = 1 << 25,
    SendEmbeds = 1 << 26,
    UploadFiles = 1 << 27,
    Masquerade = 1 << 28,
    React = 1 << 29,
    Connect = 1 << 30,
    Speak = 1 << 31,
    Video = 1 << 32,
    MuteMembers = 1 << 33,
    DeafenMembers = 1 << 34,
    MoveMembers = 1 << 35,
    GrantAllSafe = 0x000F_FFFF_FFFF_FFFF,
    GrantAll = u64::MAX,
}

impl_op_ex!(+ |a: &ChannelPermission, b: &ChannelPermission| -> u64 { *a as u64 | *b as u64 });
impl_op_ex_commutative!(+ |a: &u64, b: &ChannelPermission| -> u64 { *a | *b as u64 });

pub static ALLOW_IN_TIMEOUT: Lazy<u64> =
    Lazy::new(|| ChannelPermission::ViewChannel + ChannelPermission::ReadMessageHistory);
pub static DEFAULT_PERMISSION_VIEW_ONLY: Lazy<u64> =
    Lazy::new(|| ChannelPermission::ViewChannel + ChannelPermission::ReadMessageHistory);
pub static DEFAULT_PERMISSION: Lazy<u64> = Lazy::new(|| {
    DEFAULT_PERMISSION_VIEW_ONLY.add(
        ChannelPermission::SendMessage
            + ChannelPermission::InviteOthers
            + ChannelPermission::SendEmbeds
            + ChannelPermission::UploadFiles
            + ChannelPermission::Connect
            + ChannelPermission::Speak,
    )
});
pub static DEFAULT_PERMISSION_SAVED_MESSAGES: u64 = ChannelPermission::GrantAllSafe as u64;
pub static DEFAULT_PERMISSION_DIRECT_MESSAGE: Lazy<u64> = Lazy::new(|| {
    DEFAULT_PERMISSION.add(ChannelPermission::ManageChannel + ChannelPermission::React)
});
pub static DEFAULT_PERMISSION_SERVER: Lazy<u64> = Lazy::new(|| {
    DEFAULT_PERMISSION.add(
        ChannelPermission::React
            + ChannelPermission::ChangeNickname
            + ChannelPermission::ChangeAvatar,
    )
});

bitfield! {
    #[derive(Default)]
    pub struct Permissions(MSB0 [u64]);
    u64;

    // * Server permissions
    pub can_manage_channel, _: 63;
    pub can_manage_server, _: 62;
    pub can_manage_permissions, _: 61;
    pub can_manage_roles, _: 60;
    pub can_manage_customisation, _: 59;

    // * Member permissions
    pub can_kick_members, _: 57;
    pub can_ban_members, _: 56;
    pub can_timeout_members, _: 55;
    pub can_assign_roles, _: 54;
    pub can_change_nickname, _: 53;
    pub can_manage_nicknames, _: 52;
    pub can_change_avatar, _: 51;
    pub can_remove_avatars, _: 50;

    // * Channel permissions
    pub can_view_channel, _: 42;
    pub can_read_message_history, _: 41;
    pub can_send_message, _: 40;
    pub can_manage_messages, _: 39;
    pub can_manage_webhooks, _: 38;
    pub can_invite_others, _: 37;
    pub can_send_embeds, _: 36;
    pub can_upload_files, _: 35;
    pub can_masquerade, _: 34;

    // * Voice permissions
    pub can_connect, _: 32;
    pub can_speak, _: 31;
    pub can_share_video, _: 30;
    pub can_mute_members, _: 29;
    pub can_deafen_members, _: 28;
    pub can_move_members, _: 27;
}

pub type Perms = Permissions<[u64; 1]>;

impl From<ChannelPermission> for PermissionValue {
    fn from(v: ChannelPermission) -> Self {
        (v as u64).into()
    }
}

impl fmt::Display for ChannelPermission {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

pub static DEFAULT_WEBHOOK_PERMISSIONS: Lazy<u64> = Lazy::new(|| {
    ChannelPermission::SendMessage
        + ChannelPermission::SendEmbeds
        + ChannelPermission::Masquerade
        + ChannelPermission::React
});
