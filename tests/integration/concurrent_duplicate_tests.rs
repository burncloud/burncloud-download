//! Integration tests for concurrent duplicate detection
//!
//! Following TDD methodology: These tests are written FIRST and MUST FAIL
//! before implementation begins to ensure we're testing the actual functionality.

use burncloud_download::manager::persistent_aria2::PersistentAria2Manager;
use burncloud_download::types::TaskId;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Barrier;
use futures::future::try_join_all;

#[tokio::test]
async fn test_concurrent_duplicate_requests_same_url_same_path() {
    // This test MUST FAIL initially (TDD requirement)
    let (_temp_dir, manager) = create_test_manager().await;
    let manager = Arc::new(manager);

    let url = "https://example.com/concurrent-test.zip";
    let target_path = Path::new("./test_downloads/concurrent-test.zip");

    // Create barrier to synchronize concurrent requests
    let barrier = Arc::new(Barrier::new(5));
    let mut tasks = Vec::new();

    // Launch 5 concurrent requests for the same URL/path
    for i in 0..5 {
        let manager_clone = Arc::clone(&manager);
        let barrier_clone = Arc::clone(&barrier);
        let url_clone = url.to_string();
        let path_clone = target_path.to_path_buf();

        let task = tokio::spawn(async move {
            // Wait for all tasks to start simultaneously
            barrier_clone.wait().await;

            // Attempt to add download
            manager_clone
                .add_download(&url_clone, &path_clone, None, None)
                .await
                .map_err(|e| format!("Task {} failed: {}", i, e))
        });

        tasks.push(task);
    }

    // Wait for all tasks to complete
    let results: Result<Vec<_>, _> = try_join_all(tasks).await;
    let task_ids: Vec<TaskId> = results
        .expect("All concurrent tasks should complete")
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("All downloads should succeed");

    // All concurrent requests should return the same task ID
    assert_eq!(task_ids.len(), 5, "Should have 5 task IDs returned");

    let first_task_id = task_ids[0];
    for (i, &task_id) in task_ids.iter().enumerate() {
        assert_eq!(
            task_id, first_task_id,
            "Task {} should return same ID as first task ({})",
            i, first_task_id
        );
    }

    // Verify only one task was actually created in the database
    let all_tasks = manager.get_all_tasks().await.expect("Should get all tasks");
    let matching_tasks: Vec<_> = all_tasks
        .iter()
        .filter(|task| task.url == url && task.target_path == target_path.to_path_buf())
        .collect();

    assert_eq!(
        matching_tasks.len(),
        1,
        "Should have exactly one task in database despite concurrent requests"
    );
}

#[tokio::test]
async fn test_concurrent_duplicate_requests_different_paths() {
    // This test MUST FAIL initially (TDD requirement)
    let (_temp_dir, manager) = create_test_manager().await;
    let manager = Arc::new(manager);

    let url = "https://example.com/concurrent-different-paths.zip";
    let paths = vec![
        Path::new("./test_downloads/path1/file.zip"),
        Path::new("./test_downloads/path2/file.zip"),
        Path::new("./test_downloads/path3/file.zip"),
    ];

    // Create barrier for synchronization
    let barrier = Arc::new(Barrier::new(paths.len()));
    let mut tasks = Vec::new();

    // Launch concurrent requests with different paths
    for (i, path) in paths.iter().enumerate() {
        let manager_clone = Arc::clone(&manager);
        let barrier_clone = Arc::clone(&barrier);
        let url_clone = url.to_string();
        let path_clone = path.to_path_buf();

        let task = tokio::spawn(async move {
            barrier_clone.wait().await;

            manager_clone
                .add_download(&url_clone, &path_clone, None, None)
                .await
                .map_err(|e| format!("Task {} failed: {}", i, e))
        });

        tasks.push(task);
    }

    // Wait for all tasks to complete
    let results: Result<Vec<_>, _> = try_join_all(tasks).await;
    let task_ids: Vec<TaskId> = results
        .expect("All concurrent tasks should complete")
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("All downloads should succeed");

    // Should create separate task IDs for different paths
    assert_eq!(task_ids.len(), paths.len(), "Should have one task ID per path");

    let unique_task_ids: std::collections::HashSet<_> = task_ids.iter().collect();
    assert_eq!(
        unique_task_ids.len(),
        paths.len(),
        "All task IDs should be unique for different paths"
    );

    // Verify all tasks were created in the database
    let all_tasks = manager.get_all_tasks().await.expect("Should get all tasks");
    let matching_tasks: Vec<_> = all_tasks
        .iter()
        .filter(|task| task.url == url)
        .collect();

    assert_eq!(
        matching_tasks.len(),
        paths.len(),
        "Should have one task per unique path in database"
    );

    // Verify all paths are represented
    let task_paths: std::collections::HashSet<_> = matching_tasks
        .iter()
        .map(|task| &task.target_path)
        .collect();

    for path in &paths {
        assert!(
            task_paths.contains(&path.to_path_buf()),
            "Database should contain task for path: {:?}",
            path
        );
    }
}

#[tokio::test]
async fn test_concurrent_duplicate_requests_mixed_scenarios() {
    // This test MUST FAIL initially (TDD requirement)
    let (_temp_dir, manager) = create_test_manager().await;
    let manager = Arc::new(manager);

    let url1 = "https://example.com/file1.zip";
    let url2 = "https://example.com/file2.zip";
    let path1 = Path::new("./test_downloads/file1.zip");
    let path2 = Path::new("./test_downloads/file2.zip");

    // Create scenarios:
    // - 3 requests for url1 + path1 (should create 1 task)
    // - 2 requests for url2 + path2 (should create 1 task)
    // - 1 request for url1 + path2 (should create 1 task)
    // Total: 3 unique tasks from 6 requests

    let scenarios = vec![
        (url1, path1, 0), // Group 1
        (url1, path1, 1),
        (url1, path1, 2),
        (url2, path2, 3), // Group 2
        (url2, path2, 4),
        (url1, path2, 5), // Group 3
    ];

    let barrier = Arc::new(Barrier::new(scenarios.len()));
    let mut tasks = Vec::new();

    for (url, path, task_index) in scenarios {
        let manager_clone = Arc::clone(&manager);
        let barrier_clone = Arc::clone(&barrier);
        let url_clone = url.to_string();
        let path_clone = path.to_path_buf();

        let task = tokio::spawn(async move {
            barrier_clone.wait().await;

            manager_clone
                .add_download(&url_clone, &path_clone, None, None)
                .await
                .map_err(|e| format!("Task {} failed: {}", task_index, e))
        });

        tasks.push(task);
    }

    // Wait for all tasks to complete
    let results: Result<Vec<_>, _> = try_join_all(tasks).await;
    let task_ids: Vec<TaskId> = results
        .expect("All concurrent tasks should complete")
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("All downloads should succeed");

    assert_eq!(task_ids.len(), 6, "Should have 6 task IDs returned");

    // Group 1: url1 + path1 (indices 0, 1, 2) should have same task ID
    assert_eq!(task_ids[0], task_ids[1], "Group 1 tasks should have same ID");
    assert_eq!(task_ids[1], task_ids[2], "Group 1 tasks should have same ID");

    // Group 2: url2 + path2 (indices 3, 4) should have same task ID
    assert_eq!(task_ids[3], task_ids[4], "Group 2 tasks should have same ID");

    // All groups should have different task IDs
    assert_ne!(task_ids[0], task_ids[3], "Group 1 and 2 should differ");
    assert_ne!(task_ids[0], task_ids[5], "Group 1 and 3 should differ");
    assert_ne!(task_ids[3], task_ids[5], "Group 2 and 3 should differ");

    // Verify database contains exactly 3 unique tasks
    let all_tasks = manager.get_all_tasks().await.expect("Should get all tasks");
    let unique_combinations: std::collections::HashSet<_> = all_tasks
        .iter()
        .map(|task| (&task.url, &task.target_path))
        .collect();

    assert_eq!(
        unique_combinations.len(),
        3,
        "Should have exactly 3 unique URL/path combinations in database"
    );
}

#[tokio::test]
async fn test_concurrent_duplicate_detection_race_condition_resilience() {
    // This test MUST FAIL initially (TDD requirement)
    let (_temp_dir, manager) = create_test_manager().await;
    let manager = Arc::new(manager);

    let url = "https://example.com/race-condition-test.zip";
    let target_path = Path::new("./test_downloads/race-condition-test.zip");

    // High-concurrency test to stress-test race condition handling
    let num_concurrent_requests = 20;
    let barrier = Arc::new(Barrier::new(num_concurrent_requests));
    let mut tasks = Vec::new();

    for i in 0..num_concurrent_requests {
        let manager_clone = Arc::clone(&manager);
        let barrier_clone = Arc::clone(&barrier);
        let url_clone = url.to_string();
        let path_clone = target_path.to_path_buf();

        let task = tokio::spawn(async move {
            barrier_clone.wait().await;

            // Add small random delay to increase race condition probability
            let delay_ms = (i % 5) as u64;
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

            manager_clone
                .add_download(&url_clone, &path_clone, None, None)
                .await
                .map_err(|e| format!("Task {} failed: {}", i, e))
        });

        tasks.push(task);
    }

    // Wait for all tasks to complete
    let results: Result<Vec<_>, _> = try_join_all(tasks).await;
    let task_ids: Vec<TaskId> = results
        .expect("All concurrent tasks should complete without panics or deadlocks")
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("All downloads should succeed despite race conditions");

    assert_eq!(
        task_ids.len(),
        num_concurrent_requests,
        "Should have {} task IDs returned",
        num_concurrent_requests
    );

    // All task IDs should be identical (no duplicates created)
    let first_task_id = task_ids[0];
    for (i, &task_id) in task_ids.iter().enumerate() {
        assert_eq!(
            task_id, first_task_id,
            "Task {} should return same ID as first task despite race conditions",
            i
        );
    }

    // Verify database consistency - exactly one task should exist
    let all_tasks = manager.get_all_tasks().await.expect("Should get all tasks");
    let matching_tasks: Vec<_> = all_tasks
        .iter()
        .filter(|task| task.url == url && task.target_path == target_path.to_path_buf())
        .collect();

    assert_eq!(
        matching_tasks.len(),
        1,
        "Database should contain exactly one task despite {} concurrent requests",
        num_concurrent_requests
    );
}

#[tokio::test]
async fn test_concurrent_duplicate_detection_performance_under_load() {
    // This test MUST FAIL initially (TDD requirement)
    let (_temp_dir, manager) = create_test_manager().await;
    let manager = Arc::new(manager);

    let url = "https://example.com/performance-under-load.zip";
    let target_path = Path::new("./test_downloads/performance-under-load.zip");

    let num_requests = 50;
    let start_time = std::time::Instant::now();

    // Launch many concurrent requests
    let barrier = Arc::new(Barrier::new(num_requests));
    let mut tasks = Vec::new();

    for i in 0..num_requests {
        let manager_clone = Arc::clone(&manager);
        let barrier_clone = Arc::clone(&barrier);
        let url_clone = url.to_string();
        let path_clone = target_path.to_path_buf();

        let task = tokio::spawn(async move {
            barrier_clone.wait().await;
            manager_clone.add_download(&url_clone, &path_clone, None, None).await
        });

        tasks.push(task);
    }

    // Wait for all to complete
    let results: Result<Vec<_>, _> = try_join_all(tasks).await;
    let _task_ids: Vec<TaskId> = results
        .expect("All tasks should complete")
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("All downloads should succeed");

    let total_duration = start_time.elapsed();

    // Performance requirement: should handle concurrent load efficiently
    assert!(
        total_duration.as_millis() < 500,
        "Concurrent duplicate detection under load too slow: {:?} (should be <500ms for {} requests)",
        total_duration,
        num_requests
    );

    // Verify correctness: still only one task created
    let all_tasks = manager.get_all_tasks().await.expect("Should get all tasks");
    let matching_tasks: Vec<_> = all_tasks
        .iter()
        .filter(|task| task.url == url && task.target_path == target_path.to_path_buf())
        .collect();

    assert_eq!(
        matching_tasks.len(),
        1,
        "Should have exactly one task despite {} concurrent requests",
        num_requests
    );
}

// Helper functions for concurrent testing

async fn create_test_manager() -> (TempDir, PersistentAria2Manager) {
    let temp_dir = TempDir::new().expect("Should create temp directory");

    // Create test manager with temporary database
    let db_path = temp_dir.path().join("concurrent_test.db");
    let manager = PersistentAria2Manager::new(&db_path.to_string_lossy())
        .await
        .expect("Should create test manager");

    (temp_dir, manager)
}