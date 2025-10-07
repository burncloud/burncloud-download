use burncloud_database_download::{DownloadRepository, Database};
use burncloud_download_types::DownloadTask;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔍 Testing actual database operations...");

    // Create database with the same path as PersistentAria2Manager
    println!("📁 Creating database connection...");
    let db = Database::new_default_initialized().await?;
    println!("✅ Database created successfully");

    // Create repository
    let repository = DownloadRepository::new(db);

    // Initialize schema
    println!("🏗️  Initializing database schema...");
    repository.initialize().await?;
    println!("✅ Schema initialized");

    // Create a test task
    println!("📝 Creating test task...");
    let task = DownloadTask::new(
        "https://test.example.com/file.txt".to_string(),
        PathBuf::from("./data/test.txt")
    );
    println!("✅ Task created with ID: {}", task.id);

    // Save the task
    println!("💾 Saving task to database...");
    match repository.save_task(&task).await {
        Ok(_) => println!("✅ Task saved successfully"),
        Err(e) => {
            println!("❌ Failed to save task: {}", e);
            return Err(e.into());
        }
    }

    // Try to retrieve the task
    println!("🔍 Retrieving task from database...");
    match repository.get_task(&task.id).await {
        Ok(retrieved_task) => {
            println!("✅ Task retrieved successfully:");
            println!("   ID: {}", retrieved_task.id);
            println!("   URL: {}", retrieved_task.url);
            println!("   Path: {:?}", retrieved_task.target_path);
        }
        Err(e) => {
            println!("❌ Failed to retrieve task: {}", e);
        }
    }

    // List all tasks
    println!("📋 Listing all tasks...");
    match repository.list_tasks().await {
        Ok(tasks) => {
            println!("✅ Found {} tasks in database:", tasks.len());
            for (i, t) in tasks.iter().enumerate() {
                println!("   {}. {} -> {:?}", i + 1, t.url, t.target_path);
            }
        }
        Err(e) => {
            println!("❌ Failed to list tasks: {}", e);
        }
    }

    // Count tasks
    println!("🔢 Counting tasks...");
    match repository.count_tasks().await {
        Ok(count) => {
            println!("✅ Total tasks in database: {}", count);
        }
        Err(e) => {
            println!("❌ Failed to count tasks: {}", e);
        }
    }

    println!("🎉 Database test completed!");
    Ok(())
}