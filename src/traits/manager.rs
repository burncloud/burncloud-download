use std::path::{Path, PathBuf};
use async_trait::async_trait;
use anyhow::Result;
use burncloud_download_types::{TaskId, DownloadProgress, DownloadTask, DownloadStatus};
use crate::models::{DuplicatePolicy, DuplicateResult};

/// Core download manager trait for implementing download backends
#[async_trait]
pub trait DownloadManager: Send + Sync {
    /// Add a new download task and return task ID
    async fn add_download(&self, url: String, target_path: PathBuf) -> Result<TaskId>;

    /// Pause an active download task
    async fn pause_download(&self, task_id: TaskId) -> Result<()>;

    /// Resume a paused download task
    async fn resume_download(&self, task_id: TaskId) -> Result<()>;

    /// Cancel and remove a download task
    async fn cancel_download(&self, task_id: TaskId) -> Result<()>;

    /// Get current progress for a download task
    async fn get_progress(&self, task_id: TaskId) -> Result<DownloadProgress>;

    /// Get download task information
    async fn get_task(&self, task_id: TaskId) -> Result<DownloadTask>;

    /// List all download tasks
    async fn list_tasks(&self) -> Result<Vec<DownloadTask>>;

    /// Get number of active downloads
    async fn active_download_count(&self) -> Result<usize>;

    // New methods for duplicate detection

    /// Find existing task for the same download request
    async fn find_duplicate_task(
        &self,
        url: &str,
        target_path: &Path,
    ) -> Result<Option<TaskId>>;

    /// Add download with explicit duplicate handling policy
    async fn add_download_with_policy(
        &self,
        url: &str,
        target_path: &Path,
        policy: DuplicatePolicy,
    ) -> Result<DuplicateResult>;

    /// Verify if existing task is still valid for reuse
    async fn verify_task_validity(&self, task_id: &TaskId) -> Result<bool>;

    /// Get all potential duplicate candidates
    async fn get_duplicate_candidates(
        &self,
        url: &str,
        target_path: &Path,
    ) -> Result<Vec<TaskId>>;
}

/// Download event notification trait for implementing observers
#[async_trait]
pub trait DownloadEventHandler: Send + Sync {
    /// Called when download task status changes
    async fn on_status_changed(&self, task_id: TaskId, old_status: DownloadStatus, new_status: DownloadStatus);

    /// Called when download progress updates
    async fn on_progress_updated(&self, task_id: TaskId, progress: DownloadProgress);

    /// Called when download task is completed
    async fn on_download_completed(&self, task_id: TaskId);

    /// Called when download task fails
    async fn on_download_failed(&self, task_id: TaskId, error: String);
}