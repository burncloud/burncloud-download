//! Persistent Aria2 Download Manager
//!
//! This module integrates `Aria2DownloadManager` with `DownloadRepository` to provide
//! automatic persistence of download tasks and progress to the database. It includes:
//!
//! - Automatic task recovery on startup
//! - Progress saving every 5 seconds
//! - Task mapping management between database TaskIds and aria2 GIDs
//! - Robust error handling for database and aria2 failures
//!
//! ## Usage
//!
//! ```rust,no_run
//! use burncloud_download::{PersistentAria2Manager, DownloadManager};
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create manager with default configuration
//!     let manager = PersistentAria2Manager::new().await?;
//!
//!     // Add a download
//!     let task_id = manager.add_download(
//!         "https://example.com/file.zip".to_string(),
//!         PathBuf::from("data/file.zip")
//!     ).await?;
//!
//!     // Download is automatically persisted and will resume after restart
//!     manager.shutdown().await?;
//!     Ok(())
//! }
//! ```

use crate::traits::DownloadManager;
use burncloud_download_types::{TaskId, DownloadProgress, DownloadTask, DownloadStatus, DownloadManager as DownloadManagerTrait};
use burncloud_download_aria2::Aria2DownloadManager;
use burncloud_database_download::{DownloadRepository, Database};
use crate::models::{DuplicatePolicy, DuplicateResult, FileIdentifier, DuplicateReason, TaskStatus};
use async_trait::async_trait;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

/// Configuration constants
const ARIA2_RPC_URL: &str = "http://localhost:6800/jsonrpc";
const ARIA2_RPC_SECRET: &str = "burncloud";
const PROGRESS_SAVE_INTERVAL_SECS: u64 = 5;
const STATUS_POLL_INTERVAL_SECS: u64 = 1;

/// Persistent download manager that integrates Aria2 with database persistence
pub struct PersistentAria2Manager {
    aria2: Arc<Aria2DownloadManager>,
    repository: Arc<DownloadRepository>,
    task_mapping: Arc<RwLock<HashMap<TaskId, String>>>, // TaskId -> Aria2 GID mapping
    persistence_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    shutdown: Arc<tokio::sync::Notify>,
}

impl PersistentAria2Manager {
    /// Create a new persistent download manager with default configuration
    pub async fn new() -> Result<Self> {
        Self::new_with_config(
            ARIA2_RPC_URL.to_string(),
            ARIA2_RPC_SECRET.to_string(),
            None,
        ).await
    }

    /// Create a new persistent download manager with custom configuration
    pub async fn new_with_config(
        rpc_url: String,
        secret: String,
        db_path: Option<PathBuf>,
    ) -> Result<Self> {
        // Initialize database
        let db = if let Some(path) = db_path {
            let mut db = Database::new(path);
            db.initialize().await
                .map_err(|e| anyhow::anyhow!("Failed to initialize database: {}", e))?;
            db
        } else {
            Database::new_default_initialized().await
                .map_err(|e| anyhow::anyhow!("Failed to initialize database: {}", e))?
        };

        let repository = Arc::new(DownloadRepository::new(db));

        // Initialize database schema
        repository.initialize().await
            .map_err(|e| anyhow::anyhow!("Failed to initialize repository schema: {}", e))?;

        // Initialize Aria2 manager
        let aria2 = Arc::new(
            Aria2DownloadManager::new(rpc_url, Some(secret)).await?
        );

        let shutdown = Arc::new(tokio::sync::Notify::new());
        let task_mapping = Arc::new(RwLock::new(HashMap::new()));

        let manager = Self {
            aria2: aria2.clone(),
            repository: repository.clone(),
            task_mapping: task_mapping.clone(),
            persistence_handle: Arc::new(RwLock::new(None)),
            shutdown: shutdown.clone(),
        };

        // Restore tasks from database
        manager.restore_tasks().await?;

        // Start persistence poller
        manager.start_persistence_poller().await;

        Ok(manager)
    }

    /// Restore incomplete tasks from database on startup
    async fn restore_tasks(&self) -> Result<()> {
        let all_tasks = self.repository.list_tasks().await
            .map_err(|e| anyhow::anyhow!("Failed to list tasks from database: {}", e))?;

        log::info!("Found {} tasks in database", all_tasks.len());

        for task in all_tasks {
            // Only restore incomplete tasks
            if task.status.is_finished() {
                log::debug!("Skipping completed task: {} ({})", task.id, task.status);
                continue;
            }

            log::info!("Restoring task: {} ({})", task.id, task.url);

            // Attempt to restore the task in aria2
            match self.restore_single_task(&task).await {
                Ok(new_gid) => {
                    // Store mapping with new GID
                    self.store_task_mapping(task.id, new_gid.clone()).await;

                    log::info!("Successfully restored task: {} -> GID: {}", task.id, new_gid);
                }
                Err(e) => {
                    log::warn!("Failed to restore task {}: {}. Marking as failed.", task.id, e);

                    // Mark task as failed in database
                    let mut failed_task = task.clone();
                    failed_task.status = DownloadStatus::Failed(format!("Recovery failed: {}", e));
                    failed_task.updated_at = std::time::SystemTime::now();

                    if let Err(save_err) = self.repository.save_task(&failed_task).await {
                        log::error!("Failed to save failed task status: {}", save_err);
                    }
                }
            }
        }

        Ok(())
    }

    /// Restore a single task to aria2
    async fn restore_single_task(&self, task: &DownloadTask) -> Result<String> {
        // Re-add the download to aria2
        let restored_id = DownloadManagerTrait::add_download(&*self.aria2,
            task.url.clone(),
            task.target_path.clone()
        ).await?;

        // Get the GID for this restored task
        let gid = self.get_gid_for_task(restored_id).await?;

        // Apply original status if it was paused
        if task.status == DownloadStatus::Paused {
            DownloadManagerTrait::pause_download(&*self.aria2, restored_id).await?;
        }

        Ok(gid)
    }

    /// Get the aria2 GID for a given task ID
    async fn get_gid_for_task(&self, task_id: TaskId) -> Result<String> {
        // This would need to be implemented based on how aria2 manager handles task->GID mapping
        // For now, we'll use the task_id as a string representation
        // In a real implementation, this would query the aria2 manager's internal state

        // Get the task from aria2 to find its GID
        let _task = DownloadManagerTrait::get_task(&*self.aria2, task_id).await?;

        // The aria2 manager should provide a way to get GID, for now we use task_id
        Ok(task_id.to_string())
    }

    /// Store task mapping between TaskId and aria2 GID
    async fn store_task_mapping(&self, task_id: TaskId, gid: String) {
        let mut mapping = self.task_mapping.write().await;
        mapping.insert(task_id, gid);
        log::debug!("Stored mapping: {} -> {}", task_id, mapping.get(&task_id).unwrap());
    }

    /// Remove task mapping
    async fn remove_task_mapping(&self, task_id: TaskId) {
        let mut mapping = self.task_mapping.write().await;
        mapping.remove(&task_id);
        log::debug!("Removed mapping for task: {}", task_id);
    }


    /// Internal method to create a new download without duplicate checking
    async fn create_new_download(&self, url: String, target_path: PathBuf) -> Result<TaskId> {
        log::info!("Adding download: {} -> {}", url, target_path.display());

        // Ensure target directory exists
        if let Some(parent) = target_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Add to aria2
        let task_id = DownloadManagerTrait::add_download(&*self.aria2, url.clone(), target_path.clone()).await?;

        // Get the created task and save to database
        let task = DownloadManagerTrait::get_task(&*self.aria2, task_id).await?;
        self.repository.save_task(&task).await
            .map_err(|e| anyhow::anyhow!("Failed to persist task to database: {}", e))?;

        // Get and store GID mapping
        match self.get_gid_for_task(task_id).await {
            Ok(gid) => {
                self.store_task_mapping(task_id, gid).await;
            }
            Err(e) => {
                log::warn!("Failed to get GID for task {}: {}", task_id, e);
            }
        }

        log::info!("Successfully added download with task ID: {}", task_id);
        Ok(task_id)
    }

    /// Start the background persistence poller
    async fn start_persistence_poller(&self) {
        let aria2 = self.aria2.clone();
        let repository = self.repository.clone();
        let shutdown = self.shutdown.clone();
        let persistence_handle = self.persistence_handle.clone();
        let task_mapping = self.task_mapping.clone();

        let handle = tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(STATUS_POLL_INTERVAL_SECS));
            let mut poll_count: u64 = 0;

            log::info!("Starting persistence poller");

            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        poll_count += 1;

                        // Get all active task IDs
                        let active_task_ids = {
                            let mapping = task_mapping.read().await;
                            mapping.keys().cloned().collect::<Vec<_>>()
                        };

                        for task_id in active_task_ids {
                            // Check status changes every second
                            if let Ok(current_task) = DownloadManagerTrait::get_task(&*aria2, task_id).await {
                                // Always save task to capture status changes
                                if let Err(e) = repository.save_task(&current_task).await {
                                    log::error!("Failed to save task {}: {}", task_id, e);
                                }

                                // Save progress every 5 seconds
                                if poll_count % PROGRESS_SAVE_INTERVAL_SECS == 0 {
                                    if let Ok(progress) = DownloadManagerTrait::get_progress(&*aria2, task_id).await {
                                        if let Err(e) = repository.save_progress(&task_id, &progress).await {
                                            log::error!("Failed to save progress for task {}: {}", task_id, e);
                                        }
                                    }
                                }
                            }
                        }

                        // Log progress save cycles
                        if poll_count % PROGRESS_SAVE_INTERVAL_SECS == 0 {
                            log::debug!("Progress save cycle completed");
                        }
                    }
                    _ = shutdown.notified() => {
                        log::info!("Persistence poller shutting down");
                        break;
                    }
                }
            }

            log::info!("Persistence poller stopped");
        });

        // Store the handle
        let mut handle_lock = persistence_handle.write().await;
        *handle_lock = Some(handle);

        log::info!("Persistence poller started");
    }

    /// Save all current tasks to database
    async fn save_all_tasks(&self) -> Result<()> {
        let tasks = DownloadManagerTrait::list_tasks(&*self.aria2).await?;

        log::info!("Saving {} tasks to database", tasks.len());

        for task in tasks {
            if let Err(e) = self.repository.save_task(&task).await {
                log::error!("Failed to save task {} during shutdown: {}", task.id, e);
            }

            if let Ok(progress) = DownloadManagerTrait::get_progress(&*self.aria2, task.id).await {
                if let Err(e) = self.repository.save_progress(&task.id, &progress).await {
                    log::error!("Failed to save progress for task {} during shutdown: {}", task.id, e);
                }
            }
        }

        Ok(())
    }

    /// Gracefully shutdown the manager
    pub async fn shutdown(&self) -> Result<()> {
        log::info!("Shutting down PersistentAria2Manager");

        // Notify shutdown
        self.shutdown.notify_one();

        // Wait for persistence poller to finish
        if let Some(handle) = self.persistence_handle.write().await.take() {
            let _ = handle.await;
        }

        // Final save of all tasks
        self.save_all_tasks().await?;

        log::info!("PersistentAria2Manager shutdown complete");
        Ok(())
    }
}

#[async_trait]
impl DownloadManager for PersistentAria2Manager {
    async fn add_download(&self, url: String, target_path: PathBuf) -> Result<TaskId> {
        // Use duplicate detection with default policy (ReuseExisting)
        match self.add_download_with_policy(&url, &target_path, DuplicatePolicy::default()).await? {
            DuplicateResult::NotFound { .. } => {
                // No duplicate found, create new task
                self.create_new_download(url, target_path).await
            }
            DuplicateResult::Found { task_id, .. } => {
                // Duplicate found, return existing task ID
                Ok(task_id)
            }
            DuplicateResult::NewTask(task_id) => Ok(task_id),
            DuplicateResult::ExistingTask { task_id, .. } => Ok(task_id),
            DuplicateResult::RequiresDecision { .. } => {
                // For backwards compatibility, fallback to creating new task
                log::warn!("Duplicate detection requires decision, creating new task anyway");
                let task_id = self.create_new_download(url, target_path).await?;
                Ok(task_id)
            }
        }
    }

    async fn pause_download(&self, task_id: TaskId) -> Result<()> {
        log::info!("Pausing download: {}", task_id);

        // Pause in aria2
        DownloadManagerTrait::pause_download(&*self.aria2, task_id).await?;

        // Update status in database immediately for consistency
        if let Ok(task) = DownloadManagerTrait::get_task(&*self.aria2, task_id).await {
            if let Err(e) = self.repository.save_task(&task).await {
                log::error!("Failed to save paused task status: {}", e);
            }
        }

        Ok(())
    }

    async fn resume_download(&self, task_id: TaskId) -> Result<()> {
        log::info!("Resuming download: {}", task_id);

        // Resume in aria2
        DownloadManagerTrait::resume_download(&*self.aria2, task_id).await?;

        // Update status in database immediately for consistency
        if let Ok(task) = DownloadManagerTrait::get_task(&*self.aria2, task_id).await {
            if let Err(e) = self.repository.save_task(&task).await {
                log::error!("Failed to save resumed task status: {}", e);
            }
        }

        Ok(())
    }

    async fn cancel_download(&self, task_id: TaskId) -> Result<()> {
        log::info!("Canceling download: {}", task_id);

        // Cancel in aria2
        DownloadManagerTrait::cancel_download(&*self.aria2, task_id).await?;

        // Remove from database
        if let Err(e) = self.repository.delete_task(&task_id).await {
            log::error!("Failed to delete task from database: {}", e);
        }
        if let Err(e) = self.repository.delete_progress(&task_id).await {
            log::error!("Failed to delete progress from database: {}", e);
        }

        // Remove mapping
        self.remove_task_mapping(task_id).await;

        Ok(())
    }

    async fn get_progress(&self, task_id: TaskId) -> Result<DownloadProgress> {
        // Always get fresh data from aria2
        DownloadManagerTrait::get_progress(&*self.aria2, task_id).await
    }

    async fn get_task(&self, task_id: TaskId) -> Result<DownloadTask> {
        // Always get fresh data from aria2
        DownloadManagerTrait::get_task(&*self.aria2, task_id).await
    }

    async fn list_tasks(&self) -> Result<Vec<DownloadTask>> {
        // Get from aria2 for most current state
        DownloadManagerTrait::list_tasks(&*self.aria2).await
    }

    async fn active_download_count(&self) -> Result<usize> {
        DownloadManagerTrait::active_download_count(&*self.aria2).await
    }

    // Duplicate detection methods

    async fn find_duplicate_task(
        &self,
        url: &str,
        target_path: &Path,
    ) -> Result<Option<TaskId>> {
        // Create file identifier for duplicate detection
        let _identifier = FileIdentifier::new(url, target_path, None);

        // First check active tasks in aria2
        let active_tasks = DownloadManagerTrait::list_tasks(&*self.aria2).await?;
        for task in &active_tasks {
            if task.url == url && task.target_path == target_path {
                return Ok(Some(task.id));
            }
        }

        // If not found in active tasks, check database for all tasks
        // This allows finding paused/failed tasks that can be resumed
        match self.repository.list_tasks().await {
            Ok(all_tasks) => {
                for task in all_tasks {
                    if task.url == url && task.target_path == target_path {
                        return Ok(Some(task.id));
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to query database for duplicates: {}", e);
                // Continue with no duplicates found rather than failing
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
            // Try to get task from aria2 first (active tasks)
            let task_result = DownloadManagerTrait::get_task(&*self.aria2, existing_task_id).await;

            let task_status = match task_result {
                Ok(task) => TaskStatus::from_download_status(task.status),
                Err(_) => {
                    // Task not in aria2, check database
                    match self.repository.get_task(&existing_task_id).await {
                        Ok(task) => TaskStatus::from_download_status(task.status),
                        Err(_) => {
                            // Task not found anywhere, treat as no duplicate
                            return self.add_download_with_policy(url, target_path, DuplicatePolicy::AllowDuplicate).await;
                        }
                    }
                }
            };

            if policy.allows_reuse(&task_status) {
                // If task is paused or failed, we might want to resume it
                match task_status {
                    TaskStatus::Paused => {
                        log::info!("Resuming paused duplicate task: {}", existing_task_id);
                        let _ = self.resume_download(existing_task_id).await;
                    }
                    TaskStatus::Failed(_) => {
                        log::info!("Retrying failed duplicate task: {}", existing_task_id);
                        let _ = self.resume_download(existing_task_id).await;
                    }
                    _ => {}
                }

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
        let task_id = self.create_new_download(url.to_string(), target_path.to_path_buf()).await?;
        Ok(DuplicateResult::NewTask(task_id))
    }

    async fn verify_task_validity(&self, task_id: &TaskId) -> Result<bool> {
        // Check if task exists in aria2 (active)
        if DownloadManagerTrait::get_task(&*self.aria2, *task_id).await.is_ok() {
            return Ok(true);
        }

        // Check if task exists in database (inactive but valid)
        match self.repository.get_task(task_id).await {
            Ok(task) => {
                // Task exists in database, check if target file exists for completed tasks
                if matches!(task.status, DownloadStatus::Completed) {
                    // For completed tasks, verify the file still exists
                    Ok(tokio::fs::metadata(&task.target_path).await.is_ok())
                } else {
                    // For incomplete tasks, consider them valid if they exist in database
                    Ok(true)
                }
            }
            Err(_) => Ok(false),
        }
    }

    async fn get_duplicate_candidates(
        &self,
        url: &str,
        target_path: &Path,
    ) -> Result<Vec<TaskId>> {
        let mut candidates = Vec::new();

        // Check active tasks in aria2
        if let Ok(active_tasks) = DownloadManagerTrait::list_tasks(&*self.aria2).await {
            for task in &active_tasks {
                if task.url == url && task.target_path == target_path {
                    candidates.push(task.id);
                }
            }
        }

        // Check all tasks in database
        if let Ok(all_tasks) = self.repository.list_tasks().await {
            for task in all_tasks {
                if task.url == url && task.target_path == target_path && !candidates.contains(&task.id) {
                    candidates.push(task.id);
                }
            }
        }

        // Sort by most recent first (assuming TaskId has some time component)
        // For now, just return as-is since TaskId doesn't expose creation time
        Ok(candidates)
    }
}

impl Drop for PersistentAria2Manager {
    fn drop(&mut self) {
        // Attempt final save (best effort, can't await in drop)
        let repository = self.repository.clone();
        let aria2 = self.aria2.clone();

        tokio::spawn(async move {
            if let Ok(tasks) = DownloadManagerTrait::list_tasks(&*aria2).await {
                for task in tasks {
                    let _ = repository.save_task(&task).await;
                }
            }
        });

        log::debug!("PersistentAria2Manager dropped");
    }
}
