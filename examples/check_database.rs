use burncloud_database_download::{DownloadRepository, Database};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("📂 Current database status:");
    println!("Database path: C:\\Users\\huang\\AppData\\Local\\BurnCloud\\data.db");

    let db = Database::new_default_initialized().await?;
    let repo = DownloadRepository::new(db);
    repo.initialize().await?;

    let count = repo.count_tasks().await?;
    println!("🔢 Total tasks in database: {}", count);

    if count > 0 {
        println!("\n📋 All tasks:");
        let tasks = repo.list_tasks().await?;
        for (i, task) in tasks.iter().enumerate() {
            println!("{}. ID: {}", i + 1, task.id);
            println!("   URL: {}", task.url);
            println!("   Path: {:?}", task.target_path);
            println!("   Status: {}", task.status);
            println!("   Created: {:?}", task.created_at);
            println!("   Updated: {:?}", task.updated_at);
            println!();
        }
    } else {
        println!("❌ No tasks found in database");
    }

    // Also show raw file info
    let db_path = std::path::Path::new("C:\\Users\\huang\\AppData\\Local\\BurnCloud\\data.db");
    if db_path.exists() {
        let metadata = std::fs::metadata(&db_path)?;
        println!("📁 Database file:");
        println!("   Size: {} bytes", metadata.len());
        println!("   Modified: {:?}", metadata.modified()?);
    } else {
        println!("❌ Database file does not exist at expected path");
    }

    Ok(())
}