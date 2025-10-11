//! Result types for duplicate detection operations
//!
//! Provides structured results for duplicate detection and policy application.

use crate::types::TaskId;
use crate::models::{TaskStatus, DuplicateReason};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Result of duplicate detection and policy application
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DuplicateResult {
    /// No duplicate found - new task should be created
    NotFound {
        url_hash: String,
        target_path: PathBuf,
    },
    /// Duplicate found - existing task should be reused
    Found {
        task_id: TaskId,
        reason: DuplicateReason,
        status: TaskStatus,
    },
    /// New task was created (legacy variant)
    NewTask(TaskId),
    /// Existing task was found and will be reused (legacy variant)
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
            DuplicateResult::NotFound { .. } => None,
            DuplicateResult::Found { task_id, .. } => Some(*task_id),
            DuplicateResult::NewTask(id) => Some(*id),
            DuplicateResult::ExistingTask { task_id, .. } => Some(*task_id),
            DuplicateResult::RequiresDecision { .. } => None,
        }
    }

    /// Check if this result represents no duplicate found
    pub fn is_not_found(&self) -> bool {
        matches!(self, DuplicateResult::NotFound { .. })
    }

    /// Check if this result represents a found duplicate
    pub fn is_found(&self) -> bool {
        matches!(self, DuplicateResult::Found { .. })
    }

    /// Check if this result represents a new task (legacy)
    pub fn is_new_task(&self) -> bool {
        matches!(self, DuplicateResult::NewTask(_))
    }

    /// Check if this result represents an existing task (legacy)
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