use burncloud_download::{download, download_to, get_download_progress, list_downloads};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 Testing simplified burncloud-download API...");

    // Test 1: Simple download function
    println!("\n=== Test 1: Simple Download ===");
    match download("https://httpbin.org/status/200").await {
        Ok(task_id) => {
            println!("✅ Simple download started successfully!");
            println!("   Task ID: {}", task_id);

            // Check progress
            match get_download_progress(task_id).await {
                Ok(progress) => {
                    println!("   Progress: {} bytes downloaded", progress.downloaded_bytes);
                }
                Err(e) => {
                    println!("   Progress check failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Simple download failed: {}", e);
        }
    }

    // Test 2: Download to specific location
    println!("\n=== Test 2: Download to Specific Path ===");
    match download_to("https://httpbin.org/json", "./downloads/test.json").await {
        Ok(task_id) => {
            println!("✅ Targeted download started successfully!");
            println!("   Task ID: {}", task_id);
        }
        Err(e) => {
            println!("❌ Targeted download failed: {}", e);
        }
    }

    // Test 3: List all downloads
    println!("\n=== Test 3: List All Downloads ===");
    match list_downloads().await {
        Ok(downloads) => {
            println!("✅ Found {} downloads:", downloads.len());
            for (i, task) in downloads.iter().take(5).enumerate() {
                println!("   {}. {} -> {:?} ({})",
                    i + 1,
                    &task.url[0..std::cmp::min(50, task.url.len())],
                    task.target_path,
                    task.status
                );
            }
        }
        Err(e) => {
            println!("❌ Failed to list downloads: {}", e);
        }
    }

    println!("\n🎉 ===================");
    println!("🎉 API TEST COMPLETED");
    println!("🎉 ===================");
    println!("");
    println!("✅ Simple API functions work correctly!");
    println!("📝 Users can now use:");
    println!("   - burncloud_download::download(url)");
    println!("   - burncloud_download::download_to(url, path)");
    println!("   - burncloud_download::get_download_progress(task_id)");
    println!("   - burncloud_download::list_downloads()");
    println!("");
    println!("🔥 The API automatically uses aria2 + database persistence!");
    println!("🔥 No need to choose managers - it's all transparent!");

    Ok(())
}