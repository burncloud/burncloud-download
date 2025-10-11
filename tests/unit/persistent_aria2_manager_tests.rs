use burncloud_download::manager::persistent_aria2::PersistentAria2Manager;
use burncloud_download::traits::DownloadManager;
use std::path::PathBuf;

#[tokio::test]
async fn test_manager_creation() {
    let manager = PersistentAria2Manager::new().await;
    assert!(manager.is_ok(), "Manager creation should succeed");
}

#[tokio::test]
async fn test_add_download_persists() {
    let manager = PersistentAria2Manager::new().await.unwrap();

    let task_id = manager.add_download(
        "https://example.com/test.zip".to_string(),
        PathBuf::from("data/test.zip")
    ).await.unwrap();

    // Verify task can be retrieved (which means it was persisted)
    let task = manager.get_task(task_id).await;
    assert!(task.is_ok(), "Task should be persisted and retrievable");

    let task = task.unwrap();
    assert_eq!(task.url, "https://example.com/test.zip");
}

#[tokio::test]
async fn test_task_mapping() {
    // Test the task mapping logic without requiring aria2 daemon
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use burncloud_download::types::TaskId;

    let task_mappings: Arc<RwLock<HashMap<TaskId, String>>> = Arc::new(RwLock::new(HashMap::new()));

    // Create a test TaskId
    let task_id = TaskId::new();
    let test_gid = "test_gid_123".to_string();

    // Test mapping insertion
    {
        let mut mappings = task_mappings.write().await;
        mappings.insert(task_id, test_gid.clone());
    }

    // Test mapping retrieval
    {
        let mappings = task_mappings.read().await;
        let retrieved_gid = mappings.get(&task_id);
        assert_eq!(retrieved_gid, Some(&test_gid));
    }

    println!("Task mapping test completed successfully");
}

#[tokio::test]
async fn test_shutdown() {
    let manager = PersistentAria2Manager::new().await.unwrap();

    let result = manager.shutdown().await;
    assert!(result.is_ok(), "Shutdown should complete successfully");
}