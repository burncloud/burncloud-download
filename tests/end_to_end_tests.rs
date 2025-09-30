//! End-to-end tests for burncloud-download manager
//!
//! These tests validate complete user workflows and ensure the examples work correctly.

use burncloud_download::{
    DownloadManager, TaskQueueManager, BasicDownloadManager,
    DownloadEventHandler, DownloadStatus, DownloadProgress, TaskId
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use async_trait::async_trait;

/// Test the basic usage example from the documentation
mod example_validation {
    use super::*;

    #[tokio::test]
    async fn test_basic_manager_example() {
        // This test validates the exact usage pattern shown in documentation
        let manager: Arc<dyn DownloadManager> = Arc::new(BasicDownloadManager::new());

        // Add a download task (from documentation)
        let task_id = manager.add_download(
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/downloads/file.zip")
        ).await.expect("Failed to add download");

        // Monitor progress (from documentation)
        let progress = manager.get_progress(task_id).await.expect("Failed to get progress");
        println!("Downloaded: {} / {} bytes",
            progress.downloaded_bytes,
            progress.total_bytes.unwrap_or(0)
        );

        // Verify the task was created correctly
        let task = manager.get_task(task_id).await.expect("Failed to get task");
        assert_eq!(task.url, "https://example.com/file.zip");
        assert_eq!(task.target_path, PathBuf::from("/downloads/file.zip"));

        // Clean up
        manager.cancel_download(task_id).await.expect("Failed to cancel");
    }

    #[tokio::test]
    async fn test_queue_manager_example() {
        // This test validates the exact usage pattern shown in documentation
        let manager: Arc<dyn DownloadManager> = Arc::new(TaskQueueManager::new());

        // Add a download task (from documentation)
        let task_id = manager.add_download(
            "https://example.com/file.zip".to_string(),
            PathBuf::from("/downloads/file.zip")
        ).await.expect("Failed to add download");

        // Monitor progress (from documentation)
        let task = manager.get_task(task_id).await.expect("Failed to get task");
        println!("Task status: {}", task.status);

        // Verify the task was created correctly
        assert_eq!(task.url, "https://example.com/file.zip");
        assert_eq!(task.target_path, PathBuf::from("/downloads/file.zip"));

        // Clean up
        manager.cancel_download(task_id).await.expect("Failed to cancel");
    }

    #[tokio::test]
    async fn test_event_handler_example() {
        // Test the event handler pattern from the examples
        struct TestLoggingHandler {
            logs: Arc<Mutex<Vec<String>>>,
        }

        #[async_trait]
        impl DownloadEventHandler for TestLoggingHandler {
            async fn on_status_changed(&self, task_id: TaskId, old_status: DownloadStatus, new_status: DownloadStatus) {
                let mut logs = self.logs.lock().await;
                logs.push(format!("Status changed for {}: {} -> {}", task_id, old_status, new_status));
            }

            async fn on_progress_updated(&self, task_id: TaskId, progress: DownloadProgress) {
                let mut logs = self.logs.lock().await;
                if let Some(percentage) = progress.completion_percentage() {
                    logs.push(format!("Progress for {}: {:.1}%", task_id, percentage));
                }
            }

            async fn on_download_completed(&self, task_id: TaskId) {
                let mut logs = self.logs.lock().await;
                logs.push(format!("Completed: {}", task_id));
            }

            async fn on_download_failed(&self, task_id: TaskId, error: String) {
                let mut logs = self.logs.lock().await;
                logs.push(format!("Failed {}: {}", task_id, error));
            }
        }

        let queue_manager = TaskQueueManager::new();
        let logs = Arc::new(Mutex::new(Vec::new()));
        let handler = Arc::new(TestLoggingHandler { logs: logs.clone() });

        queue_manager.add_event_handler(handler).await;

        // Add and manage a task
        let task_id = queue_manager.add_download(
            "https://example.com/event-test.zip".to_string(),
            PathBuf::from("/downloads/event-test.zip")
        ).await.expect("Failed to add download");

        // Pause and resume to generate events
        queue_manager.pause_download(task_id).await.expect("Failed to pause");
        queue_manager.resume_download(task_id).await.expect("Failed to resume");

        // Complete the task
        queue_manager.complete_task(task_id).await.expect("Failed to complete");

        // Verify events were logged
        sleep(Duration::from_millis(50)).await; // Let events process
        let logged_events = logs.lock().await;
        assert!(!logged_events.is_empty());
        assert!(logged_events.iter().any(|log| log.contains("Status changed")));
        assert!(logged_events.iter().any(|log| log.contains("Completed")));
    }
}

/// Test complete user workflows
mod user_workflows {
    use super::*;

    #[tokio::test]
    async fn test_download_batch_workflow() {
        let manager = Arc::new(TaskQueueManager::new());

        // User adds multiple downloads
        let files = vec![
            ("https://example.com/document.pdf", "/downloads/documents/report.pdf"),
            ("https://example.com/presentation.pptx", "/downloads/documents/presentation.pptx"),
            ("https://example.com/video.mp4", "/downloads/media/video.mp4"),
            ("https://example.com/music.mp3", "/downloads/media/music.mp3"),
            ("https://example.com/software.zip", "/downloads/software/app.zip"),
        ];

        let mut task_ids = Vec::new();
        for (url, path) in &files {
            let task_id = manager.add_download(
                url.to_string(),
                PathBuf::from(path)
            ).await.expect("Failed to add download");
            task_ids.push(task_id);
        }

        // Verify only 3 are active (concurrency limit)
        assert_eq!(manager.active_download_count().await, 3);

        // User checks progress on all downloads
        for &task_id in &task_ids {
            let task = manager.get_task(task_id).await.expect("Failed to get task");
            let progress = manager.get_progress(task_id).await.expect("Failed to get progress");

            println!("Task {}: {} - {} bytes", task_id, task.status, progress.downloaded_bytes);

            // Verify task has correct URL from our test data
            assert!(files.iter().any(|(url, _)| task.url == *url));
        }

        // User pauses some downloads
        if task_ids.len() >= 2 {
            manager.pause_download(task_ids[1]).await.expect("Failed to pause");

            let paused_task = manager.get_task(task_ids[1]).await.expect("Failed to get task");
            assert_eq!(paused_task.status, DownloadStatus::Paused);
        }

        // Complete some downloads to test queue progression
        manager.complete_task(task_ids[0]).await.expect("Failed to complete");

        // Give time for queue processing
        sleep(Duration::from_millis(50)).await;

        // Should still have 3 active (queue progression)
        assert_eq!(manager.active_download_count().await, 3);

        // User resumes paused download
        if task_ids.len() >= 2 {
            manager.resume_download(task_ids[1]).await.expect("Failed to resume");
        }

        // Clean up remaining
        for &task_id in &task_ids[1..] {
            let _ = manager.cancel_download(task_id).await;
        }
    }

    #[tokio::test]
    async fn test_user_interruption_workflow() {
        let manager = Arc::new(BasicDownloadManager::new());

        // User starts a large download
        let task_id = manager.add_download(
            "https://example.com/large-file.iso".to_string(),
            PathBuf::from("/downloads/large-file.iso")
        ).await.expect("Failed to add download");

        // User monitors progress
        let initial_progress = manager.get_progress(task_id).await.expect("Failed to get progress");
        assert_eq!(initial_progress.downloaded_bytes, 0);

        // Simulate progress update in BasicDownloadManager by waiting
        sleep(Duration::from_millis(100)).await;

        // User pauses due to network issues
        manager.pause_download(task_id).await.expect("Failed to pause");
        let paused_task = manager.get_task(task_id).await.expect("Failed to get task");
        assert_eq!(paused_task.status, DownloadStatus::Paused);

        // User resumes later
        manager.resume_download(task_id).await.expect("Failed to resume");
        let resumed_task = manager.get_task(task_id).await.expect("Failed to get task");
        assert_eq!(resumed_task.status, DownloadStatus::Downloading);

        // User checks final progress
        let final_progress = manager.get_progress(task_id).await.expect("Failed to get progress");
        assert!(final_progress.speed_bps > 0);
        assert!(final_progress.total_bytes.is_some());

        // Clean up
        manager.cancel_download(task_id).await.expect("Failed to cancel");
    }

    #[tokio::test]
    async fn test_mixed_success_failure_workflow() {
        let manager = TaskQueueManager::new();

        // Add multiple downloads
        let mut task_ids = Vec::new();
        for i in 0..4 {
            let task_id = manager.add_download(
                format!("https://example.com/file{}.zip", i),
                PathBuf::from(format!("/downloads/file{}.zip", i))
            ).await.expect("Failed to add download");
            task_ids.push(task_id);
        }

        // Simulate some succeeding
        manager.complete_task(task_ids[0]).await.expect("Failed to complete");
        manager.complete_task(task_ids[1]).await.expect("Failed to complete");

        // Simulate some failing
        manager.fail_task(task_ids[2], "Network error".to_string()).await.expect("Failed to fail");

        // User lists all tasks to check results
        let tasks = manager.list_tasks().await.expect("Failed to list tasks");

        let completed_count = tasks.iter().filter(|t| t.status == DownloadStatus::Completed).count();
        let failed_count = tasks.iter().filter(|t| matches!(t.status, DownloadStatus::Failed(_))).count();

        assert_eq!(completed_count, 2);
        assert_eq!(failed_count, 1);

        // User retries failed download
        let failed_task = tasks.iter().find(|t| matches!(t.status, DownloadStatus::Failed(_))).unwrap();
        manager.resume_download(failed_task.id).await.expect("Failed to retry");

        // Clean up
        for &task_id in &task_ids {
            let _ = manager.cancel_download(task_id).await;
        }
    }
}

/// Test stress scenarios and edge cases
mod stress_tests {
    use super::*;

    #[tokio::test]
    async fn test_rapid_add_remove_cycle() {
        let manager = Arc::new(TaskQueueManager::new());

        // Rapidly add and remove downloads
        for cycle in 0..10 {
            let mut current_tasks = Vec::new();

            // Add batch
            for i in 0..5 {
                let task_id = manager.add_download(
                    format!("https://example.com/cycle{}-file{}.zip", cycle, i),
                    PathBuf::from(format!("/tmp/cycle{}-file{}.zip", cycle, i))
                ).await.expect("Failed to add download");
                current_tasks.push(task_id);
            }

            // Verify batch was added
            let tasks = manager.list_tasks().await.expect("Failed to list tasks");
            assert!(tasks.len() >= current_tasks.len());

            // Remove batch
            for task_id in current_tasks {
                manager.cancel_download(task_id).await.expect("Failed to cancel");
            }

            // Verify cleanup
            sleep(Duration::from_millis(10)).await; // Let cleanup finish
        }

        // Final verification - should be clean
        let final_tasks = manager.list_tasks().await.expect("Failed to list tasks");
        assert_eq!(final_tasks.len(), 0);
    }

    #[tokio::test]
    async fn test_concurrent_user_operations() {
        let manager = Arc::new(TaskQueueManager::new());

        // Simulate multiple users operating concurrently
        let mut handles = Vec::new();

        for user_id in 0..5 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                let mut user_tasks = Vec::new();

                // Each user adds downloads
                for file_num in 0..3 {
                    let task_id = manager_clone.add_download(
                        format!("https://user{}.example.com/file{}.zip", user_id, file_num),
                        PathBuf::from(format!("/downloads/user{}/file{}.zip", user_id, file_num))
                    ).await.expect("Failed to add download");
                    user_tasks.push(task_id);
                }

                // User operations
                for &task_id in &user_tasks {
                    // Check task status
                    let _ = manager_clone.get_task(task_id).await;
                    let _ = manager_clone.get_progress(task_id).await;

                    // Some pause/resume operations
                    if user_tasks.len() > 1 {
                        let _ = manager_clone.pause_download(task_id).await;
                        sleep(Duration::from_millis(10)).await;
                        let _ = manager_clone.resume_download(task_id).await;
                    }
                }

                // User cleans up their downloads
                for task_id in user_tasks {
                    manager_clone.cancel_download(task_id).await.expect("Failed to cancel");
                }

                user_id
            });
            handles.push(handle);
        }

        // Wait for all users to complete
        for handle in handles {
            handle.await.expect("User task panicked");
        }

        // Verify system is in clean state
        let final_tasks = manager.list_tasks().await.expect("Failed to list tasks");
        assert_eq!(final_tasks.len(), 0);

        let active_count = manager.active_download_count().await;
        assert_eq!(active_count, 0);
    }

    #[tokio::test]
    async fn test_memory_pressure_simulation() {
        let manager = Arc::new(TaskQueueManager::new());

        // Create many tasks to simulate memory pressure
        let mut all_task_ids = Vec::new();

        for batch in 0..10 {
            let mut batch_tasks = Vec::new();

            // Add batch of downloads
            for i in 0..20 {
                let task_id = manager.add_download(
                    format!("https://example.com/batch{}/large-file{}.zip", batch, i),
                    PathBuf::from(format!("/downloads/batch{}/large-file{}.zip", batch, i))
                ).await.expect("Failed to add download");
                batch_tasks.push(task_id);
            }

            all_task_ids.extend(batch_tasks.clone());

            // Simulate progress updates (memory allocation)
            for (idx, &task_id) in batch_tasks.iter().enumerate() {
                let progress = DownloadProgress {
                    downloaded_bytes: (batch * 1000 + idx * 100) as u64,
                    total_bytes: Some(100000),
                    speed_bps: 1024,
                    eta_seconds: Some(90),
                };
                let _ = manager.update_progress(task_id, progress).await;
            }

            // Verify system remains stable
            let tasks = manager.list_tasks().await.expect("Failed to list tasks");
            assert_eq!(tasks.len(), all_task_ids.len());

            // Periodically clean up some tasks
            if batch % 3 == 2 {
                for &task_id in &batch_tasks[..10] {
                    manager.cancel_download(task_id).await.expect("Failed to cancel");
                    all_task_ids.retain(|&id| id != task_id);
                }
            }
        }

        // Final cleanup
        for task_id in all_task_ids {
            let _ = manager.cancel_download(task_id).await;
        }
    }
}

/// Test backwards compatibility and API stability
mod compatibility_tests {
    use super::*;

    #[tokio::test]
    async fn test_api_interface_stability() {
        // Test that both managers implement the same interface consistently

        async fn test_manager_interface(manager: Arc<dyn DownloadManager>, name: &str) {
            println!("Testing {} interface", name);

            // Core operations that should work identically
            let task_id = manager.add_download(
                "https://example.com/api-test.zip".to_string(),
                PathBuf::from("/tmp/api-test.zip")
            ).await.expect("add_download failed");

            let task = manager.get_task(task_id).await.expect("get_task failed");
            assert_eq!(task.id, task_id);

            let progress = manager.get_progress(task_id).await.expect("get_progress failed");
            assert!(progress.downloaded_bytes >= 0);

            let tasks = manager.list_tasks().await.expect("list_tasks failed");
            assert!(!tasks.is_empty());

            let count = manager.active_download_count().await.expect("active_download_count failed");
            assert!(count >= 0);

            // State transitions should work consistently
            if task.status.can_pause() {
                manager.pause_download(task_id).await.expect("pause_download failed");
                let paused_task = manager.get_task(task_id).await.expect("get_task after pause failed");
                assert_eq!(paused_task.status, DownloadStatus::Paused);

                manager.resume_download(task_id).await.expect("resume_download failed");
            }

            manager.cancel_download(task_id).await.expect("cancel_download failed");
        }

        // Test both implementations
        let basic_manager: Arc<dyn DownloadManager> = Arc::new(BasicDownloadManager::new());
        let queue_manager: Arc<dyn DownloadManager> = Arc::new(TaskQueueManager::new());

        test_manager_interface(basic_manager, "BasicDownloadManager").await;
        test_manager_interface(queue_manager, "TaskQueueManager").await;
    }

    #[tokio::test]
    async fn test_error_consistency() {
        // Both managers should handle errors consistently
        let basic_manager: Arc<dyn DownloadManager> = Arc::new(BasicDownloadManager::new());
        let queue_manager: Arc<dyn DownloadManager> = Arc::new(TaskQueueManager::new());

        let managers = vec![
            ("Basic", basic_manager),
            ("Queue", queue_manager),
        ];

        for (name, manager) in managers {
            println!("Testing {} error handling", name);

            let non_existent_id = TaskId::new();

            // All should return errors for non-existent tasks
            assert!(manager.get_task(non_existent_id).await.is_err());
            assert!(manager.pause_download(non_existent_id).await.is_err());
            assert!(manager.resume_download(non_existent_id).await.is_err());

            // Cancel should be idempotent (not error)
            assert!(manager.cancel_download(non_existent_id).await.is_ok());
        }
    }

    #[tokio::test]
    async fn test_data_type_consistency() {
        let manager = TaskQueueManager::new();

        let task_id = manager.add_download(
            "https://example.com/consistency-test.zip".to_string(),
            PathBuf::from("/tmp/consistency-test.zip")
        ).await.expect("Failed to add download");

        // Test that data types are consistent across operations
        let task1 = manager.get_task(task_id).await.expect("Failed to get task");
        let task2 = manager.get_task(task_id).await.expect("Failed to get task again");

        // Tasks should be identical
        assert_eq!(task1.id, task2.id);
        assert_eq!(task1.url, task2.url);
        assert_eq!(task1.target_path, task2.target_path);
        assert_eq!(task1.status, task2.status);

        // Progress should be consistent
        let progress1 = manager.get_progress(task_id).await.expect("Failed to get progress");
        let progress2 = manager.get_progress(task_id).await.expect("Failed to get progress again");

        assert_eq!(progress1.downloaded_bytes, progress2.downloaded_bytes);
        assert_eq!(progress1.total_bytes, progress2.total_bytes);

        manager.cancel_download(task_id).await.expect("Failed to cancel");
    }
}