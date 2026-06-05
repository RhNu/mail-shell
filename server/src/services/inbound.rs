use std::{path::PathBuf, sync::Arc};

use crate::{
    mime_parser::parse_message,
    models::{InboundMetadata, InboundResponse},
    repository::{
        InboundAttachmentRecord, InboundMessageRecord, InboundTagRecord, Repository,
        RepositoryError,
    },
    storage,
};

#[derive(Debug, thiserror::Error)]
pub enum InboundServiceError {
    #[error("storage error: {0}")]
    Io(#[from] std::io::Error),
    #[error("mail parse error: {0}")]
    MailParse(#[from] mailparse::MailParseError),
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

#[derive(Clone)]
pub struct InboundMessageService {
    repo: Arc<dyn Repository>,
    data_dir: PathBuf,
}

impl InboundMessageService {
    pub fn new(repo: Arc<dyn Repository>, data_dir: PathBuf) -> Self {
        Self { repo, data_dir }
    }

    #[tracing::instrument(skip(self, raw_mime, metadata))]
    pub async fn ingest(
        &self,
        raw_mime: Vec<u8>,
        metadata: InboundMetadata,
    ) -> Result<InboundResponse, InboundServiceError> {
        storage::ensure_dirs(&self.data_dir)?;

        let message_id = storage::generate_id();
        let mut written_paths = Vec::new();

        let result = async {
            let raw_path = storage::save_raw(&self.data_dir, &message_id, &raw_mime).await?;
            written_paths.push(raw_path.clone());

            let parsed = parse_message(&raw_mime)?;
            tracing::debug!(
                parsed_attachment_count = parsed.attachments.len(),
                has_text = parsed.body_text.is_some(),
                has_html = parsed.body_html.is_some(),
                "parsed inbound message"
            );

            let mut attachments = Vec::with_capacity(parsed.attachments.len());
            for attachment in parsed.attachments {
                let attachment_id = storage::generate_id();
                let path =
                    storage::save_attachment(&self.data_dir, &attachment_id, &attachment.body)
                        .await?;
                written_paths.push(path.clone());
                attachments.push(InboundAttachmentRecord {
                    id: attachment_id,
                    filename: attachment.filename,
                    content_type: Some(attachment.content_type),
                    size: attachment.body.len() as i64,
                    path: path.to_string_lossy().into_owned(),
                });
            }

            let tags = derive_tags(&metadata.to);
            tracing::debug!(derived_tag_count = tags.len(), "derived tags");

            let attachment_count = attachments.len();
            let tag_count = tags.len();
            let record = InboundMessageRecord {
                id: message_id.clone(),
                from_address: metadata.from,
                to_address: metadata.to.clone(),
                subject: Some(metadata.headers.subject),
                date: Some(metadata.headers.date),
                message_id: Some(metadata.headers.message_id),
                raw_path: raw_path.to_string_lossy().into_owned(),
                body_text: parsed.body_text,
                body_html: parsed.body_html,
                attachments,
                tags,
            };

            self.repo.ingest_message(record).await?;
            tracing::info!(
                msg_id = %message_id,
                attachment_count,
                tag_count,
                "inbound message ingested"
            );
            Ok::<InboundResponse, InboundServiceError>(InboundResponse { id: message_id })
        }
        .await;

        match result {
            Ok(response) => Ok(response),
            Err(err) => {
                cleanup_files(&written_paths).await;
                Err(err)
            }
        }
    }
}

fn derive_tags(to_address: &str) -> Vec<InboundTagRecord> {
    let normalized = to_address.trim().to_lowercase();
    let mut tags = vec![InboundTagRecord {
        kind: "recipient_address".to_string(),
        value: normalized.clone(),
        label: format!("To: {normalized}"),
        source: "system".to_string(),
    }];

    if let Some((_, domain)) = normalized.rsplit_once('@') {
        tags.push(InboundTagRecord {
            kind: "recipient_domain".to_string(),
            value: domain.to_string(),
            label: format!("Domain: {domain}"),
            source: "system".to_string(),
        });
    }

    tags
}

async fn cleanup_files(paths: &[PathBuf]) {
    for path in paths {
        match tokio::fs::remove_file(path).await {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => {
                tracing::warn!(path = %path.display(), error = %err, "failed to cleanup file")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::{ListMessagesQuery, sqlx::SqlxRepository};

    fn sample_metadata(message_id: &str) -> InboundMetadata {
        InboundMetadata {
            from: "sender@example.com".to_string(),
            to: "recipient@example.com".to_string(),
            headers: crate::models::InboundHeaders {
                message_id: message_id.to_string(),
                subject: "Hello".to_string(),
                date: "Mon, 01 Jan 2024 00:00:00 +0000".to_string(),
            },
        }
    }

    fn sample_raw_message() -> Vec<u8> {
        b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Hello\r\nContent-Type: multipart/mixed; boundary=\"boundary123\"\r\n\r\n--boundary123\r\nContent-Type: text/plain\r\n\r\nBody\r\n--boundary123\r\nContent-Type: text/plain\r\nContent-Disposition: attachment; filename=\"hello.txt\"\r\n\r\ntext\r\n--boundary123--".to_vec()
    }

    #[tokio::test]
    async fn ingest_persists_message_and_attachment_files() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = Arc::new(SqlxRepository::init_pool_in_memory().await.unwrap());
        let service = InboundMessageService::new(repo.clone(), temp_dir.path().to_path_buf());

        let response = service
            .ingest(sample_raw_message(), sample_metadata("<msg-1>"))
            .await
            .unwrap();

        let page = repo
            .list_messages(ListMessagesQuery {
                tag_id: None,
                limit: 20,
                offset: 0,
            })
            .await
            .unwrap();
        assert_eq!(page.total, 1);

        let detail = repo.get_message(&response.id).await.unwrap().unwrap();
        assert_eq!(detail.attachments.len(), 1);
        assert!(temp_dir.path().join("raw").is_dir());
        assert!(temp_dir.path().join("attachments").is_dir());
    }
}
