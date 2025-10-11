//! File identifier for duplicate detection
//!
//! Provides composite key for identifying duplicate downloads based on
//! normalized URL hash and target path.

use std::path::{Path, PathBuf};
use crate::utils::url_normalization::{process_url_for_storage};
use blake3;

/// Composite key for identifying duplicate downloads
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileIdentifier {
    pub url_hash: String,
    pub target_path: PathBuf,
    pub file_size: Option<u64>,
}

impl FileIdentifier {
    /// Create new FileIdentifier with normalized URL hash
    pub fn new(url: &str, target_path: &Path, file_size: Option<u64>) -> Self {
        let (_normalized_url, url_hash) = process_url_for_storage(url)
            .unwrap_or_else(|_| {
                // Fallback to using original URL if normalization fails
                let fallback_hash = blake3::hash(url.as_bytes()).to_hex().to_string();
                (url.to_string(), fallback_hash)
            });

        Self {
            url_hash,
            target_path: target_path.to_path_buf(),
            file_size,
        }
    }

    /// Check if this identifier matches a download task
    pub fn matches_task<T>(&self, task: &T) -> bool
    where
        T: HasUrlHashAndPath,
    {
        self.url_hash == task.url_hash() && self.target_path == task.target_path()
    }
}

/// Trait for types that have url_hash and target_path for duplicate detection
pub trait HasUrlHashAndPath {
    fn url_hash(&self) -> &str;
    fn target_path(&self) -> &Path;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementation for testing
    struct MockDownloadTask {
        pub url_hash: String,
        pub target_path: PathBuf,
    }

    impl HasUrlHashAndPath for MockDownloadTask {
        fn url_hash(&self) -> &str {
            &self.url_hash
        }

        fn target_path(&self) -> &Path {
            &self.target_path
        }
    }

    #[test]
    fn test_file_identifier_new() {
        let identifier = FileIdentifier::new(
            "https://example.com/file.zip",
            Path::new("/downloads/file.zip"),
            Some(1024),
        );

        // Should normalize URL and generate hash
        assert!(!identifier.url_hash.is_empty());
        assert_eq!(identifier.target_path, Path::new("/downloads/file.zip"));
        assert_eq!(identifier.file_size, Some(1024));
    }

    #[test]
    fn test_url_normalization() {
        let id1 = FileIdentifier::new(
            "https://example.com/file.zip",
            Path::new("/downloads/file.zip"),
            None,
        );
        let id2 = FileIdentifier::new(
            "https://example.com/file.zip?param=value",
            Path::new("/downloads/file.zip"),
            None,
        );

        // URLs should normalize differently when query params are present
        assert!(!id1.url_hash.is_empty());
        assert!(!id2.url_hash.is_empty());
        // They should be different because query params are included after normalization
        assert_ne!(id1.url_hash, id2.url_hash);
    }

    #[test]
    fn test_matches_task() {
        let identifier = FileIdentifier::new(
            "https://example.com/test.zip",
            Path::new("/downloads/test.zip"),
            Some(2048),
        );

        // Mock task with matching fields
        let task = MockDownloadTask {
            url_hash: identifier.url_hash.clone(),
            target_path: identifier.target_path.clone(),
        };

        assert!(identifier.matches_task(&task));
    }

    #[test]
    fn test_hash_consistency() {
        let id1 = FileIdentifier::new(
            "https://example.com/consistent.zip",
            Path::new("/path/consistent.zip"),
            Some(512),
        );
        let id2 = FileIdentifier::new(
            "https://example.com/consistent.zip",
            Path::new("/path/consistent.zip"),
            Some(512),
        );

        // Same inputs should produce same hash
        assert_eq!(id1.url_hash, id2.url_hash);
    }
}