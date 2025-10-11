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