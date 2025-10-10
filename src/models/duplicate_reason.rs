//! Reasons for duplicate detection
//!
//! Provides structured information about why a download was considered a duplicate.

use serde::{Deserialize, Serialize};

/// Reason why a download was identified as a duplicate
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DuplicateReason {
    /// Same URL and target path
    UrlAndPath,
    /// Same file content hash
    FileContent,
    /// Similar URL after normalization
    SimilarUrl,
    /// Same filename in target directory
    Filename,
}

impl DuplicateReason {
    /// Get human-readable description of the duplicate reason
    pub fn description(&self) -> &'static str {
        match self {
            DuplicateReason::UrlAndPath => "Same URL and target path",
            DuplicateReason::FileContent => "Same file content (hash match)",
            DuplicateReason::SimilarUrl => "Similar URL after normalization",
            DuplicateReason::Filename => "Same filename in target directory",
        }
    }

    /// Get the priority of this duplicate reason (lower number = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            DuplicateReason::UrlAndPath => 1,      // Highest priority - exact match
            DuplicateReason::FileContent => 2,     // High priority - content match
            DuplicateReason::SimilarUrl => 3,      // Medium priority - URL similarity
            DuplicateReason::Filename => 4,       // Lowest priority - filename only
        }
    }

    /// Check if this reason indicates a strong duplicate match
    pub fn is_strong_match(&self) -> bool {
        matches!(self, DuplicateReason::UrlAndPath | DuplicateReason::FileContent)
    }
}

impl std::fmt::Display for DuplicateReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_reason_variants() {
        let reasons = vec![
            DuplicateReason::UrlAndPath,
            DuplicateReason::FileContent,
            DuplicateReason::SimilarUrl,
            DuplicateReason::Filename,
        ];

        assert_eq!(reasons.len(), 4);

        // Each should be different
        for (i, reason1) in reasons.iter().enumerate() {
            for (j, reason2) in reasons.iter().enumerate() {
                if i != j {
                    assert_ne!(reason1, reason2);
                }
            }
        }
    }

    #[test]
    fn test_duplicate_reason_descriptions() {
        assert_eq!(DuplicateReason::UrlAndPath.description(), "Same URL and target path");
        assert_eq!(DuplicateReason::FileContent.description(), "Same file content (hash match)");
        assert_eq!(DuplicateReason::SimilarUrl.description(), "Similar URL after normalization");
        assert_eq!(DuplicateReason::Filename.description(), "Same filename in target directory");
    }

    #[test]
    fn test_duplicate_reason_priority() {
        assert_eq!(DuplicateReason::UrlAndPath.priority(), 1);
        assert_eq!(DuplicateReason::FileContent.priority(), 2);
        assert_eq!(DuplicateReason::SimilarUrl.priority(), 3);
        assert_eq!(DuplicateReason::Filename.priority(), 4);

        // Verify priority ordering
        assert!(DuplicateReason::UrlAndPath.priority() < DuplicateReason::FileContent.priority());
        assert!(DuplicateReason::FileContent.priority() < DuplicateReason::SimilarUrl.priority());
        assert!(DuplicateReason::SimilarUrl.priority() < DuplicateReason::Filename.priority());
    }

    #[test]
    fn test_is_strong_match() {
        assert!(DuplicateReason::UrlAndPath.is_strong_match());
        assert!(DuplicateReason::FileContent.is_strong_match());
        assert!(!DuplicateReason::SimilarUrl.is_strong_match());
        assert!(!DuplicateReason::Filename.is_strong_match());
    }

    #[test]
    fn test_display_format() {
        let reason = DuplicateReason::UrlAndPath;
        let formatted = format!("{}", reason);
        assert_eq!(formatted, "Same URL and target path");
    }

    #[test]
    fn test_serialization() {
        let reason = DuplicateReason::FileContent;

        let serialized = serde_json::to_string(&reason).expect("Should serialize");
        let deserialized: DuplicateReason = serde_json::from_str(&serialized).expect("Should deserialize");

        assert_eq!(reason, deserialized);
    }

    #[test]
    fn test_clone_and_debug() {
        let reason = DuplicateReason::SimilarUrl;
        let cloned = reason.clone();

        assert_eq!(reason, cloned);

        let debug_str = format!("{:?}", reason);
        assert!(debug_str.contains("SimilarUrl"));
    }
}