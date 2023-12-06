use crate::models::server_member::MemberCompositeKey;
use crate::models::ServerBan;
use crate::Result;

#[async_trait]
pub trait AbstractServerBan: Sync + Send {
    async fn fetch_ban(&self, server: &str, user: &str) -> Result<ServerBan>;
    async fn fetch_bans(&self, server: &str) -> Result<Vec<ServerBan>>;
    async fn insert_ban(&self, ban: &ServerBan) -> Result<()>;
    async fn delete_ban(&self, id: &MemberCompositeKey) -> Result<()>;
}
