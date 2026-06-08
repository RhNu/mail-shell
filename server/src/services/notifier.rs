use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct Notification {
    pub title: String,
    pub body: String,
    pub group: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum NotifierError {
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("bark server returned error: status={status}, body={body}")]
    BarkError { status: u16, body: String },
}

#[async_trait]
pub trait Notifier: Send + Sync {
    async fn notify(&self, notification: Notification) -> Result<(), NotifierError>;
}

pub struct NoopNotifier;

#[async_trait]
impl Notifier for NoopNotifier {
    async fn notify(&self, _notification: Notification) -> Result<(), NotifierError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn noop_notifier_returns_ok() {
        let notifier = NoopNotifier;
        let notification = Notification {
            title: "test".into(),
            body: "body".into(),
            group: None,
            url: None,
        };
        notifier.notify(notification).await.unwrap();
    }
}
