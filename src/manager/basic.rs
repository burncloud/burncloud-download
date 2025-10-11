use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Instant;
use async_trait::async_trait;
use anyhow::Result;

use crate::traits::DownloadManager;
use crate::types::{TaskId, DownloadProgress, DownloadTask, DownloadStatus};
use crate::models::{DuplicatePolicy, DuplicateResult, FileIdentifier, DuplicateReason, TaskStatus};
use crate::error::DownloadError;

/// Basic download manager implementation for demonstration and testing
///
/// This implementation provides a mock download functionality that simulates
/// real download behavior for testing and demonstration purposes.
pub struct BasicDownloadManager {
    /// All tasks by ID
    tasks: Arc<RwLock<HashMap<TaskId, DownloadTask>>>,
    /// Task progress tracking
    progress: Arc<RwLock<HashMap<TaskId, DownloadProgress>>>,
    /// Mock download simulation data
    mock_data: Arc<RwLock<HashMap<TaskId, MockDownloadData>>>,
}

/// Mock data for simulating download progress
#[derive(Clone)]
struct MockDownloadData {
    start_time: Instant,
    total_size: u64,
    download_speed: u64, // bytes per second
}

impl BasicDownloadManager {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            progress: Arc::new(RwLock::new(HashMap::new())),
            mock_data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Update progress for a task (internal method)
    async fn update_task_progress(&self, task_id: TaskId) -> Result<()> {
        let mock_data = {
            let mock_data_map = self.mock_data.read().await;
            mock_data_map.get(&task_id).cloned()
        };

        if let Some(mock_data) = mock_data {
            let elapsed = mock_data.start_time.elapsed();
            let downloaded_bytes = std::cmp::min(
                elapsed.as_secs() * mock_data.download_speed,
                mock_data.total_size
            );

            let eta_seconds = if downloaded_bytes < mock_data.total_size {
                let remaining_bytes = mock_data.total_size - downloaded_bytes;
                Some(remaining_bytes / mock_data.download_speed)
            } else {
                None
            };

            let progress = DownloadProgress {
                downloaded_bytes,
                total_bytes: Some(mock_data.total_size),
                speed_bps: mock_data.download_speed,
                eta_seconds,
            };

            {
                let mut progress_map = self.progress.write().await;
                progress_map.insert(task_id, progress);
            }

            // If download is complete, update task status
            if downloaded_bytes >= mock_data.total_size {
                let mut tasks = self.tasks.write().await;
                if let Some(task) = tasks.get_mut(&task_id) {
                    task.update_status(DownloadStatus::Completed);
                }

                // Remove mock data as download is complete
                self.mock_data.write().await.remove(&task_id);
            }
        }

        Ok(())
    }

    /// Start mock download simulation for a task
    async fn start_mock_download(&self, task_id: TaskId) {
        // Create mock download data (simulate a 10MB file downloading at 1MB/s)
        let mock_data = MockDownloadData {
            start_time: Instant::now(),
            total_size: 10 * 1024 * 1024, // 10MB
            download_speed: 1024 * 1024,  // 1MB/s
        };

        self.mock_data.write().await.insert(task_id, mock_data);

        // Initialize progress
        let initial_progress = DownloadProgress {
            downloaded_bytes: 0,
            total_bytes: Some(10 * 1024 * 1024),
            speed_bps: 1024 * 1024,
            eta_seconds: Some(10),
        };

        self.progress.write().await.insert(task_id, initial_progress);
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
        task.update_status(DownloadStatus::Downloading);
        let task_id = task.id;

        // Store the task
        self.tasks.write().await.insert(task_id, task);

        // Start mock download simulation
        self.start_mock_download(task_id).await;

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

        // Remove from mock data to stop simulation
        self.mock_data.write().await.remove(&task_id);

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

        // Resume mock download simulation
        self.start_mock_download(task_id).await;

        Ok(())
    }

    async fn cancel_download(&self, task_id: TaskId) -> Result<()> {
        // Remove from all collections
        self.tasks.write().await.remove(&task_id);
        self.progress.write().await.remove(&task_id);
        self.mock_data.write().await.remove(&task_id);

        Ok(())
    }

    async fn get_progress(&self, task_id: TaskId) -> Result<DownloadProgress> {
        // Update progress before returning
        self.update_task_progress(task_id).await?;

        let progress_map = self.progress.read().await;
        progress_map.get(&task_id)
            .cloned()
            .ok_or_else(|| DownloadError::TaskNotFound(task_id).into())
    }

    async fn get_task(&self, task_id: TaskId) -> Result<DownloadTask> {
        // Update progress to ensure task status is current
        let _ = self.update_task_progress(task_id).await;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_basic_download_manager_add_download() {
        let manager = BasicDownloadManager::new();

        let task_id = manager.add_download(
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/tmp/file.zip")
        ).await.unwrap();

        let task = manager.get_task(task_id).await.unwrap();
        assert_eq!(task.url, "https://example.com/file.zip");
        assert_eq!(task.status, DownloadStatus::Downloading);
    }

    #[tokio::test]
    async fn test_basic_download_manager_progress_tracking() {
        let manager = BasicDownloadManager::new();

        let task_id = manager.add_download(
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/tmp/file.zip")
        ).await.unwrap();

        // Initial progress should be available immediately
        let progress = manager.get_progress(task_id).await.unwrap();
        assert_eq!(progress.total_bytes, Some(10 * 1024 * 1024));
        assert!(progress.speed_bps > 0);
    }

    #[tokio::test]
    async fn test_basic_download_manager_pause_resume() {
        let manager = BasicDownloadManager::new();

        let task_id = manager.add_download(
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/tmp/file.zip")
        ).await.unwrap();

        // Pause the download
        manager.pause_download(task_id).await.unwrap();
        let task = manager.get_task(task_id).await.unwrap();
        assert_eq!(task.status, DownloadStatus::Paused);

        // Resume the download
        manager.resume_download(task_id).await.unwrap();
        let task = manager.get_task(task_id).await.unwrap();
        assert_eq!(task.status, DownloadStatus::Downloading);
    }

    #[tokio::test]
    async fn test_basic_download_manager_cancel() {
        let manager = BasicDownloadManager::new();

        let task_id = manager.add_download(
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/tmp/file.zip")
        ).await.unwrap();

        // Cancel the download
        manager.cancel_download(task_id).await.unwrap();

        // Task should not be found
        assert!(manager.get_task(task_id).await.is_err());
    }

    #[tokio::test]
    async fn test_basic_download_manager_list_tasks() {
        let manager = BasicDownloadManager::new();

        let task_id1 = manager.add_download(
            "https://example.com/file1.zip".to_string(),
            PathBuf::from("/tmp/file1.zip")
        ).await.unwrap();

        let task_id2 = manager.add_download(
            "https://example.com/file2.zip".to_string(),
            PathBuf::from("/tmp/file2.zip")
        ).await.unwrap();

        let tasks = manager.list_tasks().await.unwrap();
        assert_eq!(tasks.len(), 2);

        let task_ids: Vec<TaskId> = tasks.iter().map(|t| t.id).collect();
        assert!(task_ids.contains(&task_id1));
        assert!(task_ids.contains(&task_id2));
    }

    #[tokio::test]
    async fn test_basic_download_manager_active_count() {
        let manager = BasicDownloadManager::new();

        assert_eq!(manager.active_download_count().await.unwrap(), 0);

        let _task_id1 = manager.add_download(
            "https://example.com/file1.zip".to_string(),
            PathBuf::from("/tmp/file1.zip")
        ).await.unwrap();

        let task_id2 = manager.add_download(
            "https://example.com/file2.zip".to_string(),
            PathBuf::from("/tmp/file2.zip")
        ).await.unwrap();

        assert_eq!(manager.active_download_count().await.unwrap(), 2);

        // Pause one download
        manager.pause_download(task_id2).await.unwrap();
        assert_eq!(manager.active_download_count().await.unwrap(), 1);
    }
}