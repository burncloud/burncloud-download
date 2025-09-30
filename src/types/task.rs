use std::path::PathBuf;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::types::DownloadStatus;

/// Unique identifier for download tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub Uuid);

impl TaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Core download task representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    pub id: TaskId,
    pub url: String,
    pub target_path: PathBuf,
    pub status: DownloadStatus,
    pub created_at: std::time::SystemTime,
    pub updated_at: std::time::SystemTime,
}

impl DownloadTask {
    pub fn new(url: String, target_path: PathBuf) -> Self {
        let now = std::time::SystemTime::now();
        Self {
            id: TaskId::new(),
            url,
            target_path,
            status: DownloadStatus::Waiting,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_status(&mut self, status: DownloadStatus) {
        self.status = status;
        self.updated_at = std::time::SystemTime::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_task_id_generation() {
        let id1 = TaskId::new();
        let id2 = TaskId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_task_id_display() {
        let id = TaskId::new();
        let display_str = format!("{}", id);
        assert!(!display_str.is_empty());
        assert_eq!(display_str, id.0.to_string());
    }

    #[test]
    fn test_download_task_creation() {
        let task = DownloadTask::new(
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/downloads/file.zip")
        );

        assert_eq!(task.url, "https://example.com/file.zip");
        assert_eq!(task.target_path, PathBuf::from("/downloads/file.zip"));
        assert_eq!(task.status, DownloadStatus::Waiting);
    }

    #[test]
    fn test_download_task_update_status() {
        let mut task = DownloadTask::new(
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/downloads/file.zip")
        );

        let initial_time = task.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(1));

        task.update_status(DownloadStatus::Downloading);

        assert_eq!(task.status, DownloadStatus::Downloading);
        assert!(task.updated_at > initial_time);
    }
}