//! Extended task status with duplicate detection support
//!
//! Provides additional status variants for duplicate detection while maintaining
//! compatibility with existing DownloadStatus.

use crate::types::TaskId;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_status_creation() {
        let original_task_id = TaskId::new();
        let status = TaskStatus::Duplicate(original_task_id);

        match status {
            TaskStatus::Duplicate(id) => {
                assert_eq!(id, original_task_id);
            }
            _ => panic!("Expected Duplicate variant"),
        }
    }

    #[test]
    fn test_task_status_transitions() {
        let original_task_id = TaskId::new();

        // Test valid transitions to Duplicate
        assert!(TaskStatus::Waiting.can_transition_to_duplicate());
        assert!(TaskStatus::Paused.can_transition_to_duplicate());
        assert!(TaskStatus::Failed("error".to_string()).can_transition_to_duplicate());

        // Test invalid transitions to Duplicate
        assert!(!TaskStatus::Downloading.can_transition_to_duplicate());
        assert!(!TaskStatus::Completed.can_transition_to_duplicate());
        assert!(!TaskStatus::Duplicate(original_task_id).can_transition_to_duplicate());
    }

    #[test]
    fn test_duplicate_status_serialization() {
        let original_task_id = TaskId::new();
        let status = TaskStatus::Duplicate(original_task_id);

        let serialized = serde_json::to_string(&status).expect("Should serialize");
        let deserialized: TaskStatus = serde_json::from_str(&serialized).expect("Should deserialize");

        match deserialized {
            TaskStatus::Duplicate(id) => {
                assert_eq!(id, original_task_id);
            }
            _ => panic!("Expected Duplicate variant after deserialization"),
        }
    }
}