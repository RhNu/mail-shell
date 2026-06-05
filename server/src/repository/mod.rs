use async_trait::async_trait;

use crate::error::AppError;
use crate::models::{AttachmentDownloadMeta, AttachmentMeta, MessageDetail, MessageSummary, Tag};

#[async_trait]
pub trait Repository: Send + Sync {
    /// Count total messages, optionally filtered by tag.
    async fn count_messages(&self, tag_id: Option<i64>) -> Result<i64, AppError>;

    /// List messages with optional tag filtering and pagination.
    async fn list_messages(
        &self,
        tag_id: Option<i64>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<MessageSummary>, AppError>;

    /// Retrieve a single message detail by ID.
    async fn get_message_detail(&self, id: &str) -> Result<Option<MessageDetail>, AppError>;

    /// List attachments belonging to a message.
    async fn list_attachments_by_message(
        &self,
        message_id: &str,
    ) -> Result<Vec<AttachmentMeta>, AppError>;

    /// Get attachment metadata (path, filename, content_type) by ID.
    async fn get_attachment_meta(
        &self,
        id: &str,
    ) -> Result<Option<AttachmentDownloadMeta>, AppError>;

    /// Insert a new message.
    #[allow(clippy::too_many_arguments)]
    async fn create_message(
        &self,
        id: &str,
        from_address: &str,
        to_address: &str,
        subject: Option<&str>,
        date: Option<&str>,
        message_id: Option<&str>,
        raw_path: &str,
        body_text: Option<&str>,
        body_html: Option<&str>,
    ) -> Result<(), AppError>;

    /// Insert a new attachment.
    async fn create_attachment(
        &self,
        id: &str,
        message_id: &str,
        filename: Option<&str>,
        content_type: Option<&str>,
        size: i64,
        path: &str,
    ) -> Result<(), AppError>;

    /// Ensure a tag exists, creating it if necessary. Returns the tag ID.
    async fn ensure_tag(&self, kind: &str, value: &str, label: &str) -> Result<i64, AppError>;

    /// Link a message to a tag.
    async fn link_message_tag(&self, message_id: &str, tag_id: i64) -> Result<(), AppError>;

    /// List all tags with associated message counts.
    async fn list_tags(&self) -> Result<Vec<Tag>, AppError>;
}

pub mod sqlx;
