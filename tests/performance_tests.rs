//! Performance benchmarks and stress tests for burncloud-download
//!
//! These tests validate that the download managers perform well under load
//! and meet performance requirements for production use.

use burncloud_download::{
    DownloadManager, TaskQueueManager, BasicDownloadManager,
    DownloadEventHandler, DownloadStatus, DownloadProgress, TaskId
};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use async_trait::async_trait;

/// Test async performance and responsiveness
mod async_performance {
    use super::*;

    #[tokio::test]
    async fn test_async_operation_performance() {
        let manager = Arc::new(TaskQueueManager::new());

        let start = Instant::now();

        // Test rapid async operations
        let mut handles = Vec::new();
        for i in 0..50 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                let task_id = manager_clone.add_download(
                    format!("https://example.com/async-test-{}.zip", i),
                    PathBuf::from(format!("/tmp/async-test-{}.zip", i))
                ).await?;

                // Perform multiple operations
                let _task = manager_clone.get_task(task_id).await?;
                let _progress = manager_clone.get_progress(task_id).await?;
                let _tasks = manager_clone.list_tasks().await?;
                let _count = manager_clone.active_download_count().await;

                manager_clone.cancel_download(task_id).await?;

                Ok::<TaskId, anyhow::Error>(task_id)
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        for handle in handles {
            handle.await.expect("Task panicked").expect("Task failed");
        }

        let elapsed = start.elapsed();
        println!("50 concurrent async workflows completed in: {:?}", elapsed);

        // Should complete within reasonable time (adjust based on requirements)
        assert!(elapsed < Duration::from_secs(10), "Async operations too slow: {:?}", elapsed);
    }

    #[tokio::test]
    async fn test_concurrent_read_performance() {
        let manager = Arc::new(TaskQueueManager::new());

        // Add some tasks
        let mut task_ids = Vec::new();
        for i in 0..10 {
            let task_id = manager.add_download(
                format!("https://example.com/read-test-{}.zip", i),
                PathBuf::from(format!("/tmp/read-test-{}.zip", i))
            ).await.expect("Failed to add download");
            task_ids.push(task_id);
        }

        let start = Instant::now();

        // Test concurrent read operations
        let mut handles = Vec::new();
        for _ in 0..100 {
            let manager_clone = manager.clone();
            let task_ids_clone = task_ids.clone();
            let handle = tokio::spawn(async move {
                for &task_id in &task_ids_clone {
                    let _ = manager_clone.get_task(task_id).await;
                    let _ = manager_clone.get_progress(task_id).await;
                }
                let _ = manager_clone.list_tasks().await;
                let _ = manager_clone.active_download_count().await;
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.expect("Read task panicked");
        }

        let elapsed = start.elapsed();
        println!("1000 concurrent read operations completed in: {:?}", elapsed);

        // Read operations should be fast
        assert!(elapsed < Duration::from_secs(5), "Read operations too slow: {:?}", elapsed);

        // Clean up
        for task_id in task_ids {
            let _ = manager.cancel_download(task_id).await;
        }
    }

    #[tokio::test]
    async fn test_event_system_performance() {
        struct CountingHandler {
            count: Arc<Mutex<usize>>,
        }

        #[async_trait]
        impl DownloadEventHandler for CountingHandler {
            async fn on_status_changed(&self, _: TaskId, _: DownloadStatus, _: DownloadStatus) {
                *self.count.lock().await += 1;
            }
            async fn on_progress_updated(&self, _: TaskId, _: DownloadProgress) {
                *self.count.lock().await += 1;
            }
            async fn on_download_completed(&self, _: TaskId) {
                *self.count.lock().await += 1;
            }
            async fn on_download_failed(&self, _: TaskId, _: String) {
                *self.count.lock().await += 1;
            }
        }

        let manager = TaskQueueManager::new();
        let event_count = Arc::new(Mutex::new(0));
        let handler = Arc::new(CountingHandler { count: event_count.clone() });
        manager.add_event_handler(handler).await;

        let start = Instant::now();

        // Generate many events
        let mut task_ids = Vec::new();
        for i in 0..20 {
            let task_id = manager.add_download(
                format!("https://example.com/event-perf-{}.zip", i),
                PathBuf::from(format!("/tmp/event-perf-{}.zip", i))
            ).await.expect("Failed to add download");
            task_ids.push(task_id);

            // Generate progress events
            for j in 0..5 {
                let progress = DownloadProgress {
                    downloaded_bytes: (j * 1000) as u64,
                    total_bytes: Some(5000),
                    speed_bps: 1000,
                    eta_seconds: Some(5 - j),
                };
                manager.update_progress(task_id, progress).await.expect("Failed to update progress");
            }

            // Generate status change events
            manager.pause_download(task_id).await.expect("Failed to pause");
            manager.resume_download(task_id).await.expect("Failed to resume");
        }

        let elapsed = start.elapsed();
        println!("Event generation completed in: {:?}", elapsed);

        // Give events time to process
        tokio::time::sleep(Duration::from_millis(100)).await;

        let final_event_count = *event_count.lock().await;
        println!("Generated {} events", final_event_count);

        // Should handle events efficiently
        assert!(elapsed < Duration::from_secs(2), "Event generation too slow: {:?}", elapsed);
        assert!(final_event_count > 0, "No events were generated");

        // Clean up
        for task_id in task_ids {
            let _ = manager.cancel_download(task_id).await;
        }
    }
}

/// Test memory usage and resource management
mod memory_tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_efficiency_many_tasks() {
        let manager = Arc::new(TaskQueueManager::new());

        // Add many tasks to test memory usage
        let mut task_ids = Vec::new();
        for i in 0..1000 {
            let task_id = manager.add_download(
                format!("https://example.com/memory-test-{}.zip", i),
                PathBuf::from(format!("/tmp/memory-test-{}.zip", i))
            ).await.expect("Failed to add download");
            task_ids.push(task_id);

            // Periodically update progress to simulate memory usage
            if i % 100 == 0 {
                let progress = DownloadProgress {
                    downloaded_bytes: (i * 10) as u64,
                    total_bytes: Some(10000),
                    speed_bps: 1000,
                    eta_seconds: Some(10),
                };
                let _ = manager.update_progress(task_id, progress).await;
            }
        }

        // Verify system still responds efficiently
        let start = Instant::now();
        let tasks = manager.list_tasks().await.expect("Failed to list tasks");
        let list_elapsed = start.elapsed();

        assert_eq!(tasks.len(), 1000);
        assert!(list_elapsed < Duration::from_millis(500), "List operation too slow with many tasks: {:?}", list_elapsed);

        let start = Instant::now();
        let count = manager.active_download_count().await;
        let count_elapsed = start.elapsed();

        assert_eq!(count, 3); // Concurrency limit
        assert!(count_elapsed < Duration::from_millis(100), "Count operation too slow: {:?}", count_elapsed);

        // Clean up should be efficient
        let start = Instant::now();
        for task_id in task_ids {
            manager.cancel_download(task_id).await.expect("Failed to cancel");
        }
        let cleanup_elapsed = start.elapsed();

        println!("Cleanup of 1000 tasks took: {:?}", cleanup_elapsed);
        assert!(cleanup_elapsed < Duration::from_secs(10), "Cleanup too slow: {:?}", cleanup_elapsed);

        // Verify clean state
        let remaining_tasks = manager.list_tasks().await.expect("Failed to list remaining tasks");
        assert_eq!(remaining_tasks.len(), 0);
    }

    #[tokio::test]
    async fn test_progress_update_efficiency() {
        let manager = TaskQueueManager::new();

        let task_id = manager.add_download(
            "https://example.com/progress-efficiency.zip".to_string(),
            PathBuf::from("/tmp/progress-efficiency.zip")
        ).await.expect("Failed to add download");

        let start = Instant::now();

        // Rapid progress updates
        for i in 0..1000 {
            let progress = DownloadProgress {
                downloaded_bytes: (i * 10) as u64,
                total_bytes: Some(10000),
                speed_bps: 1000,
                eta_seconds: Some((10000 - i * 10) / 1000),
            };
            manager.update_progress(task_id, progress).await.expect("Failed to update progress");
        }

        let elapsed = start.elapsed();
        println!("1000 progress updates took: {:?}", elapsed);

        // Progress updates should be efficient
        assert!(elapsed < Duration::from_secs(1), "Progress updates too slow: {:?}", elapsed);

        // Verify final progress
        let final_progress = manager.get_progress(task_id).await.expect("Failed to get final progress");
        assert_eq!(final_progress.downloaded_bytes, 9990);

        manager.cancel_download(task_id).await.expect("Failed to cancel");
    }
}

/// Test scalability under load
mod scalability_tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrent_manager_instances() {
        // Test multiple manager instances working concurrently
        let mut managers = Vec::new();
        for i in 0..5 {
            let manager = Arc::new(TaskQueueManager::new());
            managers.push((i, manager));
        }

        let start = Instant::now();

        let mut handles = Vec::new();
        for (manager_id, manager) in managers {
            let handle = tokio::spawn(async move {
                let mut task_ids = Vec::new();

                // Each manager handles its own workload
                for i in 0..20 {
                    let task_id = manager.add_download(
                        format!("https://manager{}.example.com/file{}.zip", manager_id, i),
                        PathBuf::from(format!("/tmp/manager{}/file{}.zip", manager_id, i))
                    ).await?;
                    task_ids.push(task_id);
                }

                // Perform operations
                for &task_id in &task_ids {
                    let _ = manager.get_task(task_id).await?;
                    let progress = DownloadProgress {
                        downloaded_bytes: 1000,
                        total_bytes: Some(5000),
                        speed_bps: 500,
                        eta_seconds: Some(8),
                    };
                    manager.update_progress(task_id, progress).await?;
                }

                // Clean up
                for task_id in task_ids {
                    manager.cancel_download(task_id).await?;
                }

                Ok::<usize, anyhow::Error>(manager_id)
            });
            handles.push(handle);
        }

        // Wait for all manager instances to complete
        for handle in handles {
            handle.await.expect("Manager task panicked").expect("Manager task failed");
        }

        let elapsed = start.elapsed();
        println!("5 concurrent managers with 20 tasks each completed in: {:?}", elapsed);

        // Should scale well with multiple instances
        assert!(elapsed < Duration::from_secs(15), "Multiple managers too slow: {:?}", elapsed);
    }

    #[tokio::test]
    async fn test_queue_processing_under_load() {
        let manager = Arc::new(TaskQueueManager::new());

        // Add many tasks quickly to stress the queue
        let start = Instant::now();
        let mut task_ids = Vec::new();

        for i in 0..100 {
            let task_id = manager.add_download(
                format!("https://example.com/queue-load-{}.zip", i),
                PathBuf::from(format!("/tmp/queue-load-{}.zip", i))
            ).await.expect("Failed to add download");
            task_ids.push(task_id);
        }

        let add_elapsed = start.elapsed();
        println!("Adding 100 tasks took: {:?}", add_elapsed);

        // Queue should handle rapid additions efficiently
        assert!(add_elapsed < Duration::from_secs(5), "Task addition too slow: {:?}", add_elapsed);

        // Verify queue state
        let active_count = manager.active_download_count().await;
        assert_eq!(active_count, 3); // Should respect concurrency limit

        let all_tasks = manager.list_tasks().await.expect("Failed to list tasks");
        assert_eq!(all_tasks.len(), 100);

        // Test queue progression by completing some tasks
        let completion_start = Instant::now();
        for i in 0..10 {
            manager.complete_task(task_ids[i]).await.expect("Failed to complete task");
        }

        // Give time for queue processing
        tokio::time::sleep(Duration::from_millis(100)).await;

        let completion_elapsed = completion_start.elapsed();
        println!("Completing 10 tasks and queue processing took: {:?}", completion_elapsed);

        // Should still have 3 active (queue progression)
        let final_active_count = manager.active_download_count().await;
        assert_eq!(final_active_count, 3);

        // Clean up remaining
        for &task_id in &task_ids[10..] {
            let _ = manager.cancel_download(task_id).await;
        }
    }
}

/// Compare performance between implementations
mod comparative_performance {
    use super::*;

    #[tokio::test]
    async fn test_basic_vs_queue_manager_performance() {
        let basic_manager = Arc::new(BasicDownloadManager::new());
        let queue_manager = Arc::new(TaskQueueManager::new());

        let managers: Vec<(&str, Arc<dyn DownloadManager>)> = vec![
            ("Basic", basic_manager),
            ("Queue", queue_manager),
        ];

        for (name, manager) in managers {
            println!("Testing {} manager performance", name);

            let start = Instant::now();

            // Standard operations test
            let mut task_ids = Vec::new();
            for i in 0..50 {
                let task_id = manager.add_download(
                    format!("https://example.com/{}-perf-{}.zip", name.to_lowercase(), i),
                    PathBuf::from(format!("/tmp/{}-perf-{}.zip", name.to_lowercase(), i))
                ).await.expect("Failed to add download");
                task_ids.push(task_id);
            }

            let add_elapsed = start.elapsed();

            // Operations test
            let ops_start = Instant::now();
            for &task_id in &task_ids {
                let _ = manager.get_task(task_id).await;
                let _ = manager.get_progress(task_id).await;
            }
            let ops_elapsed = ops_start.elapsed();

            // Cleanup test
            let cleanup_start = Instant::now();
            for task_id in task_ids {
                manager.cancel_download(task_id).await.expect("Failed to cancel");
            }
            let cleanup_elapsed = cleanup_start.elapsed();

            println!("  {} - Add: {:?}, Ops: {:?}, Cleanup: {:?}",
                name, add_elapsed, ops_elapsed, cleanup_elapsed);

            // Both should be reasonably performant
            assert!(add_elapsed < Duration::from_secs(5), "{} add operations too slow", name);
            assert!(ops_elapsed < Duration::from_secs(2), "{} read operations too slow", name);
            assert!(cleanup_elapsed < Duration::from_secs(5), "{} cleanup too slow", name);
        }
    }

    #[tokio::test]
    async fn test_responsiveness_under_load() {
        let manager = Arc::new(TaskQueueManager::new());

        // Create background load
        let background_manager = manager.clone();
        let background_handle = tokio::spawn(async move {
            for i in 0..1000 {
                let task_id = background_manager.add_download(
                    format!("https://example.com/background-{}.zip", i),
                    PathBuf::from(format!("/tmp/background-{}.zip", i))
                ).await.expect("Failed to add background download");

                if i % 10 == 0 {
                    let progress = DownloadProgress {
                        downloaded_bytes: (i * 10) as u64,
                        total_bytes: Some(10000),
                        speed_bps: 1000,
                        eta_seconds: Some(10),
                    };
                    let _ = background_manager.update_progress(task_id, progress).await;
                }

                // Small delay to spread the load
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        });

        // Test responsiveness during background load
        let start = Instant::now();

        let test_task_id = manager.add_download(
            "https://example.com/responsiveness-test.zip".to_string(),
            PathBuf::from("/tmp/responsiveness-test.zip")
        ).await.expect("Failed to add test download");

        let add_time = start.elapsed();

        let get_start = Instant::now();
        let _task = manager.get_task(test_task_id).await.expect("Failed to get task");
        let get_time = get_start.elapsed();

        let list_start = Instant::now();
        let _tasks = manager.list_tasks().await.expect("Failed to list tasks");
        let list_time = list_start.elapsed();

        // Should remain responsive even under load
        assert!(add_time < Duration::from_millis(100), "Add operation not responsive under load: {:?}", add_time);
        assert!(get_time < Duration::from_millis(50), "Get operation not responsive under load: {:?}", get_time);
        assert!(list_time < Duration::from_millis(200), "List operation not responsive under load: {:?}", list_time);

        println!("Responsiveness under load - Add: {:?}, Get: {:?}, List: {:?}",
            add_time, get_time, list_time);

        // Clean up
        manager.cancel_download(test_task_id).await.expect("Failed to cancel test task");

        // Wait for background to finish (with timeout)
        let _ = tokio::time::timeout(Duration::from_secs(10), background_handle).await;
    }
}