use burncloud_download::{PersistentAria2Manager, DownloadManager};
use std::path::PathBuf;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 Testing PersistentAria2Manager download functionality...");

    // Create manager
    println!("📁 Creating PersistentAria2Manager...");
    let manager = match PersistentAria2Manager::new().await {
        Ok(m) => {
            println!("✅ Manager created successfully");
            m
        }
        Err(e) => {
            println!("❌ Failed to create manager: {}", e);
            println!("💡 Note: This requires aria2 to be running on localhost:6800");
            println!("   You can start aria2 with: aria2c --enable-rpc --rpc-secret=burncloud");
            return Err(e);
        }
    };

    // Test download - using a small test file
    let test_url = "https://httpbin.org/bytes/1024".to_string(); // 1KB test file
    let target_path = PathBuf::from("./data/test_file.bin");

    println!("📥 Starting download...");
    println!("   URL: {}", test_url);
    println!("   Target: {:?}", target_path);

    let task_id = match manager.add_download(test_url, target_path.clone()).await {
        Ok(id) => {
            println!("✅ Download task created with ID: {}", id);
            id
        }
        Err(e) => {
            println!("❌ Failed to add download: {}", e);
            return Err(e);
        }
    };

    // Monitor progress for 30 seconds
    println!("👀 Monitoring download progress...");
    for i in 0..30 {
        sleep(Duration::from_secs(1)).await;

        match manager.get_progress(task_id).await {
            Ok(progress) => {
                let total = progress.total_bytes.unwrap_or(0);
                let downloaded = progress.downloaded_bytes;
                let percentage = if total > 0 { (downloaded * 100) / total } else { 0 };

                println!("   Progress: {}/{} bytes ({}%) - Speed: {} B/s",
                    downloaded, total, percentage, progress.speed_bps);

                if downloaded >= total && total > 0 {
                    println!("✅ Download completed!");
                    break;
                }
            }
            Err(e) => {
                println!("⚠️  Could not get progress: {}", e);
            }
        }

        // Check task status
        match manager.get_task(task_id).await {
            Ok(task) => {
                println!("   Status: {:?}", task.status);
                if task.status.to_string().contains("Complete") {
                    println!("✅ Task completed!");
                    break;
                }
                if task.status.to_string().contains("Error") || task.status.to_string().contains("Failed") {
                    println!("❌ Download failed!");
                    break;
                }
            }
            Err(e) => {
                println!("⚠️  Could not get task status: {}", e);
            }
        }
    }

    // Check if file exists
    if target_path.exists() {
        let file_size = std::fs::metadata(&target_path)?.len();
        println!("✅ File downloaded successfully!");
        println!("   File size: {} bytes", file_size);
        println!("   Location: {:?}", target_path.canonicalize()?);
    } else {
        println!("❌ File was not downloaded to expected location");
    }

    // Test persistence by listing all tasks
    println!("📋 Checking task persistence...");
    match manager.list_tasks().await {
        Ok(tasks) => {
            println!("✅ Found {} tasks in database:", tasks.len());
            for task in tasks {
                println!("   Task {}: {} -> {:?} ({})",
                    task.id, task.url, task.target_path, task.status);
            }
        }
        Err(e) => {
            println!("❌ Could not list tasks: {}", e);
        }
    }

    // Graceful shutdown
    println!("🔄 Shutting down manager...");
    match manager.shutdown().await {
        Ok(_) => println!("✅ Manager shutdown successfully"),
        Err(e) => println!("⚠️  Shutdown warning: {}", e),
    }

    println!("🎉 Test completed!");
    Ok(())
}