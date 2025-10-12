use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use anyhow::Result;

use crate::traits::DownloadManager;
use crate::types::{TaskId, DownloadProgress, DownloadTask, DownloadStatus};
use crate::models::{DuplicatePolicy, DuplicateResult, FileIdentifier, DuplicateReason, TaskStatus};
use crate::error::DownloadError;

/// Basic download manager implementation for testing and minimal functionality
///
/// This implementation provides basic task management without actual download
/// functionality. It's intended for testing and as a minimal reference implementation.
pub struct BasicDownloadManager {
    /// All tasks by ID
    tasks: Arc<RwLock<HashMap<TaskId, DownloadTask>>>,
    /// Task progress tracking
    progress: Arc<RwLock<HashMap<TaskId, DownloadProgress>>>,
}

impl BasicDownloadManager {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            progress: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for BasicDownloadManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DownloadManager for BasicDownloadManager {
    async fn add_download(&self, url: String, target_path: PathBuf) -> Result<TaskId> {
        let mut task = DownloadTask::new(url, target_path);
        task.update_status(DownloadStatus::Waiting);
        let task_id = task.id;

        // Store the task
        self.tasks.write().await.insert(task_id, task);

        // Initialize basic progress
        let initial_progress = DownloadProgress {
            downloaded_bytes: 0,
            total_bytes: None,
            speed_bps: 0,
            eta_seconds: None,
        };

        self.progress.write().await.insert(task_id, initial_progress);

        Ok(task_id)
    }

    async fn pause_download(&self, task_id: TaskId) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        let task = tasks.get_mut(&task_id)
            .ok_or(DownloadError::TaskNotFound(task_id))?;

        if !task.status.can_pause() {
            return Err(anyhow::anyhow!("Task cannot be paused in current status: {}", task.status));
        }

        task.update_status(DownloadStatus::Paused);

        Ok(())
    }

    async fn resume_download(&self, task_id: TaskId) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        let task = tasks.get_mut(&task_id)
            .ok_or(DownloadError::TaskNotFound(task_id))?;

        if !task.status.can_resume() {
            return Err(anyhow::anyhow!("Task cannot be resumed in current status: {}", task.status));
        }

        task.update_status(DownloadStatus::Downloading);

        Ok(())
    }

    async fn cancel_download(&self, task_id: TaskId) -> Result<()> {
        // Remove from all collections
        self.tasks.write().await.remove(&task_id);
        self.progress.write().await.remove(&task_id);

        Ok(())
    }

    async fn get_progress(&self, task_id: TaskId) -> Result<DownloadProgress> {
        let progress_map = self.progress.read().await;
        progress_map.get(&task_id)
            .cloned()
            .ok_or_else(|| DownloadError::TaskNotFound(task_id).into())
    }

    async fn get_task(&self, task_id: TaskId) -> Result<DownloadTask> {
        let tasks = self.tasks.read().await;
        tasks.get(&task_id)
            .cloned()
            .ok_or_else(|| DownloadError::TaskNotFound(task_id).into())
    }

    async fn list_tasks(&self) -> Result<Vec<DownloadTask>> {
        let tasks = self.tasks.read().await;
        Ok(tasks.values().cloned().collect())
    }

    async fn active_download_count(&self) -> Result<usize> {
        let tasks = self.tasks.read().await;
        let count = tasks.values()
            .filter(|task| task.status.is_active())
            .count();
        Ok(count)
    }

    // Duplicate detection methods

    async fn find_duplicate_task(
        &self,
        url: &str,
        target_path: &Path,
    ) -> Result<Option<TaskId>> {
        let _identifier = FileIdentifier::new(url, target_path, None);
        let tasks = self.tasks.read().await;

        // Simple in-memory duplicate detection for BasicDownloadManager
        // Look for exact URL and path matches
        for task in tasks.values() {
            if task.url == url && task.target_path == target_path {
                return Ok(Some(task.id));
            }
        }

        Ok(None)
    }

    async fn add_download_with_policy(
        &self,
        url: &str,
        target_path: &Path,
        policy: DuplicatePolicy,
    ) -> Result<DuplicateResult> {
        // Check for duplicates first
        if let Some(existing_task_id) = self.find_duplicate_task(url, target_path).await? {
            let task = self.get_task(existing_task_id).await?;
            let task_status = TaskStatus::from_download_status(task.status);

            if policy.allows_reuse(&task_status) {
                return Ok(DuplicateResult::ExistingTask {
                    task_id: existing_task_id,
                    status: task_status,
                    reason: DuplicateReason::UrlAndPath,
                });
            } else if policy.should_fail_on_duplicate() {
                return Err(DownloadError::PolicyViolation {
                    task_id: existing_task_id,
                    reason: "Duplicate found but policy forbids reuse".to_string(),
                }.into());
            }
        }

        // No duplicate found or policy allows new task, create new download
        let task_id = self.add_download(url.to_string(), target_path.to_path_buf()).await?;
        Ok(DuplicateResult::NewTask(task_id))
    }

    async fn verify_task_validity(&self, task_id: &TaskId) -> Result<bool> {
        // For BasicDownloadManager, just check if task exists
        // In real implementation, this would check file existence, source accessibility, etc.
        let tasks = self.tasks.read().await;
        Ok(tasks.contains_key(task_id))
    }

    async fn get_duplicate_candidates(
        &self,
        url: &str,
        target_path: &Path,
    ) -> Result<Vec<TaskId>> {
        let mut candidates = Vec::new();
        let tasks = self.tasks.read().await;

        // Look for exact matches first
        for task in tasks.values() {
            if task.url == url && task.target_path == target_path {
                candidates.push(task.id);
            }
        }

        // For BasicDownloadManager, we don't do complex duplicate detection
        // Just return exact matches
        Ok(candidates)
    }
}