use revolt_optional_struct::OptionalStruct;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub enum ContentReportReason {
    NoneSpecified,
    Illegal,
    PromotesHarm,
    SpamAbuse,
    Malware,
    Harassment,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub enum UserReportReason {
    NoneSpecified,
    SpamAbuse,
    InappropriateProfile,
    Impersonation,
    BanEvasion,
    Underage,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(tag = "type")]
pub enum ReportedContent {
    Message {
        id: String,
        report_reason: ContentReportReason,
    },
    Server {
        id: String,
        report_reason: ContentReportReason,
    },
    User {
        id: String,
        report_reason: UserReportReason,
    },
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
#[serde(tag = "status")]
pub enum ReportStatus {
    Created {},
    Rejected { rejection_reason: String },
    Resolved {},
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, OptionalStruct, Clone)]
#[optional_derive(Serialize, Deserialize, JsonSchema, Debug, Default, Clone)]
#[optional_name = "PartialReport"]
#[opt_skip_serializing_none]
pub struct Report {
    #[serde(rename = "_id")]
    pub id: String,
    pub author_id: String,
    pub content: ReportedContent,
    pub additional_context: String,

    #[serde(flatten)]
    pub status: ReportStatus,

    #[serde(default)]
    pub notes: String,
}
