use crate::models::Snapshot;
use crate::Result;

#[async_trait]
pub trait AbstractSnapshot: Sync + Send {
    async fn insert_snapshot(&self, snapshot: &Snapshot) -> Result<()>;
    async fn fetch_snapshot(&self, report_id: &str) -> Result<Snapshot>;
}
