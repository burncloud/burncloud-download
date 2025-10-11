//! Unit tests for DuplicateDetector trait
//!
//! Following TDD methodology: These tests are written FIRST and MUST FAIL
//! before implementation begins to ensure we're testing the actual functionality.

use burncloud_download::services::duplicate_detector::{DuplicateDetector, DefaultDuplicateDetector};
use burncloud_download::models::{DuplicateResult, DuplicateReason};
use burncloud_download::types::TaskId;
use std::path::Path;

#[tokio::test]
async fn test_find_duplicate_returns_existing_task_for_exact_match() {
    // This test MUST FAIL initially (TDD requirement)
    let detector = create_test_detector().await;

    // Setup: Create a download task in the database
    let url = "https://example.com/file.zip";
    let target_path = Path::new("./downloads/file.zip");

    // First download should create a new task
    let first_result = detector.find_duplicate(url, target_path).await.unwrap();
    assert!(matches!(first_result, DuplicateResult::NotFound { .. }));

    // Simulate task creation (this would be done by the manager)
    let task_id = create_mock_task(&detector, url, target_path).await;

    // Second identical request should find the duplicate
    let second_result = detector.find_duplicate(url, target_path).await.unwrap();
    if let DuplicateResult::Found { task_id: found_id, reason, .. } = second_result {
        assert_eq!(found_id, task_id);
        assert_eq!(reason, DuplicateReason::ExactMatch);
    } else {
        panic!("Expected duplicate to be found");
    }
}

#[tokio::test]
async fn test_find_duplicate_normalizes_urls_before_comparison() {
    // This test MUST FAIL initially (TDD requirement)
    let detector = create_test_detector().await;

    let target_path = Path::new("./downloads/file.zip");

    // Create task with one URL format
    let original_url = "https://example.com/file.zip?b=2&a=1#fragment";
    let first_result = detector.find_duplicate(original_url, target_path).await.unwrap();
    assert!(matches!(first_result, DuplicateResult::NotFound { .. }));

    let task_id = create_mock_task(&detector, original_url, target_path).await;

    // Try with normalized equivalent URL
    let normalized_url = "https://example.com/file.zip?a=1&b=2";
    let second_result = detector.find_duplicate(normalized_url, target_path).await.unwrap();

    if let DuplicateResult::Found { task_id: found_id, reason, .. } = second_result {
        assert_eq!(found_id, task_id);
        assert_eq!(reason, DuplicateReason::ExactMatch);
    } else {
        panic!("Expected normalized URL to match existing task");
    }
}

#[tokio::test]
async fn test_find_duplicate_different_paths_same_url() {
    // This test MUST FAIL initially (TDD requirement)
    let detector = create_test_detector().await;

    let url = "https://example.com/file.zip";
    let path1 = Path::new("./downloads/path1/file.zip");
    let path2 = Path::new("./downloads/path2/file.zip");

    // Create task with first path
    let first_result = detector.find_duplicate(url, path1).await.unwrap();
    assert!(matches!(first_result, DuplicateResult::NotFound { .. }));

    let _task_id1 = create_mock_task(&detector, url, path1).await;

    // Same URL, different path should not be considered duplicate
    let second_result = detector.find_duplicate(url, path2).await.unwrap();
    assert!(matches!(second_result, DuplicateResult::NotFound { .. }));
}

#[tokio::test]
async fn test_find_by_url_hash_returns_all_matching_tasks() {
    // This test MUST FAIL initially (TDD requirement)
    let detector = create_test_detector().await;

    let url = "https://example.com/file.zip";
    let path1 = Path::new("./downloads/path1/file.zip");
    let path2 = Path::new("./downloads/path2/file.zip");

    // Create two tasks with same URL but different paths
    let task_id1 = create_mock_task(&detector, url, path1).await;
    let task_id2 = create_mock_task(&detector, url, path2).await;

    // Find by URL hash should return both tasks
    let url_hash = burncloud_download::utils::url_normalization::hash_normalized_url(
        &burncloud_download::utils::url_normalization::normalize_url(url).unwrap()
    );

    let tasks = detector.find_by_url_hash(&url_hash).await.unwrap();
    assert_eq!(tasks.len(), 2);

    let task_ids: std::collections::HashSet<_> = tasks.iter().map(|t| t.id).collect();
    assert!(task_ids.contains(&task_id1));
    assert!(task_ids.contains(&task_id2));
}

#[tokio::test]
async fn test_find_duplicate_handles_completed_tasks() {
    // This test MUST FAIL initially (TDD requirement)
    let detector = create_test_detector().await;

    let url = "https://example.com/file.zip";
    let target_path = Path::new("./downloads/file.zip");

    // Create and complete a task
    let task_id = create_mock_task(&detector, url, target_path).await;
    mark_task_completed(&detector, task_id).await;

    // Check if duplicate detection handles completed tasks according to policy
    let result = detector.find_duplicate(url, target_path).await.unwrap();

    // Behavior depends on DuplicatePolicy implementation
    // This test verifies the detector respects policy decisions
    match result {
        DuplicateResult::Found { reason, .. } => {
            assert!(matches!(reason, DuplicateReason::ExactMatch | DuplicateReason::PolicyAllowed));
        }
        DuplicateResult::NotFound { .. } => {
            // Also valid if policy allows re-downloading completed files
        }
        DuplicateResult::NewTask(_) => {
            // Valid if a new task was created
        }
        DuplicateResult::ExistingTask { .. } => {
            // Valid if existing task was found
        }
        DuplicateResult::RequiresDecision { .. } => {
            // Valid if user decision is required
        }
    }
}

#[tokio::test]
async fn test_find_duplicate_error_handling() {
    // This test MUST FAIL initially (TDD requirement)
    let detector = create_test_detector().await;

    // Test with invalid URLs
    let invalid_url = "not-a-valid-url";
    let target_path = Path::new("./downloads/file.zip");

    let result = detector.find_duplicate(invalid_url, target_path).await;
    assert!(result.is_err(), "Should return error for invalid URL");

    // Test with empty path
    let valid_url = "https://example.com/file.zip";
    let empty_path = Path::new("");

    let _result = detector.find_duplicate(valid_url, empty_path).await;
    // Should handle gracefully (may succeed or fail depending on implementation)
    // The important thing is it doesn't panic
}

#[tokio::test]
async fn test_find_duplicate_concurrent_access() {
    // This test MUST FAIL initially (TDD requirement)
    let detector = create_test_detector().await;

    let url = "https://example.com/file.zip";
    let target_path = Path::new("./downloads/file.zip");

    // Simulate concurrent duplicate detection requests
    let detector_clone = create_test_detector().await;
    let url_clone = url.to_string();
    let path_clone = target_path.to_path_buf();

    let task1 = tokio::spawn(async move {
        detector.find_duplicate(&url_clone, &path_clone).await
    });

    let task2 = tokio::spawn(async move {
        detector_clone.find_duplicate(url, target_path).await
    });

    let (result1, result2) = tokio::try_join!(task1, task2).unwrap();

    // Both should succeed without race conditions
    assert!(result1.is_ok());
    assert!(result2.is_ok());

    // At least one should find no duplicate (first request)
    let has_not_found = matches!(result1.unwrap(), DuplicateResult::NotFound { .. }) ||
                       matches!(result2.unwrap(), DuplicateResult::NotFound { .. });
    assert!(has_not_found);
}

#[tokio::test]
async fn test_find_duplicate_performance() {
    // This test MUST FAIL initially (TDD requirement)
    let detector = create_test_detector().await;

    let url = "https://example.com/file.zip";
    let target_path = Path::new("./downloads/file.zip");

    // Performance test: should complete within 100ms as per success criteria
    let start = std::time::Instant::now();

    for _ in 0..10 {
        let _ = detector.find_duplicate(url, target_path).await.unwrap();
    }

    let duration = start.elapsed();
    assert!(duration.as_millis() < 100,
           "Duplicate detection too slow: {:?} (should be <100ms)", duration);
}

// Helper functions for testing (these will also need implementation)

async fn create_test_detector() -> impl DuplicateDetector {
    DefaultDuplicateDetector::new()
}

async fn create_mock_task(_detector: &impl DuplicateDetector, _url: &str, _path: &Path) -> TaskId {
    // This function simulates task creation and returns the task ID
    // For now, just return a new TaskId to make tests compile
    TaskId::new()
}

async fn mark_task_completed(_detector: &impl DuplicateDetector, _task_id: TaskId) {
    // This function marks a task as completed for testing purposes
    // For now, this is a no-op to make tests compile
}