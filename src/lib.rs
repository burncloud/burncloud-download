//! # BurnCloud Download Manager
//!
//! A unified download management interface for the BurnCloud platform.
//!
//! ## Features
//!
//! - Simple download API with automatic aria2 integration
//! - Persistent downloads that survive application restarts
//! - Progress monitoring and task lifecycle management
//! - Default storage to `./data/` directory with customizable paths
//! - Automatic database persistence and recovery
//!
//! ## Simple Usage (Recommended)
//!
//! For most users, use the simple download functions that automatically handle everything:
//!
//! ```rust,no_run
//! use burncloud_download::{download, download_to, get_download_progress};
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Simple download to default ./data/ directory
//!     let task_id = download("https://example.com/file.zip").await?;
//!
//!     // Download to custom location
//!     let task_id2 = download_to(
//!         "https://example.com/document.pdf",
//!         "./downloads/document.pdf"
//!     ).await?;
//!
//!     // Monitor progress
//!     let progress = get_download_progress(task_id).await?;
//!     println!("Downloaded: {} bytes", progress.downloaded_bytes);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Advanced Usage
//!
//! For advanced users who need direct manager access:
//!
//! ### Using PersistentAria2Manager
//!
//! ```rust,no_run
//! use burncloud_download::{DownloadManager, PersistentAria2Manager};
//! use std::path::PathBuf;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let manager: Arc<dyn DownloadManager> = Arc::new(PersistentAria2Manager::new().await?);
//!
//!     // Add a download task
//!     let task_id = manager.add_download(
//!         "https://example.com/file.zip".to_string(),
//!         PathBuf::from("data/file.zip")
//!     ).await?;
//!
//!     // Download will persist across restarts
//!     println!("Download started with ID: {}", task_id);
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

// Re-export core types from burncloud-download-types
pub use burncloud_download_types::{DownloadTask, DownloadProgress, DownloadStatus, TaskId};

// Re-export traits and implementations
pub use traits::{DownloadManager, DownloadEventHandler};
pub use queue::TaskQueueManager;
pub use manager::{BasicDownloadManager, PersistentAria2Manager};

pub use error::DownloadError;

/// Result type alias for download operations
pub type Result<T> = std::result::Result<T, anyhow::Error>;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tokio::sync::Mutex;

// Global manager instance for convenience functions
static GLOBAL_MANAGER: OnceLock<Mutex<Option<std::sync::Arc<PersistentAria2Manager>>>> = OnceLock::new();

/// Get or initialize the global download manager
async fn get_global_manager() -> Result<std::sync::Arc<PersistentAria2Manager>> {
    let manager_lock = GLOBAL_MANAGER.get_or_init(|| Mutex::new(None));
    let mut manager_guard = manager_lock.lock().await;

    if manager_guard.is_none() {
        let new_manager = PersistentAria2Manager::new().await?;
        *manager_guard = Some(std::sync::Arc::new(new_manager));
    }

    Ok(manager_guard.as_ref().unwrap().clone())
}

/// Simple download function that downloads a file to the default ./data/ directory
///
/// The filename is automatically extracted from the URL.
///
/// # Arguments
/// * `url` - The URL to download from
///
/// # Returns
/// * `TaskId` - The unique identifier for this download task
///
/// # Example
/// ```no_run
/// use burncloud_download::download;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let task_id = download("https://example.com/file.zip").await?;
///     println!("Download started: {}", task_id);
///     Ok(())
/// }
/// ```
pub async fn download<S: AsRef<str>>(url: S) -> Result<TaskId> {
    let url_str = url.as_ref();

    // Extract filename from URL
    let filename = url_str
        .split('/')
        .last()
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or("download");

    let target_path = PathBuf::from("./data").join(filename);

    download_to(url_str, target_path).await
}

/// Download a file to a specific path
///
/// # Arguments
/// * `url` - The URL to download from
/// * `target_path` - Where to save the downloaded file
///
/// # Returns
/// * `TaskId` - The unique identifier for this download task
///
/// # Example
/// ```no_run
/// use burncloud_download::download_to;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let task_id = download_to(
///         "https://example.com/document.pdf",
///         "./downloads/document.pdf"
///     ).await?;
///     println!("Download started: {}", task_id);
///     Ok(())
/// }
/// ```
pub async fn download_to<S: AsRef<str>, P: AsRef<Path>>(url: S, target_path: P) -> Result<TaskId> {
    let manager = get_global_manager().await?;
    manager.add_download(
        url.as_ref().to_string(),
        target_path.as_ref().to_path_buf()
    ).await
}

/// Get the progress of a download task
///
/// # Arguments
/// * `task_id` - The unique identifier of the download task
///
/// # Returns
/// * `DownloadProgress` - Current progress information
pub async fn get_download_progress(task_id: TaskId) -> Result<DownloadProgress> {
    let manager = get_global_manager().await?;
    manager.get_progress(task_id).await
}

/// Get detailed information about a download task
///
/// # Arguments
/// * `task_id` - The unique identifier of the download task
///
/// # Returns
/// * `DownloadTask` - Complete task information including status
pub async fn get_download_task(task_id: TaskId) -> Result<DownloadTask> {
    let manager = get_global_manager().await?;
    manager.get_task(task_id).await
}

/// Pause a download task
///
/// # Arguments
/// * `task_id` - The unique identifier of the download task
pub async fn pause_download(task_id: TaskId) -> Result<()> {
    let manager = get_global_manager().await?;
    manager.pause_download(task_id).await
}

/// Resume a paused download task
///
/// # Arguments
/// * `task_id` - The unique identifier of the download task
pub async fn resume_download(task_id: TaskId) -> Result<()> {
    let manager = get_global_manager().await?;
    manager.resume_download(task_id).await
}

/// Cancel a download task
///
/// # Arguments
/// * `task_id` - The unique identifier of the download task
pub async fn cancel_download(task_id: TaskId) -> Result<()> {
    let manager = get_global_manager().await?;
    manager.cancel_download(task_id).await
}

/// List all download tasks
///
/// # Returns
/// * `Vec<DownloadTask>` - List of all download tasks
pub async fn list_downloads() -> Result<Vec<DownloadTask>> {
    let manager = get_global_manager().await?;
    manager.list_tasks().await
}

/// Get the number of currently active downloads
///
/// # Returns
/// * `usize` - Number of active download tasks
pub async fn active_download_count() -> Result<usize> {
    let manager = get_global_manager().await?;
    manager.active_download_count().await
}