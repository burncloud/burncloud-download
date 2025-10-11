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

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn test_duplicate_policy_default() {
        let policy: DuplicatePolicy = Default::default();
        assert_eq!(policy, DuplicatePolicy::ReuseExisting);
    }

    #[test]
    fn test_duplicate_policy_variants() {
        let policies = vec![
            DuplicatePolicy::ReuseExisting,
            DuplicatePolicy::AllowDuplicate,
            DuplicatePolicy::PromptUser,
            DuplicatePolicy::ReuseIfComplete,
            DuplicatePolicy::ReuseIfIncomplete,
            DuplicatePolicy::FailIfDuplicate,
        ];

        // Should have 6 different policy types
        assert_eq!(policies.len(), 6);

        // Each should be different
        for (i, policy1) in policies.iter().enumerate() {
            for (j, policy2) in policies.iter().enumerate() {
                if i != j {
                    assert_ne!(policy1, policy2);
                }
            }
        }
    }

    #[test]
    fn test_duplicate_policy_serialization() {
        let policy = DuplicatePolicy::ReuseIfComplete;

        let serialized = serde_json::to_string(&policy).expect("Should serialize");
        let deserialized: DuplicatePolicy = serde_json::from_str(&serialized).expect("Should deserialize");

        assert_eq!(policy, deserialized);
    }

    #[test]
    fn test_duplicate_policy_clone() {
        let policy = DuplicatePolicy::AllowDuplicate;
        let cloned = policy.clone();

        assert_eq!(policy, cloned);
    }

    #[test]
    fn test_duplicate_policy_debug() {
        let policy = DuplicatePolicy::PromptUser;
        let debug_str = format!("{:?}", policy);

        assert!(debug_str.contains("PromptUser"));
    }

    #[test]
    fn test_allows_reuse_logic() {
        use crate::models::TaskStatus;

        let completed_status = TaskStatus::Completed;
        let waiting_status = TaskStatus::Waiting;
        let failed_status = TaskStatus::Failed("error".to_string());

        // ReuseExisting should allow all
        assert!(DuplicatePolicy::ReuseExisting.allows_reuse(&completed_status));
        assert!(DuplicatePolicy::ReuseExisting.allows_reuse(&waiting_status));

        // ReuseIfComplete should only allow completed
        assert!(DuplicatePolicy::ReuseIfComplete.allows_reuse(&completed_status));
        assert!(!DuplicatePolicy::ReuseIfComplete.allows_reuse(&waiting_status));

        // ReuseIfIncomplete should only allow incomplete
        assert!(!DuplicatePolicy::ReuseIfIncomplete.allows_reuse(&completed_status));
        assert!(DuplicatePolicy::ReuseIfIncomplete.allows_reuse(&waiting_status));
        assert!(DuplicatePolicy::ReuseIfIncomplete.allows_reuse(&failed_status));

        // AllowDuplicate should never allow reuse
        assert!(!DuplicatePolicy::AllowDuplicate.allows_reuse(&completed_status));
        assert!(!DuplicatePolicy::AllowDuplicate.allows_reuse(&waiting_status));
    }
}