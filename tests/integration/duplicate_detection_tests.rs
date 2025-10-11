//! Integration tests for duplicate detection functionality
//!
//! Following TDD methodology: These tests are written FIRST and MUST FAIL
//! before implementation begins to ensure we're testing the actual functionality.

use burncloud_download::manager::persistent_aria2::PersistentAria2Manager;
use burncloud_download::services::duplicate_detector::DuplicateDetector;
use burncloud_download::models::{DuplicateResult, DuplicateReason};
use burncloud_download::types::{TaskId, DownloadStatus};
use std::path::Path;
use tempfile::TempDir;
use tokio_test;

#[tokio::test]
async fn test_integration_duplicate_detection_same_url_same_path() {
    // This test MUST FAIL initially (TDD requirement)
    let (_temp_dir, manager) = create_test_manager().await;

    let url = "https://example.com/file.zip";
    let target_path = Path::new("./test_downloads/file.zip");

    // First download request should create new task
    let first_task_id = manager
        .add_download(url, target_path, None, None)
        .await
        .expect("First download should succeed");

    // Second identical request should detect duplicate
    let second_task_id = manager
        .add_download(url, target_path, None, None)
        .await
        .expect("Second download should also succeed");

    // Should return the same task ID (duplicate detected)
    assert_eq!(
        first_task_id, second_task_id,
        "Duplicate downloads should return the same task ID"
    );

    // Verify only one task exists in the system
    let all_tasks = manager.get_all_tasks().await.expect("Should get all tasks");
    let matching_tasks: Vec<_> = all_tasks
        .iter()
        .filter(|task| task.url == url && task.target_path == target_path.to_path_buf())
        .collect();

    assert_eq!(
        matching_tasks.len(),
        1,
        "Should have exactly one task for duplicate URL/path combination"
    );
}

#[tokio::test]
async fn test_integration_duplicate_detection_normalized_urls() {
    // This test MUST FAIL initially (TDD requirement)
    let (_temp_dir, manager) = create_test_manager().await;

    let target_path = Path::new("./test_downloads/file.zip");

    // URL with query parameters in different order and fragment
    let original_url = "https://example.com/file.zip?b=2&a=1#section";
    let normalized_equivalent = "https://example.com/file.zip?a=1&b=2";

    // First download with original URL
    let first_task_id = manager
        .add_download(original_url, target_path, None, None)
        .await
        .expect("First download should succeed");

    // Second download with normalized equivalent should detect duplicate
    let second_task_id = manager
        .add_download(normalized_equivalent, target_path, None, None)
        .await
        .expect("Second download should also succeed");

    assert_eq!(
        first_task_id, second_task_id,
        "Normalized equivalent URLs should be detected as duplicates"
    );

    // Verify only one task exists
    let all_tasks = manager.get_all_tasks().await.expect("Should get all tasks");
    let normalized_tasks: Vec<_> = all_tasks
        .iter()
        .filter(|task| {
            let normalized_url = burncloud_download::utils::url_normalization::normalize_url(&task.url)
                .unwrap_or_else(|_| task.url.clone());
            let expected_normalized = burncloud_download::utils::url_normalization::normalize_url(original_url)
                .unwrap_or_else(|_| original_url.to_string());
            normalized_url == expected_normalized && task.target_path == target_path.to_path_buf()
        })
        .collect();

    assert_eq!(
        normalized_tasks.len(),
        1,
        "Should have exactly one task for normalized URL equivalents"
    );
}

#[tokio::test]
async fn test_integration_duplicate_detection_different_paths() {
    // This test MUST FAIL initially (TDD requirement)
    let (_temp_dir, manager) = create_test_manager().await;

    let url = "https://example.com/file.zip";
    let path1 = Path::new("./test_downloads/path1/file.zip");
    let path2 = Path::new("./test_downloads/path2/file.zip");

    // Same URL, different paths should create separate tasks
    let task_id1 = manager
        .add_download(url, path1, None, None)
        .await
        .expect("First download should succeed");

    let task_id2 = manager
        .add_download(url, path2, None, None)
        .await
        .expect("Second download should succeed");

    // Should create different tasks
    assert_ne!(
        task_id1, task_id2,
        "Same URL with different paths should create separate tasks"
    );

    // Verify both tasks exist
    let all_tasks = manager.get_all_tasks().await.expect("Should get all tasks");
    let url_tasks: Vec<_> = all_tasks
        .iter()
        .filter(|task| task.url == url)
        .collect();

    assert_eq!(
        url_tasks.len(),
        2,
        "Should have two tasks for same URL with different paths"
    );

    // Verify different paths
    let paths: std::collections::HashSet<_> = url_tasks
        .iter()
        .map(|task| &task.target_path)
        .collect();
    assert!(paths.contains(&path1.to_path_buf()));
    assert!(paths.contains(&path2.to_path_buf()));
}

#[tokio::test]
async fn test_integration_duplicate_detection_completed_task() {
    // This test MUST FAIL initially (TDD requirement)
    let (_temp_dir, manager) = create_test_manager().await;

    let url = "https://example.com/file.zip";
    let target_path = Path::new("./test_downloads/file.zip");

    // Create and complete a download task
    let first_task_id = manager
        .add_download(url, target_path, None, None)
        .await
        .expect("First download should succeed");

    // Simulate task completion
    manager
        .update_task_status(first_task_id, DownloadStatus::Completed)
        .await
        .expect("Should update task status");

    // Try to download the same file again
    let second_task_id = manager
        .add_download(url, target_path, None, None)
        .await
        .expect("Second download should succeed");

    // Behavior depends on duplicate policy implementation
    // This test verifies the system handles completed tasks appropriately
    let all_tasks = manager.get_all_tasks().await.expect("Should get all tasks");
    let matching_tasks: Vec<_> = all_tasks
        .iter()
        .filter(|task| task.url == url && task.target_path == target_path.to_path_buf())
        .collect();

    // Should have appropriate number of tasks based on policy
    assert!(
        matching_tasks.len() >= 1,
        "Should have at least one task for the URL/path combination"
    );

    // If duplicate policy allows re-downloading completed files, could be 2 tasks
    // If duplicate policy prevents re-downloading, should be 1 task
    assert!(
        matching_tasks.len() <= 2,
        "Should have at most two tasks for the URL/path combination"
    );
}

#[tokio::test]
async fn test_integration_duplicate_detection_failed_task() {
    // This test MUST FAIL initially (TDD requirement)
    let (_temp_dir, manager) = create_test_manager().await;

    let url = "https://example.com/file.zip";
    let target_path = Path::new("./test_downloads/file.zip");

    // Create a download task and mark it as failed
    let first_task_id = manager
        .add_download(url, target_path, None, None)
        .await
        .expect("First download should succeed");

    manager
        .update_task_status(first_task_id, DownloadStatus::Failed("Network error".to_string()))
        .await
        .expect("Should update task status");

    // Try to download the same file again
    let second_task_id = manager
        .add_download(url, target_path, None, None)
        .await
        .expect("Second download should succeed");

    // Should allow retry of failed downloads
    // Could return same task ID (retry) or new task ID (new attempt)
    let all_tasks = manager.get_all_tasks().await.expect("Should get all tasks");
    let matching_tasks: Vec<_> = all_tasks
        .iter()
        .filter(|task| task.url == url && task.target_path == target_path.to_path_buf())
        .collect();

    assert!(
        matching_tasks.len() >= 1,
        "Should have at least one task for failed download retry"
    );
}

#[tokio::test]
async fn test_integration_duplicate_detection_performance() {
    // This test MUST FAIL initially (TDD requirement)
    let (_temp_dir, manager) = create_test_manager().await;

    let url = "https://example.com/performance-test.zip";
    let target_path = Path::new("./test_downloads/performance-test.zip");

    // Performance test: multiple duplicate detections should be fast
    let start_time = std::time::Instant::now();

    // First request creates the task
    let _first_task_id = manager
        .add_download(url, target_path, None, None)
        .await
        .expect("First download should succeed");

    // Multiple duplicate detection requests
    for i in 0..10 {
        let _task_id = manager
            .add_download(url, target_path, None, None)
            .await
            .expect(&format!("Duplicate request {} should succeed", i));
    }

    let duration = start_time.elapsed();

    // Should complete all operations within reasonable time (<100ms as per success criteria)
    assert!(
        duration.as_millis() < 100,
        "Duplicate detection performance too slow: {:?} (should be <100ms)",
        duration
    );

    // Verify still only one task exists
    let all_tasks = manager.get_all_tasks().await.expect("Should get all tasks");
    let matching_tasks: Vec<_> = all_tasks
        .iter()
        .filter(|task| task.url == url && task.target_path == target_path.to_path_buf())
        .collect();

    assert_eq!(
        matching_tasks.len(),
        1,
        "Should still have exactly one task after performance test"
    );
}

// Helper functions for integration testing

async fn create_test_manager() -> (TempDir, PersistentAria2Manager) {
    let temp_dir = TempDir::new().expect("Should create temp directory");

    // Create test manager with temporary database
    let db_path = temp_dir.path().join("test.db");
    let manager = PersistentAria2Manager::new(&db_path.to_string_lossy())
        .await
        .expect("Should create test manager");

    (temp_dir, manager)
}

// Mock implementations for testing
impl PersistentAria2Manager {
    pub async fn get_all_tasks(&self) -> anyhow::Result<Vec<MockDownloadTask>> {
        // This method needs to be implemented to support testing
        todo!("Need to implement get_all_tasks for testing")
    }

    pub async fn update_task_status(
        &self,
        task_id: TaskId,
        status: DownloadStatus,
    ) -> anyhow::Result<()> {
        // This method needs to be implemented to support testing
        todo!("Need to implement update_task_status for testing")
    }
}

#[derive(Debug, Clone)]
pub struct MockDownloadTask {
    pub id: TaskId,
    pub url: String,
    pub target_path: std::path::PathBuf,
    pub status: DownloadStatus,
}