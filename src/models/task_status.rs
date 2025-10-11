//! Extended task status with duplicate detection support
//!
//! Provides additional status variants for duplicate detection while maintaining
//! compatibility with existing DownloadStatus.

use crate::types::TaskId;
use crate::utils::url_normalization::is_valid_url_hash;
use serde::{Deserialize, Serialize};

/// Extended task status that includes duplicate detection states
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is waiting to start
    Waiting,
    /// Task is currently downloading
    Downloading,
    /// Task has been paused
    Paused,
    /// Task completed successfully
    Completed,
    /// Task failed with error message
    Failed(String),
    /// Task is a duplicate of another task
    Duplicate(TaskId),
}

impl TaskStatus {
    /// Check if this status can transition to Duplicate
    pub fn can_transition_to_duplicate(&self) -> bool {
        matches!(self,
            TaskStatus::Waiting |
            TaskStatus::Paused |
            TaskStatus::Failed(_)
        )
    }

    /// Convert to base DownloadStatus for compatibility
    pub fn to_download_status(&self) -> crate::types::DownloadStatus {
        match self {
            TaskStatus::Waiting => crate::types::DownloadStatus::Waiting,
            TaskStatus::Downloading => crate::types::DownloadStatus::Downloading,
            TaskStatus::Paused => crate::types::DownloadStatus::Paused,
            TaskStatus::Completed => crate::types::DownloadStatus::Completed,
            TaskStatus::Failed(msg) => crate::types::DownloadStatus::Failed(msg.clone()),
            TaskStatus::Duplicate(_) => {
                // Duplicate status maps to a special case - we'll treat it as completed
                // since the original task provides the actual download
                crate::types::DownloadStatus::Completed
            }
        }
    }

    /// Create from base DownloadStatus
    pub fn from_download_status(status: crate::types::DownloadStatus) -> Self {
        match status {
            crate::types::DownloadStatus::Waiting => TaskStatus::Waiting,
            crate::types::DownloadStatus::Downloading => TaskStatus::Downloading,
            crate::types::DownloadStatus::Paused => TaskStatus::Paused,
            crate::types::DownloadStatus::Completed => TaskStatus::Completed,
            crate::types::DownloadStatus::Failed(msg) => TaskStatus::Failed(msg),
        }
    }
}

/// Validation utilities for task-related data
pub struct TaskValidator;

impl TaskValidator {
    /// Validate URL hash format for database storage
    pub fn validate_url_hash(url_hash: &str) -> Result<(), TaskValidationError> {
        if !is_valid_url_hash(url_hash) {
            return Err(TaskValidationError::InvalidUrlHash {
                hash: url_hash.to_string(),
                expected_format: "64-character Blake3 hex string".to_string(),
            });
        }
        Ok(())
    }

    /// Validate that task ID is not empty/default
    pub fn validate_task_id(task_id: &TaskId) -> Result<(), TaskValidationError> {
        // Basic validation - ensure task ID is not the default/empty value
        if task_id.to_string().is_empty() {
            return Err(TaskValidationError::InvalidTaskId {
                reason: "Task ID cannot be empty".to_string(),
            });
        }
        Ok(())
    }

    /// Validate task status transition
    pub fn validate_status_transition(
        from: &TaskStatus,
        to: &TaskStatus,
    ) -> Result<(), TaskValidationError> {
        match (from, to) {
            // Valid transitions to Duplicate
            (TaskStatus::Waiting, TaskStatus::Duplicate(_)) => Ok(()),
            (TaskStatus::Paused, TaskStatus::Duplicate(_)) => Ok(()),
            (TaskStatus::Failed(_), TaskStatus::Duplicate(_)) => Ok(()),

            // Invalid transitions to Duplicate
            (TaskStatus::Downloading, TaskStatus::Duplicate(_)) => {
                Err(TaskValidationError::InvalidStatusTransition {
                    from: format!("{:?}", from),
                    to: format!("{:?}", to),
                    reason: "Cannot mark downloading task as duplicate".to_string(),
                })
            }
            (TaskStatus::Completed, TaskStatus::Duplicate(_)) => {
                Err(TaskValidationError::InvalidStatusTransition {
                    from: format!("{:?}", from),
                    to: format!("{:?}", to),
                    reason: "Cannot mark completed task as duplicate".to_string(),
                })
            }
            (TaskStatus::Duplicate(_), TaskStatus::Duplicate(_)) => {
                Err(TaskValidationError::InvalidStatusTransition {
                    from: format!("{:?}", from),
                    to: format!("{:?}", to),
                    reason: "Task is already marked as duplicate".to_string(),
                })
            }

            // All other transitions are allowed by default
            _ => Ok(()),
        }
    }
}

/// Validation errors for task-related operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskValidationError {
    InvalidUrlHash {
        hash: String,
        expected_format: String,
    },
    InvalidTaskId {
        reason: String,
    },
    InvalidStatusTransition {
        from: String,
        to: String,
        reason: String,
    },
}

impl std::fmt::Display for TaskValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskValidationError::InvalidUrlHash { hash, expected_format } => {
                write!(f, "Invalid URL hash '{}': expected {}", hash, expected_format)
            }
            TaskValidationError::InvalidTaskId { reason } => {
                write!(f, "Invalid task ID: {}", reason)
            }
            TaskValidationError::InvalidStatusTransition { from, to, reason } => {
                write!(f, "Invalid status transition from {} to {}: {}", from, to, reason)
            }
        }
    }
}

impl std::error::Error for TaskValidationError {}