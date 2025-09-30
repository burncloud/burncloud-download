use burncloud_download::{
    DownloadManager, TaskQueueManager, BasicDownloadManager,
    DownloadEventHandler, DownloadStatus, DownloadProgress, TaskId
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use async_trait::async_trait;

// Example event handler implementation
struct LoggingEventHandler {
    logs: Arc<Mutex<Vec<String>>>,
    name: String,
}

#[async_trait]
impl DownloadEventHandler for LoggingEventHandler {
    async fn on_status_changed(&self, task_id: TaskId, old_status: DownloadStatus, new_status: DownloadStatus) {
        let mut logs = self.logs.lock().await;
        logs.push(format!("[{}] Task {} status changed: {} -> {}", self.name, task_id, old_status, new_status));
    }

    async fn on_progress_updated(&self, task_id: TaskId, progress: DownloadProgress) {
        let mut logs = self.logs.lock().await;
        if let Some(percentage) = progress.completion_percentage() {
            logs.push(format!("[{}] Task {} progress: {:.1}%", self.name, task_id, percentage));
        }
    }

    async fn on_download_completed(&self, task_id: TaskId) {
        let mut logs = self.logs.lock().await;
        logs.push(format!("[{}] Task {} completed successfully", self.name, task_id));
    }

    async fn on_download_failed(&self, task_id: TaskId, error: String) {
        let mut logs = self.logs.lock().await;
        logs.push(format!("[{}] Task {} failed: {}", self.name, task_id, error));
    }
}

async fn demonstrate_basic_manager() -> anyhow::Result<()> {
    println!("\n=== BasicDownloadManager Demo ===");

    let manager: Arc<dyn DownloadManager> = Arc::new(BasicDownloadManager::new());

    // Add a download task
    println!("1. Adding download task...");
    let task_id = manager.add_download(
        "https://example.com/demo-file.zip".to_string(),
        PathBuf::from("/downloads/demo-file.zip")
    ).await?;

    println!("Task ID: {}", task_id);

    // Monitor progress for a few seconds
    println!("2. Monitoring progress...");
    for _i in 0..5 {
        let progress = manager.get_progress(task_id).await?;
        if let Some(percentage) = progress.completion_percentage() {
            println!("   Progress: {:.1}% ({} / {} bytes)",
                percentage,
                progress.downloaded_bytes,
                progress.total_bytes.unwrap_or(0)
            );
        }

        let task = manager.get_task(task_id).await?;
        if task.status.is_finished() {
            println!("   Download finished: {}", task.status);
            break;
        }

        sleep(Duration::from_millis(500)).await;
    }

    // Test pause/resume
    println!("3. Testing pause/resume...");
    let task = manager.get_task(task_id).await?;
    if !task.status.is_finished() {
        manager.pause_download(task_id).await?;
        println!("   Paused download");

        sleep(Duration::from_millis(100)).await;

        manager.resume_download(task_id).await?;
        println!("   Resumed download");
    }

    Ok(())
}

async fn demonstrate_queue_manager() -> anyhow::Result<()> {
    println!("\n=== TaskQueueManager Demo ===");

    // Create queue manager
    let queue_manager = TaskQueueManager::new();

    // Create event handler
    let logs = Arc::new(Mutex::new(Vec::new()));
    let handler = Arc::new(LoggingEventHandler {
        logs: logs.clone(),
        name: "Queue".to_string(),
    });

    // Add event handler
    queue_manager.add_event_handler(handler).await;

    // Add some download tasks via the DownloadManager trait
    println!("1. Adding download tasks...");
    let task1 = queue_manager.add_download(
        "https://example.com/file1.zip".to_string(),
        PathBuf::from("/downloads/file1.zip")
    ).await?;

    let _task2 = queue_manager.add_download(
        "https://example.com/file2.pdf".to_string(),
        PathBuf::from("/downloads/file2.pdf")
    ).await?;

    let _task3 = queue_manager.add_download(
        "https://example.com/file3.mp4".to_string(),
        PathBuf::from("/downloads/file3.mp4")
    ).await?;

    // Add more tasks (these should be queued due to concurrency limit)
    let _task4 = queue_manager.add_download(
        "https://example.com/file4.jpg".to_string(),
        PathBuf::from("/downloads/file4.jpg")
    ).await?;

    println!("Added 4 tasks");
    println!("Active downloads: {}", queue_manager.active_download_count().await);

    // List all tasks
    println!("\n2. Current tasks:");
    let tasks = queue_manager.list_tasks().await?;
    for task in &tasks {
        println!("  {} - {} [{}]", task.id, task.url, task.status);
    }

    // Test progress tracking
    println!("\n3. Testing progress tracking...");
    let progress = DownloadProgress {
        downloaded_bytes: 5120,
        total_bytes: Some(10240),
        speed_bps: 1024,
        eta_seconds: Some(5),
    };

    queue_manager.update_progress(task1, progress).await?;

    let retrieved_progress = queue_manager.get_progress(task1).await?;
    if let Some(percentage) = retrieved_progress.completion_percentage() {
        println!("   Task 1 progress: {:.1}%", percentage);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("BurnCloud Download Manager Example");
    println!("=================================");

    // Demonstrate BasicDownloadManager
    demonstrate_basic_manager().await?;

    // Demonstrate TaskQueueManager
    demonstrate_queue_manager().await?;

    println!("\nBoth implementations successfully demonstrate:");
    println!("✓ DownloadManager trait implementation");
    println!("✓ Progress tracking and updates");
    println!("✓ Task lifecycle management");
    println!("✓ Event notification system");
    println!("✓ API consistency across implementations");

    println!("\nExample completed successfully!");

    Ok(())
}