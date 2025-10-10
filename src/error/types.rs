use thiserror::Error;
use crate::types::TaskId;

/// Download manager error types
#[derive(Error, Debug)]
pub enum DownloadError {
    #[error("Task with ID {0} not found")]
    TaskNotFound(TaskId),

    #[error("Invalid task status transition")]
    InvalidStatusTransition,

    #[error("Maximum concurrent downloads exceeded")]
    ConcurrencyLimitExceeded,

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Invalid target path: {0}")]
    InvalidPath(String),

    #[error("Downloader not available: {0}")]
    DownloaderUnavailable(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("General error: {0}")]
    General(String),

    // New errors for duplicate detection
    #[error("Duplicate detection failed: {0}")]
    DuplicateDetectionError(String),

    #[error("Task verification failed: {0}")]
    VerificationError(String),

    #[error("Policy violation: {reason}, found duplicate task {task_id}")]
    PolicyViolation { task_id: TaskId, reason: String },
}