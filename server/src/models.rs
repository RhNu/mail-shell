use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InboundMetadata {
    pub envelope_to: String,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize, ToSchema)]
pub struct MessageSummary {
    pub id: String,
    pub from_name: Option<String>,
    pub from_address: String,
    pub to_name: Option<String>,
    pub to_address: Option<String>,
    pub envelope_to: String,
    pub subject: String,
    pub date: Option<String>,
    pub message_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MessageDetail {
    pub id: String,
    pub from_name: Option<String>,
    pub from_address: String,
    pub to_name: Option<String>,
    pub to_address: Option<String>,
    pub envelope_to: String,
    pub cc: Option<String>,
    pub reply_to: Option<String>,
    pub in_reply_to: Option<String>,
    pub subject: String,
    pub date: Option<String>,
    pub message_id: Option<String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize, ToSchema)]
pub struct AttachmentMeta {
    pub id: String,
    pub message_id: String,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub size: Option<i64>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AttachmentDownloadMeta {
    pub path: String,
    pub filename: Option<String>,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MessageRawMeta {
    pub raw_path: String,
    pub subject: String,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize, ToSchema)]
pub struct Tag {
    pub id: i64,
    pub kind: String,
    pub value: String,
    pub label: String,
    pub source: String,
    #[sqlx(default)]
    pub message_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MessageDetailResponse {
    #[serde(flatten)]
    pub message: MessageDetail,
    pub attachments: Vec<AttachmentMeta>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MessageListResponse {
    pub items: Vec<MessageSummary>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct InboundResponse {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct HeaderEntry {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MessageHeadersResponse {
    pub headers: Vec<HeaderEntry>,
}
