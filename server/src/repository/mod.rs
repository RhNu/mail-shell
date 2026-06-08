use async_trait::async_trait;

use crate::models::{AttachmentDownloadMeta, AttachmentMeta, MessageDetail, MessageRawMeta, MessageSummary, Tag};

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("database error: {0}")]
    Db(#[from] ::sqlx::Error),
}

#[derive(Debug, Clone)]
pub struct ListMessagesQuery {
    pub tag_id: Option<i64>,
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
    pub from_address: String,
    pub to_address: String,
    pub subject: Option<String>,
    pub date: Option<String>,
    pub message_id: Option<String>,
    pub raw_path: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
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

    async fn get_attachment_download(
        &self,
        id: &str,
    ) -> Result<Option<AttachmentDownloadMeta>, RepositoryError>;

    async fn get_message_raw(
        &self,
        id: &str,
    ) -> Result<Option<MessageRawMeta>, RepositoryError>;

    async fn list_tags(&self) -> Result<Vec<Tag>, RepositoryError>;
}

pub mod sqlx;
