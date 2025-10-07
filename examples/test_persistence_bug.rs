use burncloud_download::{PersistentAria2Manager, DownloadManager};
use burncloud_database_download::{DownloadRepository, Database};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔍 Testing PersistentAria2Manager database operations...");

    // Test 1: Check database path consistency
    println!("\n=== Test 1: Database Path ===");
    let db1 = Database::new_default_initialized().await?;
    let repo1 = DownloadRepository::new(db1);
    repo1.initialize().await?;

    let initial_count = repo1.count_tasks().await?;
    println!("Initial task count in database: {}", initial_count);

    // Test 2: Use PersistentAria2Manager
    println!("\n=== Test 2: PersistentAria2Manager ===");
    let manager = match PersistentAria2Manager::new().await {
        Ok(m) => {
            println!("✅ Manager created successfully");
            m
        }
        Err(e) => {
            println!("❌ Failed to create manager: {}", e);
            return Err(e);
        }
    };

    // Add a task through the manager
    println!("📝 Adding task through manager...");
    let task_id = match manager.add_download(
        "https://test-manager.example.com/file.txt".to_string(),
        PathBuf::from("./data/manager_test.txt")
    ).await {
        Ok(id) => {
            println!("✅ Task added with ID: {}", id);
            id
        }
        Err(e) => {
            println!("❌ Failed to add task: {}", e);
            return Err(e);
        }
    };

    // Test 3: Check if task was saved directly in database
    println!("\n=== Test 3: Direct Database Check ===");
    let db2 = Database::new_default_initialized().await?;
    let repo2 = DownloadRepository::new(db2);
    repo2.initialize().await?;

    let final_count = repo2.count_tasks().await?;
    println!("Final task count in database: {}", final_count);

    if final_count > initial_count {
        println!("✅ Task was saved to database!");

        // List all tasks
        match repo2.list_tasks().await {
            Ok(tasks) => {
                println!("Tasks found:");
                for task in tasks {
                    println!("  - {} -> {:?} ({})", task.url, task.target_path, task.status);
                }
            }
            Err(e) => {
                println!("❌ Failed to list tasks: {}", e);
            }
        }
    } else {
        println!("❌ NO NEW TASKS FOUND IN DATABASE!");
        println!("   This means PersistentAria2Manager is NOT saving to database");
    }

    // Test 4: Check through manager's list_tasks
    println!("\n=== Test 4: Manager List Tasks ===");
    match manager.list_tasks().await {
        Ok(tasks) => {
            println!("Manager reports {} tasks:", tasks.len());
            for task in tasks.iter().take(3) {
                println!("  - {} -> {:?} ({})", task.url, task.target_path, task.status);
            }
        }
        Err(e) => {
            println!("❌ Manager failed to list tasks: {}", e);
        }
    }

    // Shutdown manager
    println!("\n=== Test 5: Manager Shutdown ===");
    match manager.shutdown().await {
        Ok(_) => println!("✅ Manager shutdown successfully"),
        Err(e) => println!("⚠️  Shutdown warning: {}", e),
    }

    // Final check after shutdown
    println!("\n=== Final Check After Shutdown ===");
    let db3 = Database::new_default_initialized().await?;
    let repo3 = DownloadRepository::new(db3);
    repo3.initialize().await?;

    let shutdown_count = repo3.count_tasks().await?;
    println!("Task count after shutdown: {}", shutdown_count);

    if shutdown_count > initial_count {
        println!("✅ Data persisted after shutdown!");
    } else {
        println!("❌ NO DATA PERSISTED - BUG CONFIRMED!");
    }

    Ok(())
}