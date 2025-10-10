# Duplicate Detection API Contracts

## Overview

This document defines the Rust trait contracts and method signatures for duplicate download detection functionality. These contracts extend the existing `DownloadManager` trait while maintaining backward compatibility.

## Core Trait Extensions

### DownloadManager Trait (Extended)

```rust
#[async_trait]
pub trait DownloadManager {
    // Existing methods (unchanged)
    async fn add_download(&self, url: &str, target_path: &Path) -> Result<TaskId, DownloadError>;
    async fn pause_download(&self, task_id: &TaskId) -> Result<(), DownloadError>;
    async fn resume_download(&self, task_id: &TaskId) -> Result<(), DownloadError>;
    async fn cancel_download(&self, task_id: &TaskId) -> Result<(), DownloadError>;
    async fn get_task_status(&self, task_id: &TaskId) -> Result<DownloadStatus, DownloadError>;
    async fn get_download_progress(&self, task_id: &TaskId) -> Result<DownloadProgress, DownloadError>;
    async fn list_active_downloads(&self) -> Result<Vec<TaskId>, DownloadError>;

    // New methods for duplicate detection
    async fn find_duplicate_task(
        &self,
        url: &str,
        target_path: &Path,
    ) -> Result<Option<TaskId>, DownloadError>;

    async fn add_download_with_policy(
        &self,
        url: &str,
        target_path: &Path,
        policy: DuplicatePolicy,
    ) -> Result<DuplicateResult, DownloadError>;

    async fn verify_task_validity(&self, task_id: &TaskId) -> Result<bool, DownloadError>;

    async fn get_duplicate_candidates(
        &self,
        url: &str,
        target_path: &Path,
    ) -> Result<Vec<TaskId>, DownloadError>;
}
```

## New Types and Enums

### DuplicateResult

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DuplicateResult {
    /// New task was created
    NewTask(TaskId),
    /// Existing task was found and will be reused
    ExistingTask {
        task_id: TaskId,
        status: DownloadStatus,
        reason: DuplicateReason,
    },
    /// User interaction required
    RequiresDecision {
        candidates: Vec<TaskId>,
        suggested_action: DuplicateAction,
    },
}
```

### DuplicateReason

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DuplicateReason {
    /// Same URL and target path
    UrlAndPath,
    /// Same file content hash
    FileContent,
    /// Similar URL (normalized comparison)
    SimilarUrl,
    /// Same filename in target directory
    Filename,
}
```

### DuplicateAction

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DuplicateAction {
    Resume(TaskId),
    Reuse(TaskId),
    Retry(TaskId),
    CreateNew,
}
```

### DuplicatePolicy

```rust
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DuplicatePolicy {
    #[default]
    ReuseExisting,
    AllowDuplicate,
    PromptUser,
    ReuseIfComplete,
    ReuseIfIncomplete,
    FailIfDuplicate,
}
```

## Method Specifications

### find_duplicate_task

**Purpose**: Find existing task for the same download request

**Signature**:
```rust
async fn find_duplicate_task(
    &self,
    url: &str,
    target_path: &Path,
) -> Result<Option<TaskId>, DownloadError>
```

**Parameters**:
- `url`: Source URL for download
- `target_path`: Target file path

**Returns**:
- `Some(TaskId)`: Existing task found
- `None`: No duplicate found

**Behavior**:
1. Normalize URL and generate hash
2. Query database for matching url_hash + target_path
3. Return most recent valid task if found
4. Prefer active/paused tasks over completed/failed

**Example Usage**:
```rust
let existing = manager.find_duplicate_task("https://example.com/file.zip", "/downloads/file.zip").await?;
match existing {
    Some(task_id) => println!("Found existing task: {}", task_id),
    None => println!("No duplicate found"),
}
```

### add_download_with_policy

**Purpose**: Add download with explicit duplicate handling policy

**Signature**:
```rust
async fn add_download_with_policy(
    &self,
    url: &str,
    target_path: &Path,
    policy: DuplicatePolicy,
) -> Result<DuplicateResult, DownloadError>
```

**Parameters**:
- `url`: Source URL for download
- `target_path`: Target file path
- `policy`: How to handle duplicates

**Returns**:
- `DuplicateResult`: Outcome of duplicate detection and policy application

**Behavior by Policy**:
- `ReuseExisting`: Return existing task if found, create new if not
- `AllowDuplicate`: Always create new task
- `PromptUser`: Return candidates for user decision
- `ReuseIfComplete`: Reuse only completed tasks
- `ReuseIfIncomplete`: Reuse only incomplete tasks
- `FailIfDuplicate`: Return error if duplicate found

**Example Usage**:
```rust
let result = manager.add_download_with_policy(
    "https://example.com/file.zip",
    "/downloads/file.zip",
    DuplicatePolicy::ReuseExisting,
).await?;

match result {
    DuplicateResult::NewTask(task_id) => {
        println!("Created new task: {}", task_id);
    }
    DuplicateResult::ExistingTask { task_id, status, reason } => {
        println!("Reusing task {} ({}): {:?}", task_id, reason, status);
    }
    DuplicateResult::RequiresDecision { candidates, suggested_action } => {
        println!("Found {} candidates, suggest: {:?}", candidates.len(), suggested_action);
    }
}
```

### verify_task_validity

**Purpose**: Check if existing task is still valid for reuse

**Signature**:
```rust
async fn verify_task_validity(&self, task_id: &TaskId) -> Result<bool, DownloadError>
```

**Parameters**:
- `task_id`: Task to verify

**Returns**:
- `true`: Task is valid and can be reused
- `false`: Task is invalid (file deleted, source unavailable, etc.)

**Verification Checks**:
1. Task exists in database
2. Target file exists (for completed tasks)
3. Source URL is accessible (HTTP HEAD request)
4. File integrity (hash verification if available)

**Example Usage**:
```rust
let is_valid = manager.verify_task_validity(&task_id).await?;
if is_valid {
    println!("Task can be safely reused");
} else {
    println!("Task is stale, need new download");
}
```

### get_duplicate_candidates

**Purpose**: Get all potential duplicate tasks for advanced duplicate resolution

**Signature**:
```rust
async fn get_duplicate_candidates(
    &self,
    url: &str,
    target_path: &Path,
) -> Result<Vec<TaskId>, DownloadError>
```

**Parameters**:
- `url`: Source URL for download
- `target_path`: Target file path

**Returns**:
- `Vec<TaskId>`: All tasks that could be considered duplicates

**Detection Hierarchy**:
1. Exact URL and path matches
2. Normalized URL matches with different paths
3. File content hash matches (if available)
4. Filename matches in target directory

**Example Usage**:
```rust
let candidates = manager.get_duplicate_candidates(
    "https://example.com/file.zip",
    "/downloads/file.zip"
).await?;

for task_id in candidates {
    let status = manager.get_task_status(&task_id).await?;
    println!("Candidate {}: {:?}", task_id, status);
}
```

## Error Handling

### DownloadError Extensions

```rust
#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    // Existing errors (unchanged)
    #[error("Task not found: {0}")]
    TaskNotFound(TaskId),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    // New errors for duplicate detection
    #[error("Duplicate detection failed: {0}")]
    DuplicateDetectionError(String),

    #[error("Task verification failed: {0}")]
    VerificationError(String),

    #[error("Policy violation: {reason}, found duplicate task {task_id}")]
    PolicyViolation { task_id: TaskId, reason: String },
}
```

## Async Patterns

### Concurrency Considerations

**Thread Safety**:
- All methods are async and thread-safe
- Database operations use connection pools
- Duplicate detection queries are read-heavy and parallelizable

**Performance Optimizations**:
```rust
// Batch duplicate detection for multiple URLs
async fn find_duplicate_tasks_batch(
    &self,
    requests: &[(String, PathBuf)],
) -> Result<Vec<Option<TaskId>>, DownloadError>;

// Background hash calculation
async fn calculate_file_hash_background(&self, task_id: &TaskId) -> Result<(), DownloadError>;
```

### Error Recovery

**Transient Failures**:
- Network errors during verification → retry with backoff
- Database connection errors → use connection pool retry
- File system errors → graceful degradation

**Permanent Failures**:
- Invalid URL format → immediate error return
- Missing file for verification → mark task as invalid
- Database schema mismatch → migration required

## Integration with Existing APIs

### Backward Compatibility

**Default Behavior**:
```rust
// Original method unchanged
async fn add_download(&self, url: &str, target_path: &Path) -> Result<TaskId, DownloadError> {
    // Internally calls add_download_with_policy with ReuseExisting
    match self.add_download_with_policy(url, target_path, DuplicatePolicy::ReuseExisting).await? {
        DuplicateResult::NewTask(task_id) => Ok(task_id),
        DuplicateResult::ExistingTask { task_id, .. } => Ok(task_id),
        DuplicateResult::RequiresDecision { .. } => {
            // Fallback to creating new task
            self.add_download_with_policy(url, target_path, DuplicatePolicy::AllowDuplicate).await
                .and_then(|r| match r {
                    DuplicateResult::NewTask(task_id) => Ok(task_id),
                    _ => unreachable!(),
                })
        }
    }
}
```

### Event System Integration

**New Events**:
```rust
#[derive(Debug, Clone)]
pub enum DownloadEvent {
    // Existing events (unchanged)
    TaskCreated(TaskId),
    ProgressUpdate(TaskId, DownloadProgress),
    TaskCompleted(TaskId),
    TaskFailed(TaskId, String),

    // New events for duplicate detection
    DuplicateDetected {
        task_id: TaskId,
        duplicate_of: TaskId,
        reason: DuplicateReason,
    },
    TaskReused {
        task_id: TaskId,
        previous_status: DownloadStatus,
    },
    HashCalculated {
        task_id: TaskId,
        file_hash: String,
    },
}
```

## Testing Contracts

### Unit Test Requirements

**Mock Implementation**:
```rust
#[async_trait]
impl DownloadManager for MockManager {
    async fn find_duplicate_task(&self, url: &str, target_path: &Path) -> Result<Option<TaskId>, DownloadError> {
        // Test implementation with controlled responses
    }

    // ... other methods
}
```

**Test Scenarios**:
1. **Exact Duplicates**: Same URL and path → returns existing task
2. **No Duplicates**: Different URL and path → returns None
3. **Policy Application**: Each policy type → correct behavior
4. **Error Conditions**: Invalid input → appropriate errors
5. **Concurrency**: Multiple simultaneous duplicate checks → consistent results

### Integration Test Requirements

**Database Integration**:
- Real SQLite database with test data
- Schema migration testing
- Index performance verification
- Concurrent access patterns

**End-to-End Flows**:
- Complete duplicate detection workflow
- Background hash calculation
- Task validity verification
- Event emission verification

**Performance Benchmarks**:
- Duplicate detection latency < 1 second (per success criteria)
- Database query performance under load
- Memory usage with large task sets