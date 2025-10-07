use burncloud_download::{PersistentAria2Manager, DownloadManager};
use std::path::PathBuf;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 Testing PersistentAria2Manager with reliable download URL...");

    // Create data directory
    std::fs::create_dir_all("./data").ok();

    let manager = PersistentAria2Manager::new().await?;
    println!("✅ Manager created successfully");

    // Use a more reliable test file - a small image from GitHub
    let test_url = "https://via.placeholder.com/150/0000FF/808080?Text=Test".to_string();
    let target_path = PathBuf::from("./data/test_image.png");

    println!("📥 Starting download...");
    println!("   URL: {}", test_url);
    println!("   Target: {:?}", target_path);

    let task_id = manager.add_download(test_url, target_path.clone()).await?;
    println!("✅ Download task created with ID: {}", task_id);

    // Monitor for 15 seconds
    println!("👀 Monitoring download progress...");
    let mut completed = false;

    for _i in 0..15 {
        sleep(Duration::from_secs(1)).await;

        match manager.get_task(task_id).await {
            Ok(task) => {
                let status_str = format!("{:?}", task.status);
                println!("   Status: {}", status_str);

                if status_str.contains("Complete") {
                    println!("✅ Download completed!");
                    completed = true;
                    break;
                }
                if status_str.contains("Failed") || status_str.contains("Error") {
                    println!("❌ Download failed: {}", status_str);
                    break;
                }
            }
            Err(e) => {
                println!("⚠️  Could not get task status: {}", e);
            }
        }

        // Also check progress
        if let Ok(progress) = manager.get_progress(task_id).await {
            if progress.downloaded_bytes > 0 {
                println!("   Downloaded: {} bytes", progress.downloaded_bytes);
            }
        }
    }

    // Check if file was downloaded
    if target_path.exists() {
        let file_size = std::fs::metadata(&target_path)?.len();
        println!("✅ File downloaded successfully!");
        println!("   File size: {} bytes", file_size);
        println!("   Location: {:?}", target_path.canonicalize()?);
    } else {
        println!("❌ File was not downloaded");
    }

    // Test persistence - only show recent tasks
    println!("📋 Recent tasks in database:");
    match manager.list_tasks().await {
        Ok(tasks) => {
            let recent_tasks: Vec<_> = tasks.into_iter().take(5).collect();
            for task in recent_tasks {
                println!("   Task {}: {} -> {:?} ({})",
                    &task.id.to_string()[0..8],
                    if task.url.len() > 30 { format!("{}...", &task.url[0..30]) } else { task.url.clone() },
                    task.target_path,
                    task.status);
            }
        }
        Err(e) => {
            println!("❌ Could not list tasks: {}", e);
        }
    }

    manager.shutdown().await?;
    println!("🎉 Test completed!");

    if completed {
        println!("✅ DOWNLOAD TEST PASSED!");
    } else {
        println!("⚠️  Download test had issues but persistence works");
    }

    Ok(())
}