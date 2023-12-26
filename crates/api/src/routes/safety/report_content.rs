use chat_core::events::client::EventV1;
use chat_core::models::report::{ReportStatus, ReportedContent};
use chat_core::models::snapshot::{Snapshot, SnapshotContent};
use chat_core::models::{Report, User};
use chat_core::{Db, Error, Result};
use serde::Deserialize;
use ulid::Ulid;
use validator::Validate;

use rocket::serde::json::Json;

#[derive(Validate, Deserialize, JsonSchema)]
pub struct DataReportContent {
    content: ReportedContent,

    #[validate(length(min = 0, max = 1000))]
    #[serde(default)]
    additional_context: String,
}

#[openapi(tag = "User Safety")]
#[post("/report", data = "<data>")]
pub async fn report_content(db: &Db, user: User, data: Json<DataReportContent>) -> Result<()> {
    let data = data.into_inner();
    let _ = data
        .validate()
        .map_err(|error| Error::FailedValidation { error });

    if user.bot.is_none() {
        return Err(Error::IsBot);
    }

    let (snapshots, files): (Vec<SnapshotContent>, Vec<String>) = match &data.content {
        ReportedContent::Message { id, .. } => {
            let message = db.fetch_message(id).await?;

            if message.author == user.id {
                return Err(Error::CannotReportYourself);
            }

            let (snapshot, files) = SnapshotContent::generate_from_message(db, message).await?;
            (vec![snapshot], files)
        }
        ReportedContent::Server { id, .. } => {
            let server = db.fetch_server(id).await?;

            if server.owner == user.id {
                return Err(Error::CannotReportYourself);
            }

            let (snapshot, files) = SnapshotContent::generate_from_server(server)?;
            (vec![snapshot], files)
        }
        ReportedContent::User { id, message_id, .. } => {
            let reported_user = db.fetch_user(id).await?;

            if reported_user.id == user.id {
                return Err(Error::CannotReportYourself);
            }

            let message = if let Some(id) = message_id {
                db.fetch_message(id).await.ok()
            } else {
                None
            };

            let (snapshot, files) = SnapshotContent::generate_from_user(reported_user)?;

            if let Some(message) = message {
                let (message_snapshot, message_files) =
                    SnapshotContent::generate_from_message(db, message).await?;
                (
                    vec![snapshot, message_snapshot],
                    [files, message_files].concat(),
                )
            } else {
                (vec![snapshot], files)
            }
        }
    };

    for file in files {
        db.mark_attachment_as_reported(&file).await?;
    }

    let id = Ulid::new().to_string();

    for content in snapshots {
        let snapshot = Snapshot {
            id: Ulid::new().to_string(),
            report_id: id.to_string(),
            content,
        };

        db.insert_snapshot(&snapshot).await?;
    }

    let report = Report {
        id,
        author_id: user.id,
        content: data.content,
        additional_context: data.additional_context,
        status: ReportStatus::Created {},
        notes: String::new(),
    };

    db.insert_report(&report).await?;

    EventV1::ReportCreate(report).global().await;

    Ok(())
}
