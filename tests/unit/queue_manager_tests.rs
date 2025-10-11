use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::Mutex;
use async_trait::async_trait;

use burncloud_download::types::{TaskId, DownloadStatus, DownloadProgress};
use burncloud_download::traits::{DownloadEventHandler, DownloadManager};
use burncloud_download::queue::manager::TaskQueueManager;

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

    // Verify task statuses - first 3 should be downloading
    for i in 0..3 {
        let task = manager.get_task(task_ids[i]).await.unwrap();
        assert_eq!(task.status, DownloadStatus::Downloading, "Task {} should be downloading", i);
    }

    // Last 2 should be waiting
    for i in 3..5 {
        let task = manager.get_task(task_ids[i]).await.unwrap();
        assert_eq!(task.status, DownloadStatus::Waiting, "Task {} should be waiting", i);
    }

    // Complete first task
    manager.complete_task(task_ids[0]).await.unwrap();

    // Should still have 3 active (one completed, one started from queue)
    assert_eq!(manager.active_download_count().await, 3);

    // The 4th task should now be downloading (moved from queue)
    let task = manager.get_task(task_ids[3]).await.unwrap();
    assert_eq!(task.status, DownloadStatus::Downloading, "Task 3 should now be downloading");
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