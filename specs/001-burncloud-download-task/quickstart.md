# Quickstart Guide: Duplicate Download Detection

**Feature**: 001-burncloud-download-task
**Date**: 2025-10-09
**Target Audience**: Developers implementing and using duplicate detection

## Overview

This guide shows how to quickly implement and use duplicate download detection in the burncloud-download system. The feature prevents redundant downloads by detecting existing tasks and reusing them instead of creating new downloads.

## 🚀 Quick Setup

### 1. Database Migration

Add duplicate detection fields to your existing database:

```sql
-- Run this migration script
ALTER TABLE download_tasks ADD COLUMN file_hash TEXT;
ALTER TABLE download_tasks ADD COLUMN file_size_bytes INTEGER;
ALTER TABLE download_tasks ADD COLUMN url_hash TEXT;
ALTER TABLE download_tasks ADD COLUMN last_verified_at TIMESTAMP;

-- Create indexes for fast duplicate detection
CREATE INDEX idx_file_hash ON download_tasks(file_hash) WHERE file_hash IS NOT NULL;
CREATE INDEX idx_url_hash ON download_tasks(url_hash);
CREATE INDEX idx_url_target ON download_tasks(url_hash, target_path);
```

### 2. Add Dependencies

Update your `Cargo.toml`:

```toml
[dependencies]
# Existing dependencies (keep these)
tokio = { version = "1.47", features = ["full"] }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite", "chrono", "uuid"] }
serde = { version = "1.0", features = ["derive"] }

# New dependencies for duplicate detection
blake3 = "1.5"
url = "2.5"
```

### 3. Basic Usage

#### Default Behavior (Automatic Duplicate Detection)

```rust
use burncloud_download::{DownloadManager, PersistentAria2Manager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = PersistentAria2Manager::new("downloads.db").await?;

    // This will automatically check for duplicates
    let task_id = manager.add_download(
        "https://example.com/file.zip",
        Path::new("./downloads/file.zip")
    ).await?;

    println!("Task ID: {} (may be existing task if duplicate found)", task_id);
    Ok(())
}
```

#### Explicit Duplicate Control

```rust
use burncloud_download::{DownloadManager, DuplicatePolicy, DuplicateResult};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = PersistentAria2Manager::new("downloads.db").await?;

    // Check for duplicates explicitly
    let existing = manager.find_duplicate_task(
        "https://example.com/file.zip",
        Path::new("./downloads/file.zip")
    ).await?;

    match existing {
        Some(task_id) => {
            println!("Found existing task: {}", task_id);
            let status = manager.get_task_status(&task_id).await?;
            println!("Status: {:?}", status);
        }
        None => {
            // No duplicate, create new download
            let task_id = manager.add_download(
                "https://example.com/file.zip",
                Path::new("./downloads/file.zip")
            ).await?;
            println!("Created new task: {}", task_id);
        }
    }

    Ok(())
}
```

## 📋 Common Use Cases

### 1. Resume Interrupted Downloads

```rust
async fn resume_or_restart_download(
    manager: &impl DownloadManager,
    url: &str,
    path: &Path,
) -> Result<TaskId, DownloadError> {
    let result = manager.add_download_with_policy(
        url,
        path,
        DuplicatePolicy::ReuseIfIncomplete
    ).await?;

    match result {
        DuplicateResult::ExistingTask { task_id, status, .. } => {
            match status {
                DownloadStatus::Paused => {
                    manager.resume_download(&task_id).await?;
                    println!("Resumed paused download");
                }
                DownloadStatus::Failed(_) => {
                    manager.resume_download(&task_id).await?;
                    println!("Retrying failed download");
                }
                _ => println!("Download already in progress"),
            }
            Ok(task_id)
        }
        DuplicateResult::NewTask(task_id) => {
            println!("Started new download");
            Ok(task_id)
        }
        _ => unreachable!(),
    }
}
```

### 2. Check Before Download

```rust
async fn smart_download(
    manager: &impl DownloadManager,
    url: &str,
    path: &Path,
) -> Result<TaskId, DownloadError> {
    // First check if we already have this file
    if let Some(existing_task) = manager.find_duplicate_task(url, path).await? {
        let status = manager.get_task_status(&existing_task).await?;

        match status {
            DownloadStatus::Completed => {
                println!("File already downloaded!");
                return Ok(existing_task);
            }
            DownloadStatus::Downloading => {
                println!("Download already in progress");
                return Ok(existing_task);
            }
            DownloadStatus::Paused => {
                println!("Resuming paused download");
                manager.resume_download(&existing_task).await?;
                return Ok(existing_task);
            }
            DownloadStatus::Failed(_) => {
                println!("Retrying failed download");
                manager.resume_download(&existing_task).await?;
                return Ok(existing_task);
            }
            _ => {}
        }
    }

    // No suitable existing task, create new one
    manager.add_download(url, path).await
}
```

### 3. Batch Download with Deduplication

```rust
async fn batch_download_with_dedup(
    manager: &impl DownloadManager,
    downloads: Vec<(String, PathBuf)>,
) -> Result<Vec<TaskId>, DownloadError> {
    let mut task_ids = Vec::new();

    for (url, path) in downloads {
        let result = manager.add_download_with_policy(
            &url,
            &path,
            DuplicatePolicy::ReuseExisting
        ).await?;

        let task_id = match result {
            DuplicateResult::NewTask(id) => {
                println!("New download: {} -> {}", url, path.display());
                id
            }
            DuplicateResult::ExistingTask { task_id, reason, .. } => {
                println!("Reusing existing: {} ({:?})", url, reason);
                task_id
            }
            _ => unreachable!(),
        };

        task_ids.push(task_id);
    }

    Ok(task_ids)
}
```

### 4. Force New Download (Bypass Duplicate Detection)

```rust
async fn force_new_download(
    manager: &impl DownloadManager,
    url: &str,
    path: &Path,
) -> Result<TaskId, DownloadError> {
    let result = manager.add_download_with_policy(
        url,
        path,
        DuplicatePolicy::AllowDuplicate
    ).await?;

    match result {
        DuplicateResult::NewTask(task_id) => {
            println!("Forced new download created");
            Ok(task_id)
        }
        _ => unreachable!("AllowDuplicate policy should always create new task"),
    }
}
```

## 🔧 Configuration Options

### Duplicate Detection Policies

```rust
use burncloud_download::DuplicatePolicy;

// Default: Reuse any existing task
let policy = DuplicatePolicy::ReuseExisting;

// Only reuse completed downloads
let policy = DuplicatePolicy::ReuseIfComplete;

// Only reuse incomplete downloads (for resume)
let policy = DuplicatePolicy::ReuseIfIncomplete;

// Always create new download
let policy = DuplicatePolicy::AllowDuplicate;

// Fail if duplicate exists
let policy = DuplicatePolicy::FailIfDuplicate;
```

### Background Hash Calculation

```rust
// Enable background hash calculation for completed files
async fn setup_background_hashing(manager: &impl DownloadManager) {
    // This will be automatically triggered when downloads complete
    // No manual setup required - handled by the download manager
}
```

## 🔍 Monitoring and Debugging

### Check Duplicate Status

```rust
async fn analyze_duplicates(
    manager: &impl DownloadManager,
    url: &str,
    path: &Path,
) -> Result<(), DownloadError> {
    // Get all potential duplicates
    let candidates = manager.get_duplicate_candidates(url, path).await?;

    println!("Found {} potential duplicates:", candidates.len());

    for task_id in candidates {
        let status = manager.get_task_status(&task_id).await?;
        let is_valid = manager.verify_task_validity(&task_id).await?;

        println!("  {}: {:?} (valid: {})", task_id, status, is_valid);
    }

    Ok(())
}
```

### Event Monitoring

```rust
use burncloud_download::{DownloadEvent, DuplicateReason};

async fn monitor_duplicate_events(mut event_receiver: tokio::sync::mpsc::Receiver<DownloadEvent>) {
    while let Some(event) = event_receiver.recv().await {
        match event {
            DownloadEvent::DuplicateDetected { task_id, duplicate_of, reason } => {
                println!("Duplicate detected: {} is duplicate of {} ({:?})",
                        task_id, duplicate_of, reason);
            }
            DownloadEvent::TaskReused { task_id, previous_status } => {
                println!("Task {} reused (was {:?})", task_id, previous_status);
            }
            DownloadEvent::HashCalculated { task_id, file_hash } => {
                println!("Hash calculated for {}: {}", task_id, file_hash);
            }
            _ => {} // Handle other events
        }
    }
}
```

## ⚡ Performance Tips

### 1. Optimize Database Queries

```rust
// Batch queries for multiple duplicates
async fn check_multiple_duplicates(
    manager: &impl DownloadManager,
    urls_and_paths: &[(String, PathBuf)],
) -> Result<Vec<Option<TaskId>>, DownloadError> {
    let mut results = Vec::new();

    // Process in batches for better performance
    for chunk in urls_and_paths.chunks(10) {
        let batch_results = futures::future::join_all(
            chunk.iter().map(|(url, path)|
                manager.find_duplicate_task(url, path)
            )
        ).await;

        for result in batch_results {
            results.push(result?);
        }
    }

    Ok(results)
}
```

### 2. Cache Frequently Checked URLs

```rust
use std::collections::HashMap;
use tokio::sync::RwLock;

struct DuplicateCache {
    cache: RwLock<HashMap<String, Option<TaskId>>>,
}

impl DuplicateCache {
    async fn check_or_fetch(
        &self,
        url: &str,
        path: &Path,
        manager: &impl DownloadManager,
    ) -> Result<Option<TaskId>, DownloadError> {
        let cache_key = format!("{}:{}", url, path.display());

        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(*cached);
            }
        }

        // Not in cache, fetch from database
        let result = manager.find_duplicate_task(url, path).await?;

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(cache_key, result);
        }

        Ok(result)
    }
}
```

## 🧪 Testing Your Implementation

### Unit Test Example

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_duplicate_detection() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let manager = PersistentAria2Manager::new(&db_path).await.unwrap();

        let url = "https://example.com/test.zip";
        let path = temp_dir.path().join("test.zip");

        // First download - should create new task
        let task1 = manager.add_download(url, &path).await.unwrap();

        // Second download - should detect duplicate
        let duplicate = manager.find_duplicate_task(url, &path).await.unwrap();
        assert_eq!(duplicate, Some(task1));

        // Third download with reuse policy - should return existing
        let result = manager.add_download_with_policy(
            url,
            &path,
            DuplicatePolicy::ReuseExisting
        ).await.unwrap();

        match result {
            DuplicateResult::ExistingTask { task_id, .. } => {
                assert_eq!(task_id, task1);
            }
            _ => panic!("Expected existing task"),
        }
    }
}
```

## 🚨 Common Pitfalls

### 1. URL Normalization Issues

```rust
// ❌ Don't compare raw URLs
let url1 = "https://example.com/file.zip";
let url2 = "https://example.com/file.zip?param=value";
// These are treated as different downloads

// ✅ Use the API which handles normalization
let duplicate = manager.find_duplicate_task(url2, path).await?;
// This will find the duplicate correctly
```

### 2. Path Sensitivity

```rust
// ❌ Path differences matter
let path1 = Path::new("./downloads/file.zip");
let path2 = Path::new("downloads/file.zip");
// These are treated as different downloads

// ✅ Use consistent absolute paths
let path = std::fs::canonicalize("downloads/file.zip")?;
let task_id = manager.add_download(url, &path).await?;
```

### 3. Task Validity

```rust
// ❌ Don't assume existing tasks are always valid
let existing = manager.find_duplicate_task(url, path).await?;
if let Some(task_id) = existing {
    // File might have been deleted, source might be unavailable
    manager.resume_download(&task_id).await?; // This might fail
}

// ✅ Verify task validity first
let existing = manager.find_duplicate_task(url, path).await?;
if let Some(task_id) = existing {
    if manager.verify_task_validity(&task_id).await? {
        manager.resume_download(&task_id).await?;
    } else {
        // Task is stale, create new download
        let new_task = manager.add_download_with_policy(
            url, path, DuplicatePolicy::AllowDuplicate
        ).await?;
    }
}
```

## 📚 Next Steps

1. **Implement TDD**: Follow the constitution requirement - write tests first!
2. **Database Migration**: Apply the schema changes to your database
3. **Integration**: Add duplicate detection to your existing download workflows
4. **Monitoring**: Set up event monitoring to track duplicate detection performance
5. **Optimization**: Implement caching if you have high-frequency duplicate checks

For more advanced usage and implementation details, see:
- [Data Model Documentation](./data-model.md)
- [API Contracts](./contracts/duplicate_detection_api.md)
- [Implementation Plan](./plan.md)