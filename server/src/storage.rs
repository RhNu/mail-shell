use std::path::{Path, PathBuf};

use uuid::Uuid;

/// Resolve the data directory from the `MAIL_SHELL_DATA_DIR` environment variable,
/// defaulting to `"data"` in the current working directory.
pub fn resolve_data_dir() -> PathBuf {
    std::env::var("MAIL_SHELL_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("data"))
}

/// Ensure that the required subdirectories (`raw` and `attachments`) exist
/// inside the given data directory.
#[tracing::instrument]
pub fn ensure_dirs(data_dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(data_dir.join("raw"))?;
    std::fs::create_dir_all(data_dir.join("attachments"))?;
    Ok(())
}

/// Build the filesystem path for a raw `.eml` file.
pub fn raw_path(data_dir: &Path, msg_id: &str) -> PathBuf {
    data_dir.join("raw").join(format!("{msg_id}.eml"))
}

/// Build the filesystem path for an attachment file.
pub fn attachment_path(data_dir: &Path, attachment_id: &str) -> PathBuf {
    data_dir.join("attachments").join(attachment_id)
}

/// Write raw MIME bytes to disk.
#[tracing::instrument(skip(bytes), fields(msg_id))]
pub async fn save_raw(data_dir: &Path, msg_id: &str, bytes: &[u8]) -> std::io::Result<PathBuf> {
    ensure_dirs(data_dir)?;
    let path = raw_path(data_dir, msg_id);
    tokio::fs::write(&path, bytes).await?;
    tracing::debug!(path = %path.display(), "saved raw email");
    Ok(path)
}

/// Write attachment bytes to disk.
#[tracing::instrument(skip(bytes), fields(attachment_id))]
pub async fn save_attachment(
    data_dir: &Path,
    attachment_id: &str,
    bytes: &[u8],
) -> std::io::Result<PathBuf> {
    ensure_dirs(data_dir)?;
    let path = attachment_path(data_dir, attachment_id);
    tokio::fs::write(&path, bytes).await?;
    tracing::debug!(path = %path.display(), "saved attachment");
    Ok(path)
}

/// Generate a new UUID v4 string for use as a message or attachment identifier.
pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_dirs_creates_nested_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let data = tmp.path().join("data");
        ensure_dirs(&data).unwrap();
        assert!(data.join("raw").is_dir());
        assert!(data.join("attachments").is_dir());
    }

    #[tokio::test]
    async fn test_save_raw_creates_file() {
        let tmp = tempfile::tempdir().unwrap();
        let data = tmp.path().join("data");
        let content = b"raw mime content";
        let path = save_raw(&data, "msg-1", content).await.unwrap();
        assert!(path.exists());
        let read = tokio::fs::read(&path).await.unwrap();
        assert_eq!(read, content);
    }

    #[tokio::test]
    async fn test_save_attachment_creates_file() {
        let tmp = tempfile::tempdir().unwrap();
        let data = tmp.path().join("data");
        let content = b"attachment bytes";
        let path = save_attachment(&data, "att-1", content).await.unwrap();
        assert!(path.exists());
        let read = tokio::fs::read(&path).await.unwrap();
        assert_eq!(read, content);
    }
}
