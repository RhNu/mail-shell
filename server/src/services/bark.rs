use async_trait::async_trait;
use reqwest::Client;

use super::notifier::{Notification, Notifier, NotifierError};

#[derive(Debug, Clone)]
pub struct BarkConfig {
    pub server_url: String,
    pub key: String,
    pub group: Option<String>,
    pub sound: Option<String>,
    pub level: Option<String>,
}

pub struct BarkNotifier {
    client: Client,
    config: BarkConfig,
}

impl BarkNotifier {
    pub fn new(config: BarkConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub fn new_with_client(client: Client, config: BarkConfig) -> Self {
        Self { client, config }
    }
}

#[derive(serde::Serialize)]
struct BarkPayload {
    title: String,
    body: String,
    group: Option<String>,
    url: Option<String>,
    sound: Option<String>,
    level: Option<String>,
}

#[async_trait]
impl Notifier for BarkNotifier {
    async fn notify(&self, notification: Notification) -> Result<(), NotifierError> {
        let url = format!("{}/{}", self.config.server_url, self.config.key);

        let payload = BarkPayload {
            title: notification.title,
            body: notification.body,
            group: notification.group.or_else(|| self.config.group.clone()),
            url: notification.url,
            sound: self.config.sound.clone(),
            level: self.config.level.clone(),
        };

        let response = self.client.post(&url).json(&payload).send().await?;
        let status = response.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            let body = response.text().await?;
            return Err(NotifierError::BarkError { status: status_code, body });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_notification() -> Notification {
        Notification {
            title: "New Mail".into(),
            body: "From: alice@example.com\nSubject: Hello".into(),
            group: Some("mail-shell".into()),
            url: None,
        }
    }

    fn sample_config() -> BarkConfig {
        BarkConfig {
            server_url: "http://127.0.0.1".into(),
            key: "testkey".into(),
            group: Some("mail-shell".into()),
            sound: None,
            level: None,
        }
    }

    #[test]
    fn bark_config_is_clone_debug() {
        let config = sample_config();
        let _cloned = config.clone();
        let _ = format!("{:?}", config);
    }

    #[tokio::test]
    async fn bark_notifier_sends_post_to_correct_url() {
        let mut server = mockito::Server::new_async().await;
        let mut config = sample_config();
        config.server_url = server.url();

        let mock = server
            .mock("POST", "/testkey")
            .match_header("content-type", "application/json")
            .match_body(mockito::Matcher::JsonString(
                serde_json::json!({
                    "title": "New Mail",
                    "body": "From: alice@example.com\nSubject: Hello",
                    "group": "mail-shell",
                    "url": null,
                    "sound": null,
                    "level": null,
                })
                .to_string(),
            ))
            .with_status(200)
            .with_body("")
            .create_async()
            .await;

        let notifier = BarkNotifier::new(config);
        notifier.notify(sample_notification()).await.unwrap();
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn bark_notifier_returns_error_on_non_2xx() {
        let mut server = mockito::Server::new_async().await;
        let mut config = sample_config();
        config.server_url = server.url();

        let mock = server
            .mock("POST", "/testkey")
            .with_status(400)
            .with_body("bad request")
            .create_async()
            .await;

        let notifier = BarkNotifier::new(config);
        let err = notifier.notify(sample_notification()).await.unwrap_err();
        mock.assert_async().await;

        match err {
            NotifierError::BarkError { status, body } => {
                assert_eq!(status, 400);
                assert_eq!(body, "bad request");
            }
            _ => panic!("expected BarkError, got {err:?}"),
        }
    }

    #[tokio::test]
    async fn bark_notifier_uses_config_group_as_fallback() {
        let mut server = mockito::Server::new_async().await;
        let mut config = sample_config();
        config.server_url = server.url();
        config.group = Some("default-group".into());

        let mut notification = sample_notification();
        notification.group = None;

        let mock = server
            .mock("POST", "/testkey")
            .match_body(mockito::Matcher::JsonString(
                serde_json::json!({
                    "title": "New Mail",
                    "body": "From: alice@example.com\nSubject: Hello",
                    "group": "default-group",
                    "url": null,
                    "sound": null,
                    "level": null,
                })
                .to_string(),
            ))
            .with_status(200)
            .create_async()
            .await;

        let notifier = BarkNotifier::new(config);
        notifier.notify(notification).await.unwrap();
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn bark_notifier_notification_group_overrides_config() {
        let mut server = mockito::Server::new_async().await;
        let mut config = sample_config();
        config.server_url = server.url();
        config.group = Some("default-group".into());

        let mut notification = sample_notification();
        notification.group = Some("override-group".into());

        let mock = server
            .mock("POST", "/testkey")
            .match_body(mockito::Matcher::JsonString(
                serde_json::json!({
                    "title": "New Mail",
                    "body": "From: alice@example.com\nSubject: Hello",
                    "group": "override-group",
                    "url": null,
                    "sound": null,
                    "level": null,
                })
                .to_string(),
            ))
            .with_status(200)
            .create_async()
            .await;

        let notifier = BarkNotifier::new(config);
        notifier.notify(notification).await.unwrap();
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn bark_notifier_sends_sound_and_level_from_config() {
        let mut server = mockito::Server::new_async().await;
        let mut config = sample_config();
        config.server_url = server.url();
        config.sound = Some("alarm".into());
        config.level = Some("timeSensitive".into());

        let mock = server
            .mock("POST", "/testkey")
            .match_body(mockito::Matcher::JsonString(
                serde_json::json!({
                    "title": "New Mail",
                    "body": "From: alice@example.com\nSubject: Hello",
                    "group": "mail-shell",
                    "url": null,
                    "sound": "alarm",
                    "level": "timeSensitive",
                })
                .to_string(),
            ))
            .with_status(200)
            .create_async()
            .await;

        let notifier = BarkNotifier::new(config);
        notifier.notify(sample_notification()).await.unwrap();
        mock.assert_async().await;
    }
}
