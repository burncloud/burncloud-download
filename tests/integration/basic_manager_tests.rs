//! Integration tests for BasicDownloadManager
//!
//! These tests verify the public API and behavior of the BasicDownloadManager.

use burncloud_download::{BasicDownloadManager, DownloadManager};
use std::path::PathBuf;

#[tokio::test]
async fn test_basic_download_manager_add_download() {
    let manager = BasicDownloadManager::new();

    let task_id = manager.add_download(
        "https://example.com/file.zip".to_string(),
        PathBuf::from("data/file.zip")
    ).await.unwrap();

    // Task should be created with a valid ID
    assert!(!task_id.to_string().is_empty());

    // Task should exist in the manager
    let task = manager.get_task(task_id).await.unwrap();
    assert_eq!(task.url, "https://example.com/file.zip");
    assert_eq!(task.target_path, PathBuf::from("data/file.zip"));
}

#[tokio::test]
async fn test_basic_download_manager_progress_tracking() {
    let manager = BasicDownloadManager::new();

    let task_id = manager.add_download(
        "https://example.com/file.zip".to_string(),
        PathBuf::from("data/file.zip")
    ).await.unwrap();

    // Progress should be available
    let progress = manager.get_progress(task_id).await.unwrap();
    assert_eq!(progress.downloaded_bytes, 0);
    assert!(progress.total_bytes.is_none() || progress.total_bytes == Some(0));
}

#[tokio::test]
async fn test_basic_download_manager_pause_resume() {
    let manager = BasicDownloadManager::new();

    let task_id = manager.add_download(
        "https://example.com/file.zip".to_string(),
        PathBuf::from("data/file.zip")
    ).await.unwrap();

    // Test pause
    let result = manager.pause_download(task_id).await;
    assert!(result.is_ok());

    // Test resume
    let result = manager.resume_download(task_id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_basic_download_manager_cancel() {
    let manager = BasicDownloadManager::new();

    let task_id = manager.add_download(
        "https://example.com/file.zip".to_string(),
        PathBuf::from("data/file.zip")
    ).await.unwrap();

    // Test cancel
    let result = manager.cancel_download(task_id).await;
    assert!(result.is_ok());

    // Task should no longer exist
    let result = manager.get_task(task_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_basic_download_manager_list_tasks() {
    let manager = BasicDownloadManager::new();

    let task_id1 = manager.add_download(
        "https://example.com/file1.zip".to_string(),
        PathBuf::from("data/file1.zip")
    ).await.unwrap();

    let task_id2 = manager.add_download(
        "https://example.com/file2.zip".to_string(),
        PathBuf::from("data/file2.zip")
    ).await.unwrap();

    let tasks = manager.list_tasks().await.unwrap();
    assert_eq!(tasks.len(), 2);

    let task_ids: Vec<_> = tasks.iter().map(|t| t.id).collect();
    assert!(task_ids.contains(&task_id1));
    assert!(task_ids.contains(&task_id2));
}

#[tokio::test]
async fn test_basic_download_manager_active_count() {
    let manager = BasicDownloadManager::new();

    assert_eq!(manager.active_download_count().await.unwrap(), 0);

    let _task_id = manager.add_download(
        "https://example.com/file.zip".to_string(),
        PathBuf::from("data/file.zip")
    ).await.unwrap();

    assert_eq!(manager.active_download_count().await.unwrap(), 1);
}