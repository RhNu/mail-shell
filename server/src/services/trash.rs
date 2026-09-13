use std::{sync::Arc, time::Duration};

use chrono::{Duration as ChronoDuration, Utc};

use crate::repository::{Repository, RepositoryError};

pub async fn purge_expired(repo: &dyn Repository) -> Result<usize, RepositoryError> {
    purge(repo, Some(Utc::now() - ChronoDuration::days(30))).await
}

pub async fn purge_all(repo: &dyn Repository) -> Result<usize, RepositoryError> {
    purge(repo, None).await
}

async fn purge(
    repo: &dyn Repository,
    cutoff: Option<chrono::DateTime<Utc>>,
) -> Result<usize, RepositoryError> {
    let ids = repo.list_trashed_before(cutoff).await?;
    let mut purged = 0;
    for id in ids {
        if let Some(files) = repo.delete_message(&id).await? {
            remove_file(&files.raw_path).await;
            for path in files.attachment_paths {
                remove_file(&path).await;
            }
            purged += 1;
        }
    }
    Ok(purged)
}

async fn remove_file(path: &str) {
    match tokio::fs::remove_file(path).await {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => tracing::warn!(path, %error, "failed to remove purged message blob"),
    }
}

pub fn spawn_daily_cleanup(repo: Arc<dyn Repository>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(24 * 60 * 60));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        interval.tick().await;
        loop {
            interval.tick().await;
            match purge_expired(&*repo).await {
                Ok(count) if count > 0 => tracing::info!(count, "purged expired trash"),
                Ok(_) => {}
                Err(error) => tracing::warn!(%error, "failed to purge expired trash"),
            }
        }
    });
}
