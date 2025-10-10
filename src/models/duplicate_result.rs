//! Result types for duplicate detection operations
//!
//! Provides structured results for duplicate detection and policy application.

use crate::types::TaskId;
use crate::models::{TaskStatus, DuplicateReason};
use serde::{Deserialize, Serialize};

/// Result of duplicate detection and policy application
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DuplicateResult {
    /// New task was created
    NewTask(TaskId),
    /// Existing task was found and will be reused
    ExistingTask {
        task_id: TaskId,
        status: TaskStatus,
        reason: DuplicateReason,
    },
    /// User interaction required to decide
    RequiresDecision {
        candidates: Vec<TaskId>,
        suggested_action: DuplicateAction,
    },
}

/// Suggested action for duplicate resolution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DuplicateAction {
    /// Resume the specified task
    Resume(TaskId),
    /// Reuse the specified task (already completed)
    Reuse(TaskId),
    /// Retry the specified task (was failed)
    Retry(TaskId),
    /// Create a new task
    CreateNew,
}

impl DuplicateResult {
    /// Get the task ID from any result variant
    pub fn task_id(&self) -> Option<TaskId> {
        match self {
            DuplicateResult::NewTask(id) => Some(*id),
            DuplicateResult::ExistingTask { task_id, .. } => Some(*task_id),
            DuplicateResult::RequiresDecision { .. } => None,
        }
    }

    /// Check if this result represents a new task
    pub fn is_new_task(&self) -> bool {
        matches!(self, DuplicateResult::NewTask(_))
    }

    /// Check if this result represents an existing task
    pub fn is_existing_task(&self) -> bool {
        matches!(self, DuplicateResult::ExistingTask { .. })
    }

    /// Check if this result requires user decision
    pub fn requires_decision(&self) -> bool {
        matches!(self, DuplicateResult::RequiresDecision { .. })
    }
}

impl DuplicateAction {
    /// Get the task ID associated with this action, if any
    pub fn task_id(&self) -> Option<TaskId> {
        match self {
            DuplicateAction::Resume(id) => Some(*id),
            DuplicateAction::Reuse(id) => Some(*id),
            DuplicateAction::Retry(id) => Some(*id),
            DuplicateAction::CreateNew => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DuplicateReason;

    #[test]
    fn test_duplicate_result_new_task() {
        let task_id = TaskId::new();
        let result = DuplicateResult::NewTask(task_id);

        assert_eq!(result.task_id(), Some(task_id));
        assert!(result.is_new_task());
        assert!(!result.is_existing_task());
        assert!(!result.requires_decision());
    }

    #[test]
    fn test_duplicate_result_existing_task() {
        let task_id = TaskId::new();
        let result = DuplicateResult::ExistingTask {
            task_id,
            status: TaskStatus::Completed,
            reason: DuplicateReason::UrlAndPath,
        };

        assert_eq!(result.task_id(), Some(task_id));
        assert!(!result.is_new_task());
        assert!(result.is_existing_task());
        assert!(!result.requires_decision());
    }

    #[test]
    fn test_duplicate_result_requires_decision() {
        let task_id1 = TaskId::new();
        let task_id2 = TaskId::new();
        let result = DuplicateResult::RequiresDecision {
            candidates: vec![task_id1, task_id2],
            suggested_action: DuplicateAction::Resume(task_id1),
        };

        assert_eq!(result.task_id(), None);
        assert!(!result.is_new_task());
        assert!(!result.is_existing_task());
        assert!(result.requires_decision());
    }

    #[test]
    fn test_duplicate_action_task_ids() {
        let task_id = TaskId::new();

        assert_eq!(DuplicateAction::Resume(task_id).task_id(), Some(task_id));
        assert_eq!(DuplicateAction::Reuse(task_id).task_id(), Some(task_id));
        assert_eq!(DuplicateAction::Retry(task_id).task_id(), Some(task_id));
        assert_eq!(DuplicateAction::CreateNew.task_id(), None);
    }

    #[test]
    fn test_serialization() {
        let task_id = TaskId::new();
        let result = DuplicateResult::ExistingTask {
            task_id,
            status: TaskStatus::Waiting,
            reason: DuplicateReason::FileContent,
        };

        let serialized = serde_json::to_string(&result).expect("Should serialize");
        let deserialized: DuplicateResult = serde_json::from_str(&serialized).expect("Should deserialize");

        assert_eq!(result, deserialized);
    }
}