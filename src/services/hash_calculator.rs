//! Background hash calculator service
//!
//! Calculates file hashes in the background after download completion.

use crate::types::TaskId;
use crate::error::DownloadError;
use std::path::Path;
use async_trait::async_trait;

/// Service for calculating file hashes in the background
#[async_trait]
pub trait HashCalculator: Send + Sync {
    /// Queue a file for hash calculation
    async fn queue_calculation(&self, task_id: TaskId, file_path: &Path) -> Result<(), DownloadError>;

    /// Calculate hash immediately (blocking)
    async fn calculate_hash(&self, file_path: &Path) -> Result<String, DownloadError>;
}

/// Background hash calculator implementation
pub struct BackgroundHashCalculator {
    // Background task queue will be added when implemented
}

impl BackgroundHashCalculator {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl HashCalculator for BackgroundHashCalculator {
    async fn queue_calculation(&self, _task_id: TaskId, _file_path: &Path) -> Result<(), DownloadError> {
        // Placeholder implementation - will be implemented in Phase 6
        Ok(())
    }

    async fn calculate_hash(&self, file_path: &Path) -> Result<String, DownloadError> {
        // Placeholder implementation that calculates hash immediately
        use std::io::Read;

        let mut file = std::fs::File::open(file_path)
            .map_err(|e| DownloadError::IoError(e))?;

        let mut hasher = blake3::Hasher::new();
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)
                .map_err(|e| DownloadError::IoError(e))?;

            if bytes_read == 0 {
                break;
            }

            hasher.update(&buffer[..bytes_read]);
        }

        Ok(hasher.finalize().to_hex().to_string())
    }
}