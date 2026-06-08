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
        inbound_service: Arc::new(InboundMessageService::new(repo.clone(), data_dir.clone(), notifier.clone())),
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
    assert!(paths.contains_key("/api/attachments/{id}"));
    assert!(paths.contains_key("/api/tags"));
}

#[tokio::test]
async fn test_full_inbound_and_read_roundtrip() {
    let (state, _data_dir) = setup().await;
    let app = router(state.clone());

    let raw = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Hello Roundtrip\r\nContent-Type: text/plain\r\n\r\nRoundtrip body";
    let meta = r#"{"from":"sender@example.com","to":"recipient@example.com","headers":{"message-id":"<roundtrip-1>","subject":"Hello Roundtrip","date":"Mon, 01 Jan 2024 00:00:00 +0000"}}"#;
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
