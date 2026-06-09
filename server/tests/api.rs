use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use mail_shell_server::repository::sqlx::SqlxRepository;
use mail_shell_server::routes::{AppState, router};
use mail_shell_server::services::inbound::InboundMessageService;
use mail_shell_server::services::notifier::{NoopNotifier, Notifier};
use mail_shell_server::storage;
use std::path::PathBuf;
use tower::ServiceExt;

fn build_multipart_body(raw_mime: &[u8], metadata: &str) -> (Vec<u8>, String) {
    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    let mut body = Vec::new();
    body.extend_from_slice(
        format!("--{boundary}\r\nContent-Disposition: form-data; name=\"raw_mime\"; filename=\"message.eml\"\r\nContent-Type: message/rfc822\r\n\r\n").as_bytes(),
    );
    body.extend_from_slice(raw_mime);
    body.extend_from_slice(
        format!("\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"metadata\"\r\n\r\n{metadata}\r\n--{boundary}--\r\n").as_bytes(),
    );
    (body, boundary.to_string())
}

async fn setup() -> (AppState, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let data_dir = tmp.path().to_path_buf();
    storage::ensure_dirs(&data_dir).unwrap();
    let repo = Arc::new(SqlxRepository::init_pool(&data_dir).await.unwrap());
    let notifier = Arc::new(NoopNotifier) as Arc<dyn Notifier>;
    let state = AppState {
        inbound_service: Arc::new(InboundMessageService::new(
            repo.clone(),
            data_dir.clone(),
            notifier.clone(),
        )),
        repo,
        notifier,
    };
    (state, data_dir)
}

#[tokio::test]
async fn test_healthz() {
    let (state, _data_dir) = setup().await;
    let app = router(state);

    let req = Request::builder()
        .uri("/api/healthz")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn test_openapi_document_exposes_core_routes() {
    let (state, _data_dir) = setup().await;
    let app = router(state);

    let req = Request::builder()
        .uri("/api-docs/openapi.json")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let paths = json["paths"].as_object().unwrap();

    assert!(paths.contains_key("/api/healthz"));
    assert!(paths.contains_key("/api/inbound"));
    assert!(paths.contains_key("/api/messages"));
    assert!(paths.contains_key("/api/messages/{id}"));
    assert!(
        paths["/api/messages/{id}"]
            .as_object()
            .unwrap()
            .contains_key("delete")
    );
    assert!(paths.contains_key("/api/messages/{id}/mailbox"));
    assert!(
        paths["/api/messages/{id}/mailbox"]
            .as_object()
            .unwrap()
            .contains_key("patch")
    );
    assert!(paths.contains_key("/api/attachments/{id}"));
    assert!(paths.contains_key("/api/tags"));
}

#[tokio::test]
async fn test_full_inbound_and_read_roundtrip() {
    let (state, _data_dir) = setup().await;
    let app = router(state.clone());

    let raw = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Hello Roundtrip\r\nContent-Type: text/plain\r\n\r\nRoundtrip body";
    let meta = r#"{"envelope_to":"recipient@example.com"}"#;
    let (body, boundary) = build_multipart_body(raw, meta);

    // 1. POST inbound
    let req = Request::builder()
        .method("POST")
        .uri("/api/inbound")
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
    let inbound_resp: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let msg_id = inbound_resp["id"].as_str().unwrap();

    // 2. GET messages list
    let req = Request::builder()
        .uri("/api/messages")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
    let list: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(list["total"], 1);
    assert_eq!(list["items"][0]["subject"], "Hello Roundtrip");

    // 3. GET message detail
    let req = Request::builder()
        .uri(format!("/api/messages/{msg_id}"))
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
    let detail: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(detail["subject"], "Hello Roundtrip");
    assert_eq!(detail["body_text"], "Roundtrip body");

    // 4. GET tags
    let req = Request::builder()
        .uri("/api/tags")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
    let tags: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let recipient_tag = tags
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["kind"] == "recipient_address")
        .unwrap();
    assert_eq!(recipient_tag["message_count"], 1);
}

#[tokio::test]
async fn test_archive_restore_and_delete_roundtrip() {
    let (state, data_dir) = setup().await;
    let app = router(state.clone());

    let raw = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Archive Delete\r\nContent-Type: multipart/mixed; boundary=\"boundary123\"\r\n\r\n--boundary123\r\nContent-Type: text/plain\r\n\r\nBody\r\n--boundary123\r\nContent-Type: text/plain\r\nContent-Disposition: attachment; filename=\"hello.txt\"\r\n\r\ntext\r\n--boundary123--";
    let meta = r#"{"envelope_to":"recipient@example.com"}"#;
    let (body, boundary) = build_multipart_body(raw, meta);

    let req = Request::builder()
        .method("POST")
        .uri("/api/inbound")
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
    let inbound_resp: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let msg_id = inbound_resp["id"].as_str().unwrap();

    let req = Request::builder()
        .method("PATCH")
        .uri(format!("/api/messages/{msg_id}/mailbox"))
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"mailbox":"archive"}"#))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let req = Request::builder()
        .uri("/api/messages")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
    let inbox: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(inbox["total"], 0);

    let req = Request::builder()
        .uri("/api/messages?mailbox=archive")
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
    let archive: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(archive["total"], 1);
    assert_eq!(archive["items"][0]["mailbox"], "archive");

    let req = Request::builder()
        .method("PATCH")
        .uri(format!("/api/messages/{msg_id}/mailbox"))
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"mailbox":"inbox"}"#))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let req = Request::builder()
        .uri(format!("/api/messages/{msg_id}"))
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
    let detail: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(detail["mailbox"], "inbox");
    let attachment_id = detail["attachments"][0]["id"].as_str().unwrap();
    let raw_path = data_dir.join("raw").join(format!("{msg_id}.eml"));
    let attachment_path = data_dir.join("attachments").join(attachment_id);
    assert!(raw_path.exists());
    assert!(attachment_path.exists());

    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/messages/{msg_id}"))
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    assert!(!raw_path.exists());
    assert!(!attachment_path.exists());

    let req = Request::builder()
        .uri(format!("/api/messages/{msg_id}"))
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    let req = Request::builder()
        .uri(format!("/api/messages/{msg_id}/raw"))
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    let req = Request::builder()
        .uri(format!("/api/attachments/{attachment_id}"))
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_ignores_missing_blob_files() {
    let (state, data_dir) = setup().await;
    let app = router(state.clone());

    let raw = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Missing Blobs\r\nContent-Type: multipart/mixed; boundary=\"boundary123\"\r\n\r\n--boundary123\r\nContent-Type: text/plain\r\n\r\nBody\r\n--boundary123\r\nContent-Type: text/plain\r\nContent-Disposition: attachment; filename=\"missing.txt\"\r\n\r\ntext\r\n--boundary123--";
    let meta = r#"{"envelope_to":"recipient@example.com"}"#;
    let (body, boundary) = build_multipart_body(raw, meta);

    let req = Request::builder()
        .method("POST")
        .uri("/api/inbound")
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
    let inbound_resp: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let msg_id = inbound_resp["id"].as_str().unwrap();

    let req = Request::builder()
        .uri(format!("/api/messages/{msg_id}"))
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
    let detail: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let attachment_id = detail["attachments"][0]["id"].as_str().unwrap();
    let raw_path = data_dir.join("raw").join(format!("{msg_id}.eml"));
    let attachment_path = data_dir.join("attachments").join(attachment_id);

    tokio::fs::remove_file(&raw_path).await.unwrap();
    tokio::fs::remove_file(&attachment_path).await.unwrap();

    let req = Request::builder()
        .method("DELETE")
        .uri(format!("/api/messages/{msg_id}"))
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let req = Request::builder()
        .uri(format!("/api/messages/{msg_id}"))
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_mailbox_404() {
    let (state, _data_dir) = setup().await;
    let app = router(state);

    let req = Request::builder()
        .method("PATCH")
        .uri("/api/messages/missing/mailbox")
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"mailbox":"archive"}"#))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_message_headers_endpoint() {
    let (state, data_dir) = setup().await;
    let app = router(state.clone());

    let raw = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Hello Headers\r\nMessage-ID: <headers-test@example.com>\r\nContent-Type: text/plain\r\n\r\nBody";
    let meta = r#"{"envelope_to":"recipient@example.com"}"#;
    let (body, boundary) = build_multipart_body(raw, meta);

    let req = Request::builder()
        .method("POST")
        .uri("/api/inbound")
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
    let inbound_resp: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let msg_id = inbound_resp["id"].as_str().unwrap();

    tokio::fs::remove_file(data_dir.join("raw").join(format!("{msg_id}.eml")))
        .await
        .unwrap();

    let req = Request::builder()
        .uri(format!("/api/messages/{msg_id}"))
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
    let detail: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(detail["body_text"], "Body");

    let req = Request::builder()
        .uri(format!("/api/messages/{msg_id}/headers"))
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
    let headers_resp: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let headers = headers_resp["headers"].as_array().unwrap();
    let names: Vec<&str> = headers
        .iter()
        .map(|h| h["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"From"));
    assert!(names.contains(&"To"));
    assert!(names.contains(&"Subject"));
    assert!(names.contains(&"Message-ID"));

    let req = Request::builder()
        .uri(format!("/api/messages/{msg_id}/raw"))
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_message_headers_404() {
    let (state, _data_dir) = setup().await;
    let app = router(state);

    let req = Request::builder()
        .uri("/api/messages/nonexistent/headers")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
