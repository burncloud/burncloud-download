use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::{RwLock, Mutex};
use anyhow::{Result, bail};
use async_trait::async_trait;
use crate::types::{TaskId, DownloadTask, DownloadStatus, DownloadProgress};
use crate::traits::{DownloadEventHandler, DownloadManager};
use crate::error::DownloadError;

/// Maximum number of concurrent downloads
const MAX_CONCURRENT_DOWNLOADS: usize = 3;

/// Task queue manager for controlling download concurrency
pub struct TaskQueueManager {
    /// Active download tasks (currently downloading)
    active_tasks: Arc<RwLock<HashMap<TaskId, DownloadTask>>>,
    /// Queued tasks waiting to start
    queued_tasks: Arc<Mutex<VecDeque<DownloadTask>>>,
    /// All tasks by ID
    all_tasks: Arc<RwLock<HashMap<TaskId, DownloadTask>>>,
    /// Task progress tracking
    progress: Arc<RwLock<HashMap<TaskId, DownloadProgress>>>,
    /// Event handlers
    event_handlers: Arc<RwLock<Vec<Arc<dyn DownloadEventHandler>>>>,
}

impl Default for TaskQueueManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskQueueManager {
    pub fn new() -> Self {
        Self {
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            queued_tasks: Arc::new(Mutex::new(VecDeque::new())),
            all_tasks: Arc::new(RwLock::new(HashMap::new())),
            progress: Arc::new(RwLock::new(HashMap::new())),
            event_handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add a new download task to the queue
    pub async fn add_task(&self, url: String, target_path: std::path::PathBuf) -> Result<TaskId> {
        let mut task = DownloadTask::new(url, target_path);
        let task_id = task.id;

        // Check if we can start immediately or need to queue
        let active_count = self.active_tasks.read().await.len();
        let should_start = active_count < MAX_CONCURRENT_DOWNLOADS;

        if should_start {
            // Start immediately
            task.update_status(DownloadStatus::Downloading);
            self.active_tasks.write().await.insert(task_id, task.clone());

            // Store in all_tasks registry with updated status
            self.all_tasks.write().await.insert(task_id, task.clone());

            // Notify after locks released
            self.notify_status_changed(task_id, DownloadStatus::Waiting, DownloadStatus::Downloading).await;
        } else {
            // Add to queue (keep waiting status)
            self.queued_tasks.lock().await.push_back(task.clone());

            // Store in all_tasks registry
            self.all_tasks.write().await.insert(task_id, task);
        }

        Ok(task_id)
    }

    /// Update progress for a task
    pub async fn update_progress(&self, task_id: TaskId, progress: DownloadProgress) -> Result<()> {
        // Verify task exists
        if !self.all_tasks.read().await.contains_key(&task_id) {
            return Err(DownloadError::TaskNotFound(task_id).into());
        }

        // Update progress
        self.progress.write().await.insert(task_id, progress.clone());

        // Notify event handlers
        self.notify_progress_updated(task_id, progress).await;

        Ok(())
    }

    /// Get progress for a task
    pub async fn get_progress(&self, task_id: TaskId) -> Result<DownloadProgress> {
        // First verify task exists
        if !self.all_tasks.read().await.contains_key(&task_id) {
            return Err(DownloadError::TaskNotFound(task_id).into());
        }

        let progress_map = self.progress.read().await;
        Ok(progress_map.get(&task_id)
            .cloned()
            .unwrap_or_else(DownloadProgress::new))
    }

    /// Pause a download task
    pub async fn pause_task(&self, task_id: TaskId) -> Result<()> {
        let old_status = {
            let mut all_tasks = self.all_tasks.write().await;
            let task = all_tasks.get_mut(&task_id)
                .ok_or(DownloadError::TaskNotFound(task_id))?;

            if !task.status.can_pause() {
                bail!("Task cannot be paused in current status: {}", task.status);
            }

            let old_status = task.status.clone();
            task.update_status(DownloadStatus::Paused);
            old_status
        }; // Release write lock

        // Remove from active tasks if present
        self.active_tasks.write().await.remove(&task_id);

        // Try to start next queued task
        self.try_start_next_queued_task().await?;

        // Notify after locks released
        self.notify_status_changed(task_id, old_status, DownloadStatus::Paused).await;
        Ok(())
    }

    /// Resume a paused download task
    pub async fn resume_task(&self, task_id: TaskId) -> Result<()> {
        let (old_status, new_status, task_clone) = {
            let mut all_tasks = self.all_tasks.write().await;
            let task = all_tasks.get_mut(&task_id)
                .ok_or(DownloadError::TaskNotFound(task_id))?;

            if !task.status.can_resume() {
                bail!("Task cannot be resumed in current status: {}", task.status);
            }

            let old_status = task.status.clone();

            // Check if we can start immediately or need to queue
            let active_count = self.active_tasks.read().await.len();
            if active_count < MAX_CONCURRENT_DOWNLOADS {
                task.update_status(DownloadStatus::Downloading);
                (old_status, DownloadStatus::Downloading, Some(task.clone()))
            } else {
                task.update_status(DownloadStatus::Waiting);
                (old_status, DownloadStatus::Waiting, Some(task.clone()))
            }
        }; // Release write lock

        // Update appropriate collections after lock released
        if new_status == DownloadStatus::Downloading {
            if let Some(task) = task_clone {
                self.active_tasks.write().await.insert(task_id, task);
            }
        } else if let Some(task) = task_clone {
            self.queued_tasks.lock().await.push_back(task);
        }

        // Notify after locks released
        self.notify_status_changed(task_id, old_status, new_status).await;

        Ok(())
    }

    /// Cancel and remove a download task
    pub async fn cancel_task(&self, task_id: TaskId) -> Result<()> {
        // Remove from all collections
        self.all_tasks.write().await.remove(&task_id);
        self.active_tasks.write().await.remove(&task_id);

        // Remove from queue if present
        {
            let mut queue = self.queued_tasks.lock().await;
            queue.retain(|task| task.id != task_id);
        }

        // Try to start next queued task
        self.try_start_next_queued_task().await?;

        Ok(())
    }

    /// Get task information
    pub async fn get_task(&self, task_id: TaskId) -> Result<DownloadTask> {
        let all_tasks = self.all_tasks.read().await;
        all_tasks.get(&task_id)
            .cloned()
            .ok_or_else(|| DownloadError::TaskNotFound(task_id).into())
    }

    /// List all tasks
    pub async fn list_tasks(&self) -> Result<Vec<DownloadTask>> {
        let all_tasks = self.all_tasks.read().await;
        Ok(all_tasks.values().cloned().collect())
    }

    /// Get number of active downloads
    pub async fn active_download_count(&self) -> usize {
        self.active_tasks.read().await.len()
    }

    /// Mark task as completed and try to start next queued task
    pub async fn complete_task(&self, task_id: TaskId) -> Result<()> {
        let old_status = {
            let mut all_tasks = self.all_tasks.write().await;
            if let Some(task) = all_tasks.get_mut(&task_id) {
                let old_status = task.status.clone();
                task.update_status(DownloadStatus::Completed);
                Some(old_status)
            } else {
                None
            }
        }; // Release write lock before notifications

        // Remove from active tasks
        self.active_tasks.write().await.remove(&task_id);

        // Try to start next queued task
        self.try_start_next_queued_task().await?;

        // Notify after all locks are released
        if let Some(old_status) = old_status {
            self.notify_status_changed(task_id, old_status, DownloadStatus::Completed).await;
            self.notify_download_completed(task_id).await;
        }

        Ok(())
    }

    /// Mark task as failed and try to start next queued task
    pub async fn fail_task(&self, task_id: TaskId, error: String) -> Result<()> {
        let old_status = {
            let mut all_tasks = self.all_tasks.write().await;
            if let Some(task) = all_tasks.get_mut(&task_id) {
                let old_status = task.status.clone();
                task.update_status(DownloadStatus::Failed(error.clone()));
                Some(old_status)
            } else {
                None
            }
        }; // Release write lock before notifications

        // Remove from active tasks
        self.active_tasks.write().await.remove(&task_id);

        // Try to start next queued task
        self.try_start_next_queued_task().await?;

        // Notify after all locks are released
        if let Some(old_status) = old_status {
            self.notify_status_changed(task_id, old_status, DownloadStatus::Failed(error.clone())).await;
            self.notify_download_failed(task_id, error).await;
        }

        Ok(())
    }

    /// Add event handler
    pub async fn add_event_handler(&self, handler: Arc<dyn DownloadEventHandler>) {
        self.event_handlers.write().await.push(handler);
    }

    /// Try to start the next queued task if slot available
    async fn try_start_next_queued_task(&self) -> Result<()> {
        let active_count = self.active_tasks.read().await.len();
        if active_count >= MAX_CONCURRENT_DOWNLOADS {
            return Ok(());
        }

        let next_task = {
            let mut queue = self.queued_tasks.lock().await;
            queue.pop_front()
        };

        if let Some(mut task) = next_task {
            let task_id = task.id;
            task.update_status(DownloadStatus::Downloading);

            // Update in all_tasks registry
            {
                let mut all_tasks = self.all_tasks.write().await;
                all_tasks.insert(task_id, task.clone());
            }

            // Add to active tasks
            self.active_tasks.write().await.insert(task_id, task);

            self.notify_status_changed(task_id, DownloadStatus::Waiting, DownloadStatus::Downloading).await;
        }

        Ok(())
    }

    /// Notify event handlers of status change
    async fn notify_status_changed(&self, task_id: TaskId, old_status: DownloadStatus, new_status: DownloadStatus) {
        let handlers = {
            let handlers_lock = self.event_handlers.read().await;
            handlers_lock.clone()
        }; // Release read lock before calling handlers

        for handler in handlers.iter() {
            handler.on_status_changed(task_id, old_status.clone(), new_status.clone()).await;
        }
    }

    /// Notify event handlers of download completion
    async fn notify_download_completed(&self, task_id: TaskId) {
        let handlers = {
            let handlers_lock = self.event_handlers.read().await;
            handlers_lock.clone()
        }; // Release read lock before calling handlers

        for handler in handlers.iter() {
            handler.on_download_completed(task_id).await;
        }
    }

    /// Notify event handlers of download failure
    async fn notify_download_failed(&self, task_id: TaskId, error: String) {
        let handlers = {
            let handlers_lock = self.event_handlers.read().await;
            handlers_lock.clone()
        }; // Release read lock before calling handlers

        for handler in handlers.iter() {
            handler.on_download_failed(task_id, error.clone()).await;
        }
    }

    /// Notify event handlers of progress update
    async fn notify_progress_updated(&self, task_id: TaskId, progress: DownloadProgress) {
        let handlers = {
            let handlers_lock = self.event_handlers.read().await;
            handlers_lock.clone()
        }; // Release read lock before calling handlers

        for handler in handlers.iter() {
            handler.on_progress_updated(task_id, progress.clone()).await;
        }
    }
}

#[async_trait]
impl DownloadManager for TaskQueueManager {
    async fn add_download(&self, url: String, target_path: PathBuf) -> Result<TaskId> {
        self.add_task(url, target_path).await
    }

    async fn pause_download(&self, task_id: TaskId) -> Result<()> {
        self.pause_task(task_id).await
    }

    async fn resume_download(&self, task_id: TaskId) -> Result<()> {
        self.resume_task(task_id).await
    }

    async fn cancel_download(&self, task_id: TaskId) -> Result<()> {
        self.cancel_task(task_id).await
    }

    async fn get_progress(&self, task_id: TaskId) -> Result<DownloadProgress> {
        TaskQueueManager::get_progress(self, task_id).await
    }

    async fn get_task(&self, task_id: TaskId) -> Result<DownloadTask> {
        TaskQueueManager::get_task(self, task_id).await
    }

    async fn list_tasks(&self) -> Result<Vec<DownloadTask>> {
        TaskQueueManager::list_tasks(self).await
    }

    async fn active_download_count(&self) -> Result<usize> {
        Ok(TaskQueueManager::active_download_count(self).await)
    }

    // Duplicate detection methods

    async fn find_duplicate_task(
        &self,
        url: &str,
        target_path: &std::path::Path,
    ) -> Result<Option<TaskId>> {
        // Check all tasks for URL and path matches
        let all_tasks = self.all_tasks.read().await;
        for task in all_tasks.values() {
            if task.url == url && task.target_path == target_path {
                return Ok(Some(task.id));
            }
        }
        Ok(None)
    }

    async fn add_download_with_policy(
        &self,
        url: &str,
        target_path: &std::path::Path,
        policy: crate::models::DuplicatePolicy,
    ) -> Result<crate::models::DuplicateResult> {
        use crate::models::{DuplicateResult, DuplicateReason, TaskStatus};

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
                return Err(crate::error::DownloadError::PolicyViolation {
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
        // For TaskQueueManager, just check if task exists
        let all_tasks = self.all_tasks.read().await;
        Ok(all_tasks.contains_key(task_id))
    }

    async fn get_duplicate_candidates(
        &self,
        url: &str,
        target_path: &std::path::Path,
    ) -> Result<Vec<TaskId>> {
        let mut candidates = Vec::new();
        let all_tasks = self.all_tasks.read().await;

        // Look for exact matches
        for task in all_tasks.values() {
            if task.url == url && task.target_path == target_path {
                candidates.push(task.id);
            }
        }

        Ok(candidates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use crate::traits::DownloadEventHandler;
    use crate::types::{DownloadStatus, DownloadProgress};
    use async_trait::async_trait;

    // Test event handler for capturing events
    struct TestEventHandler {
        events: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl DownloadEventHandler for TestEventHandler {
        async fn on_status_changed(&self, task_id: TaskId, old_status: DownloadStatus, new_status: DownloadStatus) {
            let mut events = self.events.lock().await;
            events.push(format!("Status changed for {}: {} -> {}", task_id, old_status, new_status));
        }

        async fn on_progress_updated(&self, task_id: TaskId, _progress: DownloadProgress) {
            let mut events = self.events.lock().await;
            events.push(format!("Progress updated for {}", task_id));
        }

        async fn on_download_completed(&self, task_id: TaskId) {
            let mut events = self.events.lock().await;
            events.push(format!("Download completed: {}", task_id));
        }

        async fn on_download_failed(&self, task_id: TaskId, error: String) {
            let mut events = self.events.lock().await;
            events.push(format!("Download failed {}: {}", task_id, error));
        }
    }

    #[tokio::test]
    async fn test_add_task() {
        let manager = TaskQueueManager::new();
        let task_id = manager.add_task(
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/downloads/file.zip")
        ).await.unwrap();

        let task = manager.get_task(task_id).await.unwrap();
        assert_eq!(task.url, "https://example.com/file.zip");
        assert_eq!(task.status, DownloadStatus::Downloading);
    }

    #[tokio::test]
    async fn test_concurrency_limit() {
        let manager = TaskQueueManager::new();

        // Add 5 tasks (should only start 3)
        let mut task_ids = Vec::new();
        for i in 0..5 {
            let task_id = manager.add_task(
                format!("https://example.com/file{}.zip", i),
                PathBuf::from(format!("/downloads/file{}.zip", i))
            ).await.unwrap();
            task_ids.push(task_id);
        }

        // First 3 should be downloading, last 2 should be waiting
        assert_eq!(manager.active_download_count().await, 3);

        // Verify the queued tasks
        let queued_count = manager.queued_tasks.lock().await.len();
        assert_eq!(queued_count, 2);

        // Verify task statuses
        for i in 0..3 {
            let task = manager.get_task(task_ids[i]).await.unwrap();
            assert_eq!(task.status, DownloadStatus::Downloading, "Task {} should be downloading", i);
        }

        // Manually complete first task - simulate what complete_task does but simpler
        {
            // Update status
            let mut all_tasks = manager.all_tasks.write().await;
            if let Some(task) = all_tasks.get_mut(&task_ids[0]) {
                task.update_status(DownloadStatus::Completed);
            }
        }

        // Remove from active
        manager.active_tasks.write().await.remove(&task_ids[0]);

        // Try to start next queued task
        manager.try_start_next_queued_task().await.unwrap();

        // Should still have 3 active (one completed, one started from queue)
        assert_eq!(manager.active_download_count().await, 3);

        // Queue should now have only 1 task
        let queued_count = manager.queued_tasks.lock().await.len();
        assert_eq!(queued_count, 1);
    }

    #[tokio::test]
    async fn test_pause_resume_task() {
        let manager = TaskQueueManager::new();
        let task_id = manager.add_task(
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/downloads/file.zip")
        ).await.unwrap();

        // Pause task
        manager.pause_task(task_id).await.unwrap();
        let task = manager.get_task(task_id).await.unwrap();
        assert_eq!(task.status, DownloadStatus::Paused);

        // Resume task
        manager.resume_task(task_id).await.unwrap();
        let task = manager.get_task(task_id).await.unwrap();
        assert_eq!(task.status, DownloadStatus::Downloading);
    }

    #[tokio::test]
    async fn test_cancel_task() {
        let manager = TaskQueueManager::new();
        let task_id = manager.add_task(
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/downloads/file.zip")
        ).await.unwrap();

        // Cancel task
        manager.cancel_task(task_id).await.unwrap();

        // Task should not be found
        assert!(manager.get_task(task_id).await.is_err());
    }

    #[tokio::test]
    async fn test_task_list() {
        let manager = TaskQueueManager::new();

        // Add multiple tasks
        let task_id1 = manager.add_task(
            "https://example.com/file1.zip".to_string(),
            PathBuf::from("/downloads/file1.zip")
        ).await.unwrap();

        let task_id2 = manager.add_task(
            "https://example.com/file2.zip".to_string(),
            PathBuf::from("/downloads/file2.zip")
        ).await.unwrap();

        let tasks = manager.list_tasks().await.unwrap();
        assert_eq!(tasks.len(), 2);

        let task_ids: Vec<TaskId> = tasks.iter().map(|t| t.id).collect();
        assert!(task_ids.contains(&task_id1));
        assert!(task_ids.contains(&task_id2));
    }

    #[tokio::test]
    async fn test_event_notifications() {
        let manager = TaskQueueManager::new();
        let events = Arc::new(Mutex::new(Vec::new()));
        let handler = Arc::new(TestEventHandler { events: events.clone() });

        manager.add_event_handler(handler).await;

        // Add task
        let task_id = manager.add_task(
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/downloads/file.zip")
        ).await.unwrap();

        // Pause task
        manager.pause_task(task_id).await.unwrap();

        // Resume task
        manager.resume_task(task_id).await.unwrap();

        // Complete task
        manager.complete_task(task_id).await.unwrap();

        // Verify events
        let events = events.lock().await;
        assert!(events.iter().any(|e| e.contains("Status changed")));
        assert!(events.iter().any(|e| e.contains("Download completed")));
    }

    #[tokio::test]
    async fn test_fail_task() {
        let manager = TaskQueueManager::new();
        let events = Arc::new(Mutex::new(Vec::new()));
        let handler = Arc::new(TestEventHandler { events: events.clone() });

        manager.add_event_handler(handler).await;

        let task_id = manager.add_task(
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/downloads/file.zip")
        ).await.unwrap();

        // Fail task
        manager.fail_task(task_id, "Connection error".to_string()).await.unwrap();

        let task = manager.get_task(task_id).await.unwrap();
        assert!(matches!(task.status, DownloadStatus::Failed(_)));

        // Check events
        let events = events.lock().await;
        assert!(events.iter().any(|e| e.contains("Download failed")));
    }

    #[tokio::test]
    async fn test_progress_tracking() {
        let manager = TaskQueueManager::new();
        let task_id = manager.add_task(
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/downloads/file.zip")
        ).await.unwrap();

        // Update progress
        let progress = DownloadProgress {
            downloaded_bytes: 1024,
            total_bytes: Some(10240),
            speed_bps: 512,
            eta_seconds: Some(18),
        };

        manager.update_progress(task_id, progress.clone()).await.unwrap();

        // Get progress
        let retrieved_progress = manager.get_progress(task_id).await.unwrap();
        assert_eq!(retrieved_progress.downloaded_bytes, 1024);
        assert_eq!(retrieved_progress.total_bytes, Some(10240));
        assert_eq!(retrieved_progress.speed_bps, 512);
        assert_eq!(retrieved_progress.eta_seconds, Some(18));
    }

    #[tokio::test]
    async fn test_download_manager_trait_implementation() {
        let manager: Arc<dyn DownloadManager> = Arc::new(TaskQueueManager::new());

        // Test add_download
        let task_id = manager.add_download(
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/downloads/file.zip")
        ).await.unwrap();

        // Test get_task
        let task = manager.get_task(task_id).await.unwrap();
        assert_eq!(task.url, "https://example.com/file.zip");

        // Test get_progress
        let progress = manager.get_progress(task_id).await.unwrap();
        assert_eq!(progress.downloaded_bytes, 0);

        // Test list_tasks
        let tasks = manager.list_tasks().await.unwrap();
        assert_eq!(tasks.len(), 1);

        // Test active_download_count
        let count = manager.active_download_count().await.unwrap();
        assert_eq!(count, 1);

        // Test pause_download
        manager.pause_download(task_id).await.unwrap();
        let task = manager.get_task(task_id).await.unwrap();
        assert_eq!(task.status, DownloadStatus::Paused);

        // Test resume_download
        manager.resume_download(task_id).await.unwrap();
        let task = manager.get_task(task_id).await.unwrap();
        assert_eq!(task.status, DownloadStatus::Downloading);

        // Test cancel_download
        manager.cancel_download(task_id).await.unwrap();
        assert!(manager.get_task(task_id).await.is_err());
    }
}