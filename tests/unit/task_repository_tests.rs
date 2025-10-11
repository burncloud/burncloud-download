//! Unit tests for TaskRepository operations
//!
//! These tests verify the TaskRepository trait methods.

use burncloud_download::services::task_repository::{TaskRepository, DefaultTaskRepository};
use burncloud_download::types::{TaskId};
use std::path::Path;

#[tokio::test]
async fn test_find_by_url_hash_and_path() {
    let repository = create_test_repository().await;

    let url_hash = "test_hash";
    let target_path = Path::new("./downloads/file.zip");

    // Test that no tasks are found initially
    let result = repository.find_by_url_hash_and_path(url_hash, target_path).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[tokio::test]
async fn test_find_by_file_hash() {
    let repository = create_test_repository().await;

    let file_hash = "test_file_hash";

    // Test that no tasks are found initially
    let result = repository.find_by_file_hash(file_hash).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[tokio::test]
async fn test_update_duplicate_fields() {
    let repository = create_test_repository().await;

    let task_id = TaskId::new();
    let url_hash = "test_url_hash";
    let file_hash = Some("test_file_hash");
    let file_size = Some(1024u64);

    // Test that update succeeds (placeholder implementation)
    let result = repository.update_duplicate_fields(&task_id, url_hash, file_hash.as_deref(), file_size).await;
    assert!(result.is_ok());
}

// Helper functions for testing

async fn create_test_repository() -> impl TaskRepository {
    DefaultTaskRepository::new()
}