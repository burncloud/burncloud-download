//! # BurnCloud Download Manager
//!
//! A unified download management interface for the BurnCloud platform.
//!
//! ## Features
//!
//! - Unified download management interface through `DownloadManager` trait
//! - Task queue management with configurable concurrency limits
//! - Multiple download manager implementations (Basic, TaskQueue)
//! - Support for multiple download backends (qBittorrent, Alist, aria2, etc.)
//! - Async/await architecture built on tokio
//! - Progress monitoring and event notifications
//! - Task lifecycle management (add, pause, resume, cancel)
//!
//! ## Available Implementations
//!
//! ### BasicDownloadManager
//! A simple download manager implementation with mock functionality for demonstration and testing.
//!
//! ### TaskQueueManager
//! A sophisticated queue-based download manager with concurrency control and event handling.
//!
//! ## Usage
//!
//! ### Using BasicDownloadManager
//!
//! ```rust,no_run
//! use burncloud_download::{DownloadManager, BasicDownloadManager};
//! use std::path::PathBuf;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let manager: Arc<dyn DownloadManager> = Arc::new(BasicDownloadManager::new());
//!
//!     // Add a download task
//!     let task_id = manager.add_download(
//!         "https://example.com/file.zip".to_string(),
//!         PathBuf::from("/downloads/file.zip")
//!     ).await?;
//!
//!     // Monitor progress
//!     let progress = manager.get_progress(task_id).await?;
//!     println!("Downloaded: {} / {} bytes",
//!         progress.downloaded_bytes,
//!         progress.total_bytes.unwrap_or(0)
//!     );
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Using TaskQueueManager
//!
//! ```rust,no_run
//! use burncloud_download::{DownloadManager, TaskQueueManager};
//! use std::path::PathBuf;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let manager: Arc<dyn DownloadManager> = Arc::new(TaskQueueManager::new());
//!
//!     // Add a download task
//!     let task_id = manager.add_download(
//!         "https://example.com/file.zip".to_string(),
//!         PathBuf::from("/downloads/file.zip")
//!     ).await?;
//!
//!     // Monitor progress
//!     let task = manager.get_task(task_id).await?;
//!     println!("Task status: {}", task.status);
//!
//!     Ok(())
//! }
//! ```

pub mod types;
pub mod traits;
pub mod queue;
pub mod manager;
pub mod error;
pub mod utils;

// Re-export core types and traits
pub use types::{DownloadTask, DownloadProgress, DownloadStatus, TaskId};
pub use traits::{DownloadManager, DownloadEventHandler};
pub use queue::TaskQueueManager;
pub use manager::BasicDownloadManager;
pub use error::DownloadError;

/// Result type alias for download operations
pub type Result<T> = std::result::Result<T, anyhow::Error>;