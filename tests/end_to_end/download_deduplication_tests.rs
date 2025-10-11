//! End-to-end tests for download deduplication workflow
//!
//! Following TDD methodology: These tests are written FIRST and MUST FAIL
//! before implementation begins to ensure we're testing the actual functionality.

use burncloud_download::manager::persistent_aria2::PersistentAria2Manager;
use burncloud_download::types::{TaskId, DownloadStatus, DownloadProgress};
use std::path::Path;
use tempfile::TempDir;

#[tokio::test]
async fn test_e2e_complete_deduplication_workflow() {
    // This test MUST FAIL initially (TDD requirement)
    let (_temp_dir, manager) = create_test_manager().await;

    let url = "https://httpbin.org/bytes/1024"; // Real endpoint for testing
    let target_path = Path::new("./test_downloads/test_file.bin");

    // Step 1: Start first download
    let task_id1 = manager
        .add_download(url, target_path, None, None)
        .await
        .expect("First download should start successfully");

    // Verify initial task state
    let task = manager
        .get_task_info(task_id1)
        .await
        .expect("Should get task info")
        .expect("Task should exist");

    assert_eq!(task.id, task_id1);
    assert_eq!(task.url, url);
    assert_eq!(task.target_path, target_path.to_path_buf());
    assert_eq!(task.status, DownloadStatus::Waiting);

    // Step 2: Attempt duplicate download while first is in progress
    let task_id2 = manager
        .add_download(url, target_path, None, None)
        .await
        .expect("Duplicate download request should be handled gracefully");

    // Should return the same task ID (duplicate detected)
    assert_eq!(
        task_id1, task_id2,
        "Duplicate download should return same task ID"
    );

    // Step 3: Start the download process
    manager
        .start_download(task_id1)
        .await
        .expect("Should start download");

    // Verify status changed to downloading
    let task = manager
        .get_task_info(task_id1)
        .await
        .expect("Should get task info")
        .expect("Task should exist");
    assert_eq!(task.status, DownloadStatus::Downloading);

    // Step 4: Attempt another duplicate while downloading
    let task_id3 = manager
        .add_download(url, target_path, None, None)
        .await
        .expect("Duplicate during download should be handled");

    assert_eq!(
        task_id1, task_id3,
        "Duplicate during download should return same task ID"
    );

    // Step 5: Monitor download progress
    let mut progress_checks = 0;
    let max_checks = 30; // Timeout after 30 checks

    loop {
        progress_checks += 1;
        if progress_checks > max_checks {
            panic!("Download took too long or stuck");
        }

        let progress = manager
            .get_download_progress(task_id1)
            .await
            .expect("Should get progress");

        match progress {
            Some(DownloadProgress::Completed { .. }) => {
                println!("Download completed successfully");
                break;
            }
            Some(DownloadProgress::InProgress { downloaded_bytes, total_bytes, .. }) => {
                println!(
                    "Download progress: {}/{} bytes",
                    downloaded_bytes,
                    total_bytes.unwrap_or(0)
                );
            }
            Some(DownloadProgress::Failed { error, .. }) => {
                panic!("Download failed: {}", error);
            }
            None => {
                println!("No progress information available yet");
            }
        }

        // Wait a bit before next check
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    // Step 6: Verify final task state
    let final_task = manager
        .get_task_info(task_id1)
        .await
        .expect("Should get final task info")
        .expect("Task should exist");

    assert_eq!(final_task.status, DownloadStatus::Completed);

    // Step 7: Test duplicate detection after completion
    let task_id4 = manager
        .add_download(url, target_path, None, None)
        .await
        .expect("Post-completion duplicate should be handled");

    // Behavior depends on duplicate policy:
    // - If re-downloading completed files is allowed: new task ID
    // - If re-downloading is prevented: same task ID
    let all_tasks = manager.get_all_tasks().await.expect("Should get all tasks");
    let matching_tasks: Vec<_> = all_tasks
        .iter()
        .filter(|task| task.url == url && task.target_path == target_path.to_path_buf())
        .collect();

    assert!(
        matching_tasks.len() >= 1 && matching_tasks.len() <= 2,
        "Should have 1-2 tasks depending on duplicate policy"
    );

    // Step 8: Verify file was actually downloaded
    assert!(
        target_path.exists(),
        "Downloaded file should exist at target path"
    );

    let file_size = std::fs::metadata(target_path)
        .expect("Should get file metadata")
        .len();

    assert!(
        file_size > 0,
        "Downloaded file should have content (size: {})",
        file_size
    );
}

#[tokio::test]
async fn test_e2e_deduplication_with_multiple_files() {
    // This test MUST FAIL initially (TDD requirement)
    let (_temp_dir, manager) = create_test_manager().await;

    // Test multiple different downloads and their deduplication
    let downloads = vec![
        ("https://httpbin.org/bytes/512", "./test_downloads/file1.bin"),
        ("https://httpbin.org/bytes/1024", "./test_downloads/file2.bin"),
        ("https://httpbin.org/bytes/512", "./test_downloads/file1_duplicate.bin"), // Same URL, different path
        ("https://httpbin.org/bytes/1024", "./test_downloads/file2.bin"), // Exact duplicate
    ];

    let mut task_ids = Vec::new();

    // Start all downloads
    for (url, path_str) in &downloads {
        let path = Path::new(path_str);
        let task_id = manager
            .add_download(url, path, None, None)
            .await
            .expect(&format!("Should start download for {}", url));

        task_ids.push(task_id);
        println!("Started download: {} -> {} (Task: {})", url, path_str, task_id);
    }

    // Verify deduplication logic
    assert_ne!(task_ids[0], task_ids[1], "Different URLs should have different task IDs");
    assert_ne!(task_ids[0], task_ids[2], "Same URL, different path should have different task IDs");
    assert_eq!(task_ids[1], task_ids[3], "Exact duplicates should have same task ID");

    // Start downloads for unique tasks
    let unique_task_ids: std::collections::HashSet<_> = task_ids.iter().copied().collect();

    for &task_id in &unique_task_ids {
        manager
            .start_download(task_id)
            .await
            .expect(&format!("Should start download for task {}", task_id));
    }

    // Monitor all downloads to completion
    let mut completed_tasks = std::collections::HashSet::new();
    let mut attempts = 0;
    let max_attempts = 60; // 30 seconds timeout

    while completed_tasks.len() < unique_task_ids.len() && attempts < max_attempts {
        attempts += 1;

        for &task_id in &unique_task_ids {
            if completed_tasks.contains(&task_id) {
                continue;
            }

            if let Some(progress) = manager.get_download_progress(task_id).await.expect("Should get progress") {
                match progress {
                    DownloadProgress::Completed { .. } => {
                        completed_tasks.insert(task_id);
                        println!("Task {} completed", task_id);
                    }
                    DownloadProgress::Failed { error, .. } => {
                        panic!("Task {} failed: {}", task_id, error);
                    }
                    DownloadProgress::InProgress { downloaded_bytes, total_bytes, .. } => {
                        println!(
                            "Task {} progress: {}/{} bytes",
                            task_id,
                            downloaded_bytes,
                            total_bytes.unwrap_or(0)
                        );
                    }
                }
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    assert_eq!(
        completed_tasks.len(),
        unique_task_ids.len(),
        "All unique downloads should complete"
    );

    // Verify all expected files exist
    for (i, (_, path_str)) in downloads.iter().enumerate() {
        let path = Path::new(path_str);

        if i == 3 {
            // This is the exact duplicate (same URL, same path as downloads[1])
            // File should exist from the first download of this URL/path combination
            assert_eq!(path_str, &downloads[1].1, "Sanity check: duplicate should have same path");
        }

        assert!(
            path.exists(),
            "File should exist at path: {} (from download {})",
            path_str,
            i
        );

        let file_size = std::fs::metadata(path)
            .expect(&format!("Should get metadata for {}", path_str))
            .len();

        assert!(
            file_size > 0,
            "File should have content: {} (size: {})",
            path_str,
            file_size
        );
    }

    // Verify database state
    let all_tasks = manager.get_all_tasks().await.expect("Should get all tasks");
    assert_eq!(
        all_tasks.len(),
        unique_task_ids.len(),
        "Database should contain exactly {} unique tasks",
        unique_task_ids.len()
    );
}

#[tokio::test]
async fn test_e2e_deduplication_error_handling() {
    // This test MUST FAIL initially (TDD requirement)
    let (_temp_dir, manager) = create_test_manager().await;

    // Test deduplication with various error scenarios
    let invalid_url = "https://nonexistent.example.com/file.zip";
    let valid_url = "https://httpbin.org/bytes/256";
    let target_path = Path::new("./test_downloads/error_test.bin");

    // Step 1: Try download that will fail
    let failing_task_id = manager
        .add_download(invalid_url, target_path, None, None)
        .await
        .expect("Should accept download request even for invalid URL");

    // Step 2: Try duplicate of failing download
    let duplicate_failing_task_id = manager
        .add_download(invalid_url, target_path, None, None)
        .await
        .expect("Should handle duplicate of failing download");

    assert_eq!(
        failing_task_id, duplicate_failing_task_id,
        "Duplicate of failing download should return same task ID"
    );

    // Step 3: Start the failing download
    manager
        .start_download(failing_task_id)
        .await
        .expect("Should attempt to start download");

    // Wait for failure
    let mut checks = 0;
    while checks < 20 {
        checks += 1;

        if let Some(progress) = manager.get_download_progress(failing_task_id).await.expect("Should get progress") {
            if let DownloadProgress::Failed { .. } = progress {
                println!("Download failed as expected");
                break;
            }
        }

        if checks == 20 {
            panic!("Download should have failed by now");
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    // Step 4: Try to download different URL to same path after failure
    let recovery_task_id = manager
        .add_download(valid_url, target_path, None, None)
        .await
        .expect("Should allow recovery download to same path");

    // Should create new task (different URL)
    assert_ne!(
        failing_task_id, recovery_task_id,
        "Recovery download with different URL should create new task"
    );

    // Step 5: Complete the recovery download
    manager
        .start_download(recovery_task_id)
        .await
        .expect("Should start recovery download");

    // Wait for completion
    let mut checks = 0;
    while checks < 30 {
        checks += 1;

        if let Some(progress) = manager.get_download_progress(recovery_task_id).await.expect("Should get progress") {
            match progress {
                DownloadProgress::Completed { .. } => {
                    println!("Recovery download completed");
                    break;
                }
                DownloadProgress::Failed { error, .. } => {
                    panic!("Recovery download failed: {}", error);
                }
                _ => {}
            }
        }

        if checks == 30 {
            panic!("Recovery download should have completed");
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    // Verify recovery file exists
    assert!(
        target_path.exists(),
        "Recovery download should have created the file"
    );
}

// Helper functions for end-to-end testing

async fn create_test_manager() -> (TempDir, PersistentAria2Manager) {
    let temp_dir = TempDir::new().expect("Should create temp directory");

    // Create test manager with temporary database
    let db_path = temp_dir.path().join("e2e_test.db");
    let manager = PersistentAria2Manager::new(&db_path.to_string_lossy())
        .await
        .expect("Should create test manager");

    (temp_dir, manager)
}

// Mock implementations needed for testing
impl PersistentAria2Manager {
    pub async fn get_task_info(&self, task_id: TaskId) -> anyhow::Result<Option<MockTaskInfo>> {
        // This method needs to be implemented for end-to-end testing
        todo!("Need to implement get_task_info for e2e testing")
    }

    pub async fn start_download(&self, task_id: TaskId) -> anyhow::Result<()> {
        // This method needs to be implemented for end-to-end testing
        todo!("Need to implement start_download for e2e testing")
    }

    pub async fn get_download_progress(&self, task_id: TaskId) -> anyhow::Result<Option<DownloadProgress>> {
        // This method needs to be implemented for end-to-end testing
        todo!("Need to implement get_download_progress for e2e testing")
    }
}

#[derive(Debug, Clone)]
pub struct MockTaskInfo {
    pub id: TaskId,
    pub url: String,
    pub target_path: std::path::PathBuf,
    pub status: DownloadStatus,
}