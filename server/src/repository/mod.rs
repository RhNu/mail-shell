use async_trait::async_trait;

use crate::mime_parser::ParsedMailSnapshotV1;
use crate::models::{
    AttachmentDownloadMeta, AttachmentMeta, HeaderEntry, Mailbox, MessageDetail, MessageRawMeta,
    MessageSummary, Tag,
};

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("database error: {0}")]
    Db(#[from] ::sqlx::Error),
    #[error("message {message_id} contains invalid parsed snapshot data: {reason}")]
    InvalidSnapshotData { message_id: String, reason: String },
    #[error("message {message_id} contains invalid parsed snapshot JSON: {source}")]
    InvalidSnapshot {
        message_id: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("message {message_id} uses unsupported parsed snapshot version {version}")]
    UnsupportedSnapshotVersion { message_id: String, version: i64 },
}

#[derive(Debug, Clone)]
pub struct ListMessagesQuery {
    pub tag_id: Option<i64>,
    pub mailbox: Mailbox,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone)]
pub struct MessagePage<T> {
    pub items: Vec<T>,
    pub total: i64,
}

#[derive(Debug, Clone)]
pub struct MessageRecord {
    pub message: MessageDetail,
    pub attachments: Vec<AttachmentMeta>,
}

#[derive(Debug, Clone)]
pub struct DeletedMessageFiles {
    pub raw_path: String,
    pub attachment_paths: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct InboundAttachmentRecord {
    pub id: String,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub size: i64,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct InboundTagRecord {
    pub kind: String,
    pub value: String,
    pub label: String,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct InboundMessageRecord {
    pub id: String,
    pub message_id: Option<String>,
    pub subject: String,
    pub from_name: Option<String>,
    pub from_address: String,
    pub to_name: Option<String>,
    pub to_address: Option<String>,
    pub envelope_to: String,
    pub date: Option<String>,
    pub raw_path: String,
    pub snapshot: ParsedMailSnapshotV1,
    pub attachments: Vec<InboundAttachmentRecord>,
    pub tags: Vec<InboundTagRecord>,
}

#[async_trait]
pub trait Repository: Send + Sync {
    async fn ingest_message(&self, record: InboundMessageRecord) -> Result<(), RepositoryError>;

    async fn list_messages(
        &self,
        query: ListMessagesQuery,
    ) -> Result<MessagePage<MessageSummary>, RepositoryError>;

    async fn get_message(&self, id: &str) -> Result<Option<MessageRecord>, RepositoryError>;

    async fn update_message_mailbox(
        &self,
        id: &str,
        mailbox: Mailbox,
    ) -> Result<bool, RepositoryError>;

    async fn delete_message(
        &self,
        id: &str,
    ) -> Result<Option<DeletedMessageFiles>, RepositoryError>;

    async fn get_message_headers(
        &self,
        id: &str,
    ) -> Result<Option<Vec<HeaderEntry>>, RepositoryError>;

    async fn get_attachment_download(
        &self,
        id: &str,
    ) -> Result<Option<AttachmentDownloadMeta>, RepositoryError>;

    async fn get_message_raw(&self, id: &str) -> Result<Option<MessageRawMeta>, RepositoryError>;

    async fn list_tags(&self) -> Result<Vec<Tag>, RepositoryError>;
}

pub mod sqlx;
