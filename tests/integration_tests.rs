//! Comprehensive integration tests for burncloud-download manager
//!
//! This test suite focuses on functional validation and real-world scenarios
//! to ensure the download manager implementations work correctly in production.

use burncloud_download::{
    DownloadManager, TaskQueueManager, BasicDownloadManager,
    DownloadEventHandler, DownloadStatus, DownloadProgress, TaskId
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration, timeout};
use async_trait::async_trait;

/// Test event handler that captures all events for verification
#[derive(Clone)]
struct TestEventCapture {
    events: Arc<Mutex<Vec<TestEvent>>>,
}

#[derive(Debug, Clone)]
enum TestEvent {
    StatusChanged { task_id: TaskId, old_status: DownloadStatus, new_status: DownloadStatus },
    ProgressUpdated { task_id: TaskId, progress: DownloadProgress },
    DownloadCompleted { task_id: TaskId },
    DownloadFailed { task_id: TaskId, error: String },
}

impl TestEventCapture {
    fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn get_events(&self) -> Vec<TestEvent> {
        self.events.lock().await.clone()
    }

    async fn clear_events(&self) {
        self.events.lock().await.clear();
    }

    async fn wait_for_event_count(&self, count: usize, timeout_ms: u64) -> bool {
        let start = std::time::Instant::now();
        loop {
            if self.events.lock().await.len() >= count {
                return true;
            }
            if start.elapsed().as_millis() > timeout_ms as u128 {
                return false;
            }
            sleep(Duration::from_millis(10)).await;
        }
    }
}

#[async_trait]
impl DownloadEventHandler for TestEventCapture {
    async fn on_status_changed(&self, task_id: TaskId, old_status: DownloadStatus, new_status: DownloadStatus) {
        let mut events = self.events.lock().await;
        events.push(TestEvent::StatusChanged { task_id, old_status, new_status });
    }

    async fn on_progress_updated(&self, task_id: TaskId, progress: DownloadProgress) {
        let mut events = self.events.lock().await;
        events.push(TestEvent::ProgressUpdated { task_id, progress });
    }

    async fn on_download_completed(&self, task_id: TaskId) {
        let mut events = self.events.lock().await;
        events.push(TestEvent::DownloadCompleted { task_id });
    }

    async fn on_download_failed(&self, task_id: TaskId, error: String) {
        let mut events = self.events.lock().await;
        events.push(TestEvent::DownloadFailed { task_id, error });
    }
}

/// Test trait-based usage patterns that other crates would use
mod trait_based_usage {
    use super::*;

    async fn test_manager_through_trait(manager: Arc<dyn DownloadManager>) -> anyhow::Result<()> {
        // Test basic lifecycle through trait
        let task_id = manager.add_download(
            "https://example.com/test-file.zip".to_string(),
            PathBuf::from("/tmp/test-file.zip")
        ).await?;

        // Verify task was created
        let task = manager.get_task(task_id).await?;
        assert_eq!(task.url, "https://example.com/test-file.zip");
        assert!(!task.status.is_finished());

        // Test progress retrieval
        let progress = manager.get_progress(task_id).await?;
        assert!(progress.downloaded_bytes >= 0);

        // Test pause/resume
        if task.status.can_pause() {
            manager.pause_download(task_id).await?;
            let paused_task = manager.get_task(task_id).await?;
            assert_eq!(paused_task.status, DownloadStatus::Paused);

            manager.resume_download(task_id).await?;
            let resumed_task = manager.get_task(task_id).await?;
            assert!(resumed_task.status.can_pause() || resumed_task.status.is_finished());
        }

        // Test listing tasks
        let tasks = manager.list_tasks().await?;
        assert!(!tasks.is_empty());
        assert!(tasks.iter().any(|t| t.id == task_id));

        // Test active count
        let count = manager.active_download_count().await?;
        assert!(count <= 3); // Concurrency limit

        // Clean up
        manager.cancel_download(task_id).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_basic_manager_trait_usage() {
        let manager: Arc<dyn DownloadManager> = Arc::new(BasicDownloadManager::new());
        test_manager_through_trait(manager).await.expect("BasicDownloadManager trait test failed");
    }

    #[tokio::test]
    async fn test_queue_manager_trait_usage() {
        let manager: Arc<dyn DownloadManager> = Arc::new(TaskQueueManager::new());
        test_manager_through_trait(manager).await.expect("TaskQueueManager trait test failed");
    }

    #[tokio::test]
    async fn test_polymorphic_usage() {
        let managers: Vec<Arc<dyn DownloadManager>> = vec![
            Arc::new(BasicDownloadManager::new()),
            Arc::new(TaskQueueManager::new()),
        ];

        for (i, manager) in managers.iter().enumerate() {
            println!("Testing manager {}", i);
            let task_id = manager.add_download(
                format!("https://example.com/file{}.zip", i),
                PathBuf::from(format!("/tmp/file{}.zip", i))
            ).await.expect("Failed to add download");

            let task = manager.get_task(task_id).await.expect("Failed to get task");
            assert_eq!(task.url, format!("https://example.com/file{}.zip", i));

            manager.cancel_download(task_id).await.expect("Failed to cancel download");
        }
    }
}

/// Test concurrency control and task queue management
mod concurrency_tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrent_task_limits() {
        let manager: Arc<dyn DownloadManager> = Arc::new(TaskQueueManager::new());

        // Add more tasks than the concurrency limit (3)
        let mut task_ids = Vec::new();
        for i in 0..6 {
            let task_id = manager.add_download(
                format!("https://example.com/file{}.zip", i),
                PathBuf::from(format!("/tmp/file{}.zip", i))
            ).await.expect("Failed to add download");
            task_ids.push(task_id);
        }

        // Should only have 3 active downloads
        let active_count = manager.active_download_count().await.expect("Failed to get active count");
        assert_eq!(active_count, 3);

        // Verify task statuses - first 3 should be downloading/active, rest waiting
        let tasks = manager.list_tasks().await.expect("Failed to list tasks");
        let active_tasks: Vec<_> = tasks.iter().filter(|t| t.status.is_active()).collect();
        let waiting_tasks: Vec<_> = tasks.iter().filter(|t| t.status == DownloadStatus::Waiting).collect();

        assert_eq!(active_tasks.len(), 3);
        assert_eq!(waiting_tasks.len(), 3);

        // Cancel one active task, should promote one waiting task
        let first_active_id = active_tasks[0].id;
        manager.cancel_download(first_active_id).await.expect("Failed to cancel download");

        // Give time for queue processing
        sleep(Duration::from_millis(50)).await;

        // Should still have 3 active (one cancelled, one promoted)
        let new_active_count = manager.active_download_count().await.expect("Failed to get active count");
        assert_eq!(new_active_count, 3);

        // Clean up
        for task_id in task_ids {
            let _ = manager.cancel_download(task_id).await;
        }
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let manager: Arc<dyn DownloadManager> = Arc::new(TaskQueueManager::new());

        // Spawn multiple concurrent operations
        let mut handles = Vec::new();

        // Add tasks concurrently
        for i in 0..10 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                manager_clone.add_download(
                    format!("https://example.com/concurrent{}.zip", i),
                    PathBuf::from(format!("/tmp/concurrent{}.zip", i))
                ).await
            });
            handles.push(handle);
        }

        // Wait for all additions to complete
        let mut task_ids = Vec::new();
        for handle in handles {
            let task_id = handle.await.expect("Task panicked").expect("Failed to add task");
            task_ids.push(task_id);
        }

        assert_eq!(task_ids.len(), 10);

        // Verify final state
        let tasks = manager.list_tasks().await.expect("Failed to list tasks");
        assert_eq!(tasks.len(), 10);

        let active_count = manager.active_download_count().await.expect("Failed to get active count");
        assert_eq!(active_count, 3); // Concurrency limit

        // Clean up
        for task_id in task_ids {
            let _ = manager.cancel_download(task_id).await;
        }
    }
}

/// Test progress tracking and event system
mod progress_and_events_tests {
    use super::*;

    #[tokio::test]
    async fn test_progress_tracking_accuracy() {
        let manager = TaskQueueManager::new();
        let event_capture = TestEventCapture::new();
        manager.add_event_handler(Arc::new(event_capture.clone())).await;

        let task_id = manager.add_download(
            "https://example.com/progress-test.zip".to_string(),
            PathBuf::from("/tmp/progress-test.zip")
        ).await.expect("Failed to add download");

        // Test progress updates
        let progress_updates = vec![
            DownloadProgress { downloaded_bytes: 1024, total_bytes: Some(10240), speed_bps: 512, eta_seconds: Some(18) },
            DownloadProgress { downloaded_bytes: 5120, total_bytes: Some(10240), speed_bps: 1024, eta_seconds: Some(5) },
            DownloadProgress { downloaded_bytes: 10240, total_bytes: Some(10240), speed_bps: 2048, eta_seconds: Some(0) },
        ];

        for (i, progress) in progress_updates.iter().enumerate() {
            manager.update_progress(task_id, progress.clone()).await
                .expect(&format!("Failed to update progress {}", i));

            let retrieved_progress = manager.get_progress(task_id).await
                .expect("Failed to get progress");

            assert_eq!(retrieved_progress.downloaded_bytes, progress.downloaded_bytes);
            assert_eq!(retrieved_progress.total_bytes, progress.total_bytes);
            assert_eq!(retrieved_progress.speed_bps, progress.speed_bps);
            assert_eq!(retrieved_progress.eta_seconds, progress.eta_seconds);

            // Verify completion percentage calculation
            if let Some(percentage) = retrieved_progress.completion_percentage() {
                let expected = (progress.downloaded_bytes as f64 / progress.total_bytes.unwrap() as f64) * 100.0;
                assert!((percentage - expected).abs() < 0.01);
            }
        }

        // Verify events were fired
        assert!(event_capture.wait_for_event_count(4, 1000).await); // 1 status change + 3 progress updates
        let events = event_capture.get_events().await;
        let progress_events: Vec<_> = events.iter().filter(|e| matches!(e, TestEvent::ProgressUpdated { .. })).collect();
        assert_eq!(progress_events.len(), 3);

        manager.cancel_download(task_id).await.expect("Failed to cancel download");
    }

    #[tokio::test]
    async fn test_event_system_comprehensive() {
        let manager = TaskQueueManager::new();
        let event_capture = TestEventCapture::new();
        manager.add_event_handler(Arc::new(event_capture.clone())).await;

        // Test full lifecycle events
        let task_id = manager.add_download(
            "https://example.com/event-test.zip".to_string(),
            PathBuf::from("/tmp/event-test.zip")
        ).await.expect("Failed to add download");

        // Pause
        manager.pause_download(task_id).await.expect("Failed to pause download");

        // Resume
        manager.resume_download(task_id).await.expect("Failed to resume download");

        // Complete
        manager.complete_task(task_id).await.expect("Failed to complete task");

        // Wait for all events
        assert!(event_capture.wait_for_event_count(4, 1000).await); // waiting->downloading, downloading->paused, paused->downloading, downloading->completed + completion event

        let events = event_capture.get_events().await;

        // Verify we got status change events
        let status_events: Vec<_> = events.iter().filter(|e| matches!(e, TestEvent::StatusChanged { .. })).collect();
        assert!(status_events.len() >= 3);

        // Verify we got completion event
        let completion_events: Vec<_> = events.iter().filter(|e| matches!(e, TestEvent::DownloadCompleted { .. })).collect();
        assert_eq!(completion_events.len(), 1);
    }

    #[tokio::test]
    async fn test_failure_events() {
        let manager = TaskQueueManager::new();
        let event_capture = TestEventCapture::new();
        manager.add_event_handler(Arc::new(event_capture.clone())).await;

        let task_id = manager.add_download(
            "https://example.com/fail-test.zip".to_string(),
            PathBuf::from("/tmp/fail-test.zip")
        ).await.expect("Failed to add download");

        // Simulate failure
        let error_msg = "Network connection lost";
        manager.fail_task(task_id, error_msg.to_string()).await.expect("Failed to fail task");

        // Wait for events
        assert!(event_capture.wait_for_event_count(3, 1000).await); // status change + failure event

        let events = event_capture.get_events().await;

        // Verify failure event
        let failure_events: Vec<_> = events.iter().filter(|e| matches!(e, TestEvent::DownloadFailed { .. })).collect();
        assert_eq!(failure_events.len(), 1);

        if let TestEvent::DownloadFailed { task_id: failed_id, error } = &failure_events[0] {
            assert_eq!(*failed_id, task_id);
            assert_eq!(error, error_msg);
        }

        // Verify task status
        let task = manager.get_task(task_id).await.expect("Failed to get task");
        assert!(matches!(task.status, DownloadStatus::Failed(_)));
    }
}

/// Test error handling across all components
mod error_handling_tests {
    use super::*;

    #[tokio::test]
    async fn test_invalid_task_operations() {
        let manager = TaskQueueManager::new();
        let non_existent_id = TaskId::new();

        // Test operations on non-existent task
        let result = manager.get_task(non_existent_id).await;
        assert!(result.is_err());

        let result = manager.get_progress(non_existent_id).await;
        assert!(result.is_err());

        let result = manager.pause_download(non_existent_id).await;
        assert!(result.is_err());

        let result = manager.resume_download(non_existent_id).await;
        assert!(result.is_err());

        // Cancel non-existent task should not error (idempotent)
        let result = manager.cancel_download(non_existent_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_status_transitions() {
        let manager = TaskQueueManager::new();

        let task_id = manager.add_download(
            "https://example.com/status-test.zip".to_string(),
            PathBuf::from("/tmp/status-test.zip")
        ).await.expect("Failed to add download");

        // Complete the task
        manager.complete_task(task_id).await.expect("Failed to complete task");

        // Try to pause completed task (should fail)
        let result = manager.pause_download(task_id).await;
        assert!(result.is_err());

        // Try to resume completed task (should fail)
        let result = manager.resume_download(task_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_basic_manager_error_conditions() {
        let manager = BasicDownloadManager::new();
        let non_existent_id = TaskId::new();

        // Test all error conditions
        assert!(manager.get_task(non_existent_id).await.is_err());
        assert!(manager.get_progress(non_existent_id).await.is_err());
        assert!(manager.pause_download(non_existent_id).await.is_err());
        assert!(manager.resume_download(non_existent_id).await.is_err());

        // Cancel should be ok (idempotent)
        assert!(manager.cancel_download(non_existent_id).await.is_ok());
    }

    #[tokio::test]
    async fn test_edge_case_inputs() {
        let manager = TaskQueueManager::new();

        // Test with empty URL
        let result = manager.add_download(
            "".to_string(),
            PathBuf::from("/tmp/empty-url.zip")
        ).await;
        // Should succeed (URL validation is not in scope of this manager)
        assert!(result.is_ok());
        if let Ok(task_id) = result {
            manager.cancel_download(task_id).await.expect("Failed to cancel");
        }

        // Test with empty path
        let result = manager.add_download(
            "https://example.com/test.zip".to_string(),
            PathBuf::new()
        ).await;
        assert!(result.is_ok());
        if let Ok(task_id) = result {
            manager.cancel_download(task_id).await.expect("Failed to cancel");
        }
    }
}

/// Test realistic usage scenarios
mod realistic_scenarios {
    use super::*;

    #[tokio::test]
    async fn test_typical_download_workflow() {
        let manager = Arc::new(TaskQueueManager::new());
        let event_capture = TestEventCapture::new();
        manager.add_event_handler(Arc::new(event_capture.clone())).await;

        // Simulate a typical user workflow
        let downloads = vec![
            ("https://example.com/document.pdf", "/downloads/document.pdf"),
            ("https://example.com/video.mp4", "/downloads/video.mp4"),
            ("https://example.com/archive.zip", "/downloads/archive.zip"),
            ("https://example.com/image.jpg", "/downloads/image.jpg"),
        ];

        let mut task_ids = Vec::new();

        // Add all downloads
        for (url, path) in downloads {
            let task_id = manager.add_download(
                url.to_string(),
                PathBuf::from(path)
            ).await.expect("Failed to add download");
            task_ids.push(task_id);
        }

        // Verify initial state
        assert_eq!(manager.active_download_count().await, 3);
        let tasks = manager.list_tasks().await.expect("Failed to list tasks");
        assert_eq!(tasks.len(), 4);

        // Pause one download
        manager.pause_download(task_ids[1]).await.expect("Failed to pause");

        // Simulate progress on active downloads
        for &task_id in &task_ids[0..1] {
            if let Ok(task) = manager.get_task(task_id).await {
                if task.status.is_active() {
                    let progress = DownloadProgress {
                        downloaded_bytes: 2048,
                        total_bytes: Some(10240),
                        speed_bps: 1024,
                        eta_seconds: Some(8),
                    };
                    manager.update_progress(task_id, progress).await.expect("Failed to update progress");
                }
            }
        }

        // Complete some downloads
        manager.complete_task(task_ids[0]).await.expect("Failed to complete");

        // Give time for queue processing
        sleep(Duration::from_millis(50)).await;

        // Resume paused download
        manager.resume_download(task_ids[1]).await.expect("Failed to resume");

        // Verify final state makes sense
        let final_tasks = manager.list_tasks().await.expect("Failed to list tasks");
        let completed_tasks: Vec<_> = final_tasks.iter().filter(|t| t.status == DownloadStatus::Completed).collect();
        assert_eq!(completed_tasks.len(), 1);

        // Clean up
        for task_id in task_ids {
            let _ = manager.cancel_download(task_id).await;
        }
    }

    #[tokio::test]
    async fn test_high_load_scenario() {
        let manager = Arc::new(TaskQueueManager::new());

        // Simulate high load - many downloads added quickly
        let mut handles = Vec::new();
        for i in 0..20 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                manager_clone.add_download(
                    format!("https://cdn.example.com/large-file-{}.zip", i),
                    PathBuf::from(format!("/downloads/large-file-{}.zip", i))
                ).await
            });
            handles.push(handle);
        }

        // Wait for all to complete
        let mut task_ids = Vec::new();
        for handle in handles {
            let result = timeout(Duration::from_secs(5), handle).await;
            let task_id = result.expect("Task timed out").expect("Task panicked").expect("Failed to add task");
            task_ids.push(task_id);
        }

        // Verify system handled the load correctly
        assert_eq!(task_ids.len(), 20);
        assert_eq!(manager.active_download_count().await, 3);

        let tasks = manager.list_tasks().await.expect("Failed to list tasks");
        assert_eq!(tasks.len(), 20);

        // Clean up
        for task_id in task_ids {
            let _ = manager.cancel_download(task_id).await;
        }
    }

    #[tokio::test]
    async fn test_mixed_manager_usage() {
        // Test both managers in the same scenario
        let basic_manager: Arc<dyn DownloadManager> = Arc::new(BasicDownloadManager::new());
        let queue_manager: Arc<dyn DownloadManager> = Arc::new(TaskQueueManager::new());

        let managers = vec![
            ("Basic", basic_manager),
            ("Queue", queue_manager),
        ];

        for (name, manager) in managers {
            println!("Testing {} manager", name);

            // Add downloads
            let task1 = manager.add_download(
                format!("https://example.com/{}-file1.zip", name.to_lowercase()),
                PathBuf::from(format!("/tmp/{}-file1.zip", name.to_lowercase()))
            ).await.expect("Failed to add download 1");

            let task2 = manager.add_download(
                format!("https://example.com/{}-file2.pdf", name.to_lowercase()),
                PathBuf::from(format!("/tmp/{}-file2.pdf", name.to_lowercase()))
            ).await.expect("Failed to add download 2");

            // Test operations
            let tasks = manager.list_tasks().await.expect("Failed to list tasks");
            assert_eq!(tasks.len(), 2);

            let count = manager.active_download_count().await.expect("Failed to get count");
            assert!(count <= 3);

            // Test pause/resume
            if let Ok(task) = manager.get_task(task1).await {
                if task.status.can_pause() {
                    manager.pause_download(task1).await.expect("Failed to pause");
                    manager.resume_download(task1).await.expect("Failed to resume");
                }
            }

            // Clean up
            manager.cancel_download(task1).await.expect("Failed to cancel 1");
            manager.cancel_download(task2).await.expect("Failed to cancel 2");
        }
    }
}

/// Test performance and resource management
mod performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_resource_cleanup() {
        let manager: Arc<dyn DownloadManager> = Arc::new(TaskQueueManager::new());

        // Add many tasks
        let mut task_ids = Vec::new();
        for i in 0..10 {
            let task_id = manager.add_download(
                format!("https://example.com/cleanup-test-{}.zip", i),
                PathBuf::from(format!("/tmp/cleanup-test-{}.zip", i))
            ).await.expect("Failed to add download");
            task_ids.push(task_id);
        }

        // Cancel all tasks
        for task_id in &task_ids {
            manager.cancel_download(*task_id).await.expect("Failed to cancel download");
        }

        // Verify cleanup
        let remaining_tasks = manager.list_tasks().await.expect("Failed to list tasks");
        assert_eq!(remaining_tasks.len(), 0);

        let active_count = manager.active_download_count().await.expect("Failed to get active count");
        assert_eq!(active_count, 0);
    }

    #[tokio::test]
    async fn test_basic_manager_performance() {
        let manager = BasicDownloadManager::new();
        let start = std::time::Instant::now();

        // Test rapid operations
        for i in 0..100 {
            let task_id = manager.add_download(
                format!("https://example.com/perf-test-{}.zip", i),
                PathBuf::from(format!("/tmp/perf-test-{}.zip", i))
            ).await.expect("Failed to add download");

            // Quick operations
            let _ = manager.get_task(task_id).await;
            let _ = manager.get_progress(task_id).await;
            manager.cancel_download(task_id).await.expect("Failed to cancel");
        }

        let elapsed = start.elapsed();
        println!("BasicDownloadManager: 100 add/cancel cycles took {:?}", elapsed);

        // Should complete reasonably quickly (adjust threshold as needed)
        assert!(elapsed < Duration::from_secs(10));
    }

    #[tokio::test]
    async fn test_async_performance() {
        let manager = Arc::new(TaskQueueManager::new());

        // Test concurrent async operations
        let start = std::time::Instant::now();

        let mut handles = Vec::new();
        for i in 0..50 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                // Add task
                let task_id = manager_clone.add_download(
                    format!("https://example.com/async-perf-{}.zip", i),
                    PathBuf::from(format!("/tmp/async-perf-{}.zip", i))
                ).await.expect("Failed to add download");

                // Do some operations
                let _ = manager_clone.get_task(task_id).await;
                let _ = manager_clone.get_progress(task_id).await;

                // Clean up
                manager_clone.cancel_download(task_id).await.expect("Failed to cancel");

                task_id
            });
            handles.push(handle);
        }

        // Wait for all to complete
        for handle in handles {
            handle.await.expect("Task panicked");
        }

        let elapsed = start.elapsed();
        println!("TaskQueueManager: 50 concurrent operations took {:?}", elapsed);

        // Should handle concurrent load well
        assert!(elapsed < Duration::from_secs(15));
    }
}