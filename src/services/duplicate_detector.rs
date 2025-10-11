//! Duplicate detection service
//!
//! Core service for detecting duplicate downloads and applying policies.

use crate::types::TaskId;
use crate::models::{DuplicatePolicy, DuplicateResult};
use crate::utils::url_normalization::{process_url_for_storage};
use crate::error::DownloadError;
use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;

/// Service for detecting duplicate downloads
#[async_trait]
pub trait DuplicateDetector: Send + Sync {
    /// Find duplicate download task for the given URL and target path
    async fn find_duplicate(
        &self,
        url: &str,
        target_path: &Path,
    ) -> Result<DuplicateResult>;

    /// Find all tasks with the same URL hash
    async fn find_by_url_hash(
        &self,
        url_hash: &str,
    ) -> Result<Vec<MockDownloadTask>>;

    /// Legacy method for compatibility
    async fn find_by_url_and_path(
        &self,
        url: &str,
        target_path: &Path,
    ) -> Result<Option<TaskId>, DownloadError>;

    /// Apply duplicate policy and return appropriate result
    async fn apply_policy(
        &self,
        url: &str,
        target_path: &Path,
        policy: DuplicatePolicy,
    ) -> Result<DuplicateResult, DownloadError>;

    /// Get all potential duplicate candidates
    async fn get_candidates(
        &self,
        url: &str,
        target_path: &Path,
    ) -> Result<Vec<TaskId>, DownloadError>;
}

/// Default implementation of DuplicateDetector
pub struct DefaultDuplicateDetector {
    // Repository dependencies will be added when implemented
}

impl Default for DefaultDuplicateDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultDuplicateDetector {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl DuplicateDetector for DefaultDuplicateDetector {
    async fn find_duplicate(
        &self,
        url: &str,
        target_path: &Path,
    ) -> Result<DuplicateResult> {
        // Implementation for TDD - this will be fully implemented
        let (_normalized_url, url_hash) = process_url_for_storage(url)?;

        // TODO: Query database for existing task with same url_hash and target_path
        // For now, return NotFound to make tests compile
        Ok(DuplicateResult::NotFound {
            url_hash,
            target_path: target_path.to_path_buf(),
        })
    }

    async fn find_by_url_hash(
        &self,
        _url_hash: &str,
    ) -> Result<Vec<MockDownloadTask>> {
        // Implementation for TDD - this will query database by URL hash
        Ok(Vec::new())
    }

    async fn find_by_url_and_path(
        &self,
        _url: &str,
        _target_path: &Path,
    ) -> Result<Option<TaskId>, DownloadError> {
        // Placeholder implementation - will be implemented in Phase 3
        Ok(None)
    }

    async fn apply_policy(
        &self,
        _url: &str,
        _target_path: &Path,
        policy: DuplicatePolicy,
    ) -> Result<DuplicateResult, DownloadError> {
        // Placeholder implementation - will be implemented in Phase 3
        match policy {
            DuplicatePolicy::AllowDuplicate => {
                let (_normalized_url, url_hash) = process_url_for_storage(_url)
                    .map_err(|e| DownloadError::InvalidUrl(e.to_string()))?;
                Ok(DuplicateResult::NotFound {
                    url_hash,
                    target_path: _target_path.to_path_buf(),
                })
            }
            _ => {
                let (_normalized_url, url_hash) = process_url_for_storage(_url)
                    .map_err(|e| DownloadError::InvalidUrl(e.to_string()))?;
                Ok(DuplicateResult::NotFound {
                    url_hash,
                    target_path: _target_path.to_path_buf(),
                })
            }
        }
    }

    async fn get_candidates(
        &self,
        _url: &str,
        _target_path: &Path,
    ) -> Result<Vec<TaskId>, DownloadError> {
        // Placeholder implementation - will be implemented in Phase 3
        Ok(vec![])
    }
}

// Mock types for testing
#[derive(Debug, Clone)]
pub struct MockDownloadTask {
    pub id: TaskId,
    pub url: String,
    pub url_hash: String,
    pub target_path: std::path::PathBuf,
    pub status: crate::types::DownloadStatus,
}