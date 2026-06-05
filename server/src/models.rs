use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Metadata accompanying an inbound email submission via multipart form.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundMetadata {
    pub from: String,
    pub to: String,
    pub headers: InboundHeaders,
}

/// Key email headers extracted from inbound metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundHeaders {
    #[serde(rename = "message-id")]
    pub message_id: String,
    pub subject: String,
    pub date: String,
}

/// Summary view of a message for list endpoints.
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct MessageSummary {
    pub id: String,
    pub from_address: String,
    pub to_address: String,
    pub subject: Option<String>,
    pub date: Option<String>,
    pub message_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Full message detail including body content.
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct MessageDetail {
    pub id: String,
    pub from_address: String,
    pub to_address: String,
    pub subject: Option<String>,
    pub date: Option<String>,
    pub message_id: Option<String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Metadata for a single attachment belonging to a message.
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct AttachmentMeta {
    pub id: String,
    pub message_id: String,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub size: Option<i64>,
}

/// Metadata needed to download an attachment (path + headers).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AttachmentDownloadMeta {
    pub path: String,
    pub filename: Option<String>,
    pub content_type: Option<String>,
}

/// A tag that can be associated with messages.
///
/// `message_count` is populated by aggregate queries and defaults to `None`
/// when loaded directly from the `tags` table.
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Tag {
    pub id: i64,
    pub kind: String,
    pub value: String,
    pub label: String,
    pub source: String,
    #[sqlx(default)]
    pub message_count: Option<i64>,
}

/// Response wrapper that combines a message with its attachments.
#[derive(Debug, Clone, Serialize)]
pub struct MessageDetailResponse {
    #[serde(flatten)]
    pub message: MessageDetail,
    pub attachments: Vec<AttachmentMeta>,
}

/// Generic pagination envelope for list responses.
#[derive(Debug, Clone, Serialize)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
}

/// Response returned after successfully ingesting an inbound email.
#[derive(Debug, Clone, Serialize)]
pub struct InboundResponse {
    pub id: String,
}
