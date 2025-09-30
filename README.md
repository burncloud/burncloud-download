# BurnCloud Download Manager

A unified download management interface for the BurnCloud platform.

## Features

- **Unified Interface**: Single trait-based interface for all download backends
- **Concurrency Control**: Configurable concurrent download limits (default: 3)
- **Task Lifecycle Management**: Add, pause, resume, cancel, and monitor downloads
- **Event System**: Real-time notifications for status changes and progress updates
- **Async/Await**: Built on Tokio for efficient async operations
- **Type Safety**: Strong typing with comprehensive error handling
- **Extensible Design**: Easy to implement custom download backends

## Core Components

### Data Structures

- **`TaskId`**: Unique identifier for download tasks
- **`DownloadTask`**: Core task representation with metadata
- **`DownloadProgress`**: Progress tracking with completion percentage
- **`DownloadStatus`**: Task status enumeration (Waiting, Downloading, Paused, Completed, Failed)

### Traits

- **`DownloadManager`**: Core interface for download backend implementations
- **`DownloadEventHandler`**: Event notification interface for observers

### Task Queue Manager

- **`TaskQueueManager`**: Built-in queue manager with concurrency control
- Enforces maximum 3 concurrent downloads
- Automatic task scheduling and lifecycle management
- Event notification system

## Usage

```rust
use burncloud_download::{TaskQueueManager, DownloadEventHandler};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create queue manager
    let queue_manager = TaskQueueManager::new();

    // Add a download task
    let task_id = queue_manager.add_task(
        "https://example.com/file.zip".to_string(),
        PathBuf::from("/downloads/file.zip")
    ).await?;

    // Monitor task
    let task = queue_manager.get_task(task_id).await?;
    println!("Status: {}", task.status);

    // Pause task
    queue_manager.pause_task(task_id).await?;

    // Resume task
    queue_manager.resume_task(task_id).await?;

    // Complete or fail task
    queue_manager.complete_task(task_id).await?;

    Ok(())
}
```

## Example

Run the basic usage example:

```bash
cargo run --example basic_usage
```

## Testing

Run all tests:

```bash
cargo test
```

## Architecture

The crate is designed for extensibility:

1. **Core Types**: Define common data structures
2. **Traits**: Provide interfaces for implementations
3. **Queue Manager**: Built-in task management
4. **Error Handling**: Comprehensive error types
5. **Events**: Real-time notifications

Future download backends (qBittorrent, Alist, aria2, etc.) can implement the `DownloadManager` trait to integrate seamlessly.

## Dependencies

- `tokio`: Async runtime
- `anyhow`: Error handling
- `uuid`: Task ID generation
- `serde`: Serialization
- `thiserror`: Custom error types
- `async-trait`: Async trait support

## License

MIT