use crate::models::server_member::{FieldsMember, Member, MemberCompositeKey, PartialMember};
use crate::Result;

#[async_trait]
pub trait AbstractServerMember: Sync + Send {
    async fn fetch_member(&self, server: &str, user: &str) -> Result<Member>;
    async fn insert_member(&self, member: &Member) -> Result<()>;
    async fn update_member(
        &self,
        id: &MemberCompositeKey,
        member: &PartialMember,
        remove: Vec<FieldsMember>,
    ) -> Result<()>;
    async fn delete_member(&self, id: &MemberCompositeKey) -> Result<()>;
    async fn fetch_all_members<'a>(&self, server: &str) -> Result<Vec<Member>>;
    async fn fetch_all_memberships<'a>(&self, user: &str) -> Result<Vec<Member>>;
    async fn fetch_members<'a>(&self, server: &str, ids: &'a [String]) -> Result<Vec<Member>>;
    async fn fetch_member_count(&self, server: &str) -> Result<usize>;
    async fn fetch_server_count(&self, user: &str) -> Result<usize>;
}
