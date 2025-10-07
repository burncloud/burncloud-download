pub mod task;
pub mod progress;
pub mod status;

// Re-export types from burncloud-download-types for backwards compatibility
pub use burncloud_download_types::{DownloadTask, TaskId, DownloadProgress, DownloadStatus};