use crate::models::report::PartialReport;
use crate::models::Report;
use crate::Result;

#[async_trait]
pub trait AbstractReport: Sync + Send {
    async fn insert_report(&self, report: &Report) -> Result<()>;
    async fn update_report(&self, id: &str, message: &PartialReport) -> Result<()>;
    async fn fetch_report(&self, report_id: &str) -> Result<Report>;
    async fn fetch_reports(&self) -> Result<Vec<Report>>;
}
