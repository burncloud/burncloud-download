//! Duplicate handling policies
//!
//! Defines how the system should behave when duplicate downloads are detected.

use serde::{Deserialize, Serialize};

/// Policy for handling duplicate downloads
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DuplicatePolicy {
    /// Reuse existing task regardless of status (default)
    ReuseExisting,
    /// Always create new task, ignore duplicates
    AllowDuplicate,
    /// Ask user for decision when duplicates found
    PromptUser,
    /// Reuse only if original task is completed
    ReuseIfComplete,
    /// Reuse only if original task is incomplete (for resume)
    ReuseIfIncomplete,
    /// Fail with error if duplicate is found
    FailIfDuplicate,
}

impl Default for DuplicatePolicy {
    fn default() -> Self {
        Self::ReuseExisting
    }
}

impl DuplicatePolicy {
    /// Check if this policy allows reusing the given task status
    pub fn allows_reuse(&self, status: &crate::models::TaskStatus) -> bool {
        match self {
            DuplicatePolicy::ReuseExisting => true,
            DuplicatePolicy::AllowDuplicate => false,
            DuplicatePolicy::PromptUser => false, // Requires user decision
            DuplicatePolicy::ReuseIfComplete => {
                matches!(status, crate::models::TaskStatus::Completed)
            }
            DuplicatePolicy::ReuseIfIncomplete => {
                matches!(status,
                    crate::models::TaskStatus::Waiting |
                    crate::models::TaskStatus::Downloading |
                    crate::models::TaskStatus::Paused |
                    crate::models::TaskStatus::Failed(_)
                )
            }
            DuplicatePolicy::FailIfDuplicate => false,
        }
    }

    /// Check if this policy should fail when duplicates are found
    pub fn should_fail_on_duplicate(&self) -> bool {
        matches!(self, DuplicatePolicy::FailIfDuplicate)
    }

    /// Check if this policy requires user interaction
    pub fn requires_user_decision(&self) -> bool {
        matches!(self, DuplicatePolicy::PromptUser)
    }
}