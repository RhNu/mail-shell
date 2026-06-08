use std::sync::Arc;

use mail_shell_server::models::InboundMetadata;
use mail_shell_server::repository::{
    InboundAttachmentRecord, InboundMessageRecord, InboundTagRecord, ListMessagesQuery, Repository,
    sqlx::SqlxRepository,
};
use mail_shell_server::services::inbound::InboundMessageService;
use mail_shell_server::services::notifier::NoopNotifier;

fn sample_metadata() -> InboundMetadata {
    InboundMetadata {
        envelope_to: "recipient@example.com".to_string(),
    }
}

fn sample_record(id: &str, attachment_id: &str, message_id: &str) -> InboundMessageRecord {
    InboundMessageRecord {
        id: id.to_string(),
        message_id: Some(message_id.to_string()),
        subject: "Hello".to_string(),
        from_name: None,
        from_address: "sender@example.com".to_string(),
        to_name: None,
        to_address: Some("recipient@example.com".to_string()),
        envelope_to: "recipient@example.com".to_string(),
        cc: None,
        reply_to: None,
        in_reply_to: None,
        date: Some("2024-01-01T00:00:00+00:00".to_string()),
        raw_path: format!("/tmp/{id}.eml"),
        body_text: Some("Body".to_string()),
        body_html: None,
        attachments: vec![InboundAttachmentRecord {
            id: attachment_id.to_string(),
            filename: Some("hello.txt".to_string()),
            content_type: Some("text/plain".to_string()),
            size: 4,
            path: format!("/tmp/{attachment_id}.bin"),
        }],
        tags: vec![
            InboundTagRecord {
                kind: "recipient_address".to_string(),
                value: "recipient@example.com".to_string(),
                label: "To: recipient@example.com".to_string(),
                source: "system".to_string(),
            },
            InboundTagRecord {
                kind: "recipient_domain".to_string(),
                value: "example.com".to_string(),
                label: "Domain: example.com".to_string(),
                source: "system".to_string(),
            },
        ],
    }
}

#[tokio::test]
async fn aggregate_ingest_persists_message_graph() {
    let repo = SqlxRepository::init_pool_in_memory().await.unwrap();

    repo.ingest_message(sample_record("msg-1", "att-1", "<msg-1>"))
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
    assert_eq!(page.items.len(), 1);

    let detail = repo.get_message("msg-1").await.unwrap().unwrap();
    assert_eq!(detail.message.id, "msg-1");
    assert_eq!(detail.attachments.len(), 1);

    let tags = repo.list_tags().await.unwrap();
    assert_eq!(tags.len(), 2);
}

#[tokio::test]
async fn aggregate_ingest_rolls_back_on_duplicate_message_id() {
    let repo = SqlxRepository::init_pool_in_memory().await.unwrap();

    repo.ingest_message(sample_record("msg-1", "att-1", "<dup>"))
        .await
        .unwrap();

    let err = repo
        .ingest_message(sample_record("msg-2", "att-2", "<dup>"))
        .await;
    assert!(err.is_err());

    let page = repo
        .list_messages(ListMessagesQuery {
            tag_id: None,
            limit: 20,
            offset: 0,
        })
        .await
        .unwrap();
    assert_eq!(page.total, 1);
    assert!(repo.get_message("msg-2").await.unwrap().is_none());
    assert!(
        repo.get_attachment_download("att-2")
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn inbound_service_cleans_up_files_when_repository_write_fails() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Arc::new(SqlxRepository::init_pool_in_memory().await.unwrap());
    let service = InboundMessageService::new(
        repo.clone(),
        temp_dir.path().to_path_buf(),
        Arc::new(NoopNotifier),
    );

    let raw = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Hello\r\nMessage-ID: <test-dup-msg-id>\r\nContent-Type: multipart/mixed; boundary=\"boundary123\"\r\n\r\n--boundary123\r\nContent-Type: text/plain\r\n\r\nBody\r\n--boundary123\r\nContent-Type: text/plain\r\nContent-Disposition: attachment; filename=\"hello.txt\"\r\n\r\ntext\r\n--boundary123--";

    service
        .ingest(raw.to_vec(), sample_metadata())
        .await
        .unwrap();

    let second = service.ingest(raw.to_vec(), sample_metadata()).await;
    assert!(second.is_err());

    let raw_files = std::fs::read_dir(temp_dir.path().join("raw"))
        .unwrap()
        .count();
    let attachment_files = std::fs::read_dir(temp_dir.path().join("attachments"))
        .unwrap()
        .count();

    assert_eq!(raw_files, 1);
    assert_eq!(attachment_files, 1);
}
