use crate::{
    models::{channel::ChannelType, user::RelationshipStatus},
    Override,
};

#[async_trait]
pub trait PermissionQuery {
    async fn are_we_privileged(&mut self) -> bool;
    async fn are_we_a_bot(&mut self) -> bool;
    async fn are_the_users_same(&mut self) -> bool;
    async fn user_relationship(&mut self) -> RelationshipStatus;
    async fn user_is_bot(&mut self) -> bool;
    async fn have_mutual_connection(&mut self) -> bool;
    async fn are_we_server_owner(&mut self) -> bool;
    async fn are_we_a_member(&mut self) -> bool;
    async fn get_default_server_permissions(&mut self) -> u64;
    async fn get_our_server_role_overrides(&mut self) -> Vec<Override>;
    async fn are_we_timed_out(&mut self) -> bool;
    async fn get_channel_type(&mut self) -> ChannelType;
    async fn get_default_channel_permissions(&mut self) -> Override;
    async fn get_our_channel_role_overrides(&mut self) -> Vec<Override>;
    async fn do_we_own_the_channel(&mut self) -> bool;
    async fn are_we_part_of_the_channel(&mut self) -> bool;
    async fn set_recipient_as_user(&mut self);
    async fn set_server_from_channel(&mut self);
}
