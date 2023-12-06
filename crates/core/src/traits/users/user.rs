use crate::models::user::{FieldsUser, PartialUser, RelationshipStatus, User};
use crate::Result;

#[async_trait]
pub trait AbstractUser: Sync + Send {
    async fn fetch_user(&self, id: &str) -> Result<User>;
    async fn fetch_user_by_username(&self, username: &str) -> Result<User>;
    async fn fetch_user_by_token(&self, token: &str) -> Result<User>;
    async fn insert_user(&self, user: &User) -> Result<()>;
    async fn update_user(
        &self,
        id: &str,
        user: &PartialUser,
        remove: Vec<FieldsUser>,
    ) -> Result<()>;
    async fn delete_user(&self, id: &str) -> Result<()>;
    async fn fetch_users<'a>(&self, ids: &'a [String]) -> Result<Vec<User>>;
    async fn is_username_taken(&self, username: &str) -> Result<bool>;
    async fn fetch_mutual_user_ids(&self, user_a: &str, user_b: &str) -> Result<Vec<String>>;
    async fn fetch_mutual_channel_ids(&self, user_a: &str, user_b: &str) -> Result<Vec<String>>;
    async fn fetch_mutual_server_ids(&self, user_a: &str, user_b: &str) -> Result<Vec<String>>;
    async fn set_relationship(
        &self,
        user_id: &str,
        target_id: &str,
        relationship: &RelationshipStatus,
    ) -> Result<()>;
    async fn pull_relationship(&self, user_id: &str, target_id: &str) -> Result<()>;
}
