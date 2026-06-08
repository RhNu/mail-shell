use std::{path::PathBuf, sync::Arc};

use crate::{
    mime_parser::{MailAddress, parse_message},
    models::{InboundMetadata, InboundResponse},
    repository::{
        InboundAttachmentRecord, InboundMessageRecord, InboundTagRecord, Repository,
        RepositoryError,
    },
    services::notifier::{Notification, Notifier},
    storage,
};

#[derive(Debug, thiserror::Error)]
pub enum InboundServiceError {
    #[error("storage error: {0}")]
    Io(#[from] std::io::Error),
    #[error("mail parse error: {0}")]
    MailParse(#[from] crate::mime_parser::ParseError),
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

#[derive(Clone)]
pub struct InboundMessageService {
    repo: Arc<dyn Repository>,
    data_dir: PathBuf,
    notifier: Arc<dyn Notifier>,
}

impl InboundMessageService {
    pub fn new(repo: Arc<dyn Repository>, data_dir: PathBuf, notifier: Arc<dyn Notifier>) -> Self {
        Self {
            repo,
            data_dir,
            notifier,
        }
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
                subject = %parsed.subject,
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

            let tags = derive_tags(&metadata.envelope_to, &parsed.from);
            tracing::debug!(derived_tag_count = tags.len(), "derived tags");

            let attachment_count = attachments.len();
            let tag_count = tags.len();

            let notify_from = parsed.from.first().map(|a| a.display()).unwrap_or_default();
            let notify_subject = parsed.subject.clone();

            let (from_name, from_address) = {
                let (n, a) = first_address_fields(&parsed.from);
                (n, a.unwrap_or_default())
            };
            let (to_name, to_address) = first_address_fields(&parsed.to);

            let record = InboundMessageRecord {
                id: message_id.clone(),
                message_id: parsed.message_id,
                subject: parsed.subject,
                from_name,
                from_address,
                to_name,
                to_address,
                envelope_to: metadata.envelope_to,
                cc: addresses_to_json(&parsed.cc),
                reply_to: addresses_to_json(&parsed.reply_to),
                in_reply_to: parsed.in_reply_to,
                date: parsed.date,
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

            let notifier = self.notifier.clone();
            let msg_id_for_notify = message_id.clone();
            tokio::spawn(async move {
                let notification = Notification {
                    title: "New Mail".to_string(),
                    body: format!("From: {notify_from}\nSubject: {notify_subject}"),
                    group: Some("mail-shell".to_string()),
                    url: None,
                };
                match notifier.notify(notification).await {
                    Ok(()) => tracing::debug!(msg_id = %msg_id_for_notify, "notification sent"),
                    Err(err) => tracing::warn!(msg_id = %msg_id_for_notify, error = %err, "notification failed"),
                }
            });

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

fn first_address_fields(addrs: &[MailAddress]) -> (Option<String>, Option<String>) {
    match addrs.first() {
        Some(a) => (a.name.clone(), a.address.clone()),
        None => (None, None),
    }
}

fn addresses_to_json(addrs: &[MailAddress]) -> Option<String> {
    if addrs.is_empty() {
        return None;
    }
    serde_json::to_string(addrs).ok()
}

fn derive_tags(envelope_to: &str, from: &[MailAddress]) -> Vec<InboundTagRecord> {
    let normalized = envelope_to.trim().to_lowercase();
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

    if let Some(from_addr) = from.first()
        && let Some(addr) = &from_addr.address
    {
        let from_normalized = addr.trim().to_lowercase();
        if let Some((_, domain)) = from_normalized.rsplit_once('@') {
            tags.push(InboundTagRecord {
                kind: "sender_domain".to_string(),
                value: domain.to_string(),
                label: format!("From domain: {domain}"),
                source: "system".to_string(),
            });
        }
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
    use crate::services::notifier::NoopNotifier;

    fn sample_metadata() -> InboundMetadata {
        InboundMetadata {
            envelope_to: "recipient@example.com".to_string(),
        }
    }

    fn sample_raw_message() -> Vec<u8> {
        b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Hello\r\nContent-Type: multipart/mixed; boundary=\"boundary123\"\r\n\r\n--boundary123\r\nContent-Type: text/plain\r\n\r\nBody\r\n--boundary123\r\nContent-Type: text/plain\r\nContent-Disposition: attachment; filename=\"hello.txt\"\r\n\r\ntext\r\n--boundary123--".to_vec()
    }

    #[tokio::test]
    async fn ingest_persists_message_and_attachment_files() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = Arc::new(SqlxRepository::init_pool_in_memory().await.unwrap());
        let notifier = Arc::new(NoopNotifier) as Arc<dyn Notifier>;
        let service =
            InboundMessageService::new(repo.clone(), temp_dir.path().to_path_buf(), notifier);

        let response = service
            .ingest(sample_raw_message(), sample_metadata())
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
        assert_eq!(page.items[0].subject, "Hello");
        assert_eq!(page.items[0].from_address, "sender@example.com");
        assert_eq!(page.items[0].envelope_to, "recipient@example.com");

        let detail = repo.get_message(&response.id).await.unwrap().unwrap();
        assert_eq!(detail.attachments.len(), 1);
        assert!(temp_dir.path().join("raw").is_dir());
        assert!(temp_dir.path().join("attachments").is_dir());
    }

    #[tokio::test]
    async fn ingest_real_eml_decodes_subject() {
        let raw = b"Date: Mon, 08 Jun 2026 06:39:51 +0000\r\nFrom: Discord <noreply@discord.com>\r\nTo: discord+rhnu@rhnu.org\r\nMessage-ID: <verify@example.com>\r\nSubject: =?UTF-8?B?6aqM6K+BIERpc2NvcmQg55qE55S15a2Q6YKu5Lu25Zyw5Z2A?=\r\nContent-Type: text/html; charset=utf-8\r\n\r\n<p>Verify your address.</p>".to_vec();
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = Arc::new(SqlxRepository::init_pool_in_memory().await.unwrap());
        let notifier = Arc::new(NoopNotifier) as Arc<dyn Notifier>;
        let service =
            InboundMessageService::new(repo.clone(), temp_dir.path().to_path_buf(), notifier);

        let metadata = InboundMetadata {
            envelope_to: "discord+rhnu@rhnu.org".to_string(),
        };

        let response = service.ingest(raw, metadata).await.unwrap();

        let detail = repo.get_message(&response.id).await.unwrap().unwrap();
        assert_eq!(detail.message.subject, "验证 Discord 的电子邮件地址");
        assert_eq!(detail.message.from_name.as_deref(), Some("Discord"));
        assert_eq!(detail.message.from_address, "noreply@discord.com");
        assert_eq!(detail.message.envelope_to, "discord+rhnu@rhnu.org");
        assert!(detail.message.body_html.is_some());
    }

    #[tokio::test]
    async fn derive_tags_includes_sender_domain() {
        let tags = derive_tags(
            "to@example.com",
            &[MailAddress {
                name: Some("Discord".to_string()),
                address: Some("noreply@discord.com".to_string()),
            }],
        );
        assert_eq!(tags.len(), 3);
        assert_eq!(tags[0].kind, "recipient_address");
        assert_eq!(tags[1].kind, "recipient_domain");
        assert_eq!(tags[2].kind, "sender_domain");
        assert_eq!(tags[2].value, "discord.com");
    }
}
