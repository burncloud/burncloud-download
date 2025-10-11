//! Reasons for duplicate detection
//!
//! Provides structured information about why a download was considered a duplicate.

use serde::{Deserialize, Serialize};

/// Reason why a download was identified as a duplicate
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DuplicateReason {
    /// Exact match - same URL hash and target path
    ExactMatch,
    /// Same URL and target path (legacy variant)
    UrlAndPath,
    /// Same file content hash
    FileContent,
    /// Similar URL after normalization
    SimilarUrl,
    /// Same filename in target directory
    Filename,
    /// Policy-based allowance (e.g., re-downloading completed files)
    PolicyAllowed,
}

impl DuplicateReason {
    /// Get human-readable description of the duplicate reason
    pub fn description(&self) -> &'static str {
        match self {
            DuplicateReason::ExactMatch => "Exact match - same URL hash and target path",
            DuplicateReason::UrlAndPath => "Same URL and target path",
            DuplicateReason::FileContent => "Same file content (hash match)",
            DuplicateReason::SimilarUrl => "Similar URL after normalization",
            DuplicateReason::Filename => "Same filename in target directory",
            DuplicateReason::PolicyAllowed => "Policy allows duplicate operation",
        }
    }

    /// Get the priority of this duplicate reason (lower number = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            DuplicateReason::ExactMatch => 0,         // Highest priority - exact hash match
            DuplicateReason::UrlAndPath => 1,         // High priority - exact URL/path match
            DuplicateReason::FileContent => 2,        // High priority - content match
            DuplicateReason::SimilarUrl => 3,         // Medium priority - URL similarity
            DuplicateReason::Filename => 4,          // Low priority - filename only
            DuplicateReason::PolicyAllowed => 5,     // Lowest priority - policy decision
        }
    }

    /// Check if this reason indicates a strong duplicate match
    pub fn is_strong_match(&self) -> bool {
        matches!(self,
            DuplicateReason::ExactMatch |
            DuplicateReason::UrlAndPath |
            DuplicateReason::FileContent
        )
    }
}

impl std::fmt::Display for DuplicateReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}