use serde::{Deserialize, Serialize};

/// Download progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    /// Number of bytes downloaded
    pub downloaded_bytes: u64,
    /// Total file size in bytes (None if unknown)
    pub total_bytes: Option<u64>,
    /// Current download speed in bytes per second
    pub speed_bps: u64,
    /// Estimated time to completion in seconds (None if unknown)
    pub eta_seconds: Option<u64>,
}

impl DownloadProgress {
    pub fn new() -> Self {
        Self {
            downloaded_bytes: 0,
            total_bytes: None,
            speed_bps: 0,
            eta_seconds: None,
        }
    }

    /// Calculate completion percentage (0-100)
    pub fn completion_percentage(&self) -> Option<f64> {
        self.total_bytes.map(|total| {
            if total == 0 {
                100.0
            } else {
                (self.downloaded_bytes as f64 / total as f64) * 100.0
            }
        })
    }
}

impl Default for DownloadProgress {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_progress_new() {
        let progress = DownloadProgress::new();
        assert_eq!(progress.downloaded_bytes, 0);
        assert_eq!(progress.total_bytes, None);
        assert_eq!(progress.speed_bps, 0);
        assert_eq!(progress.eta_seconds, None);
    }

    #[test]
    fn test_download_progress_default() {
        let progress = DownloadProgress::default();
        assert_eq!(progress.downloaded_bytes, 0);
        assert_eq!(progress.total_bytes, None);
        assert_eq!(progress.speed_bps, 0);
        assert_eq!(progress.eta_seconds, None);
    }

    #[test]
    fn test_completion_percentage() {
        let mut progress = DownloadProgress::new();
        progress.downloaded_bytes = 50;
        progress.total_bytes = Some(100);

        assert_eq!(progress.completion_percentage(), Some(50.0));
    }

    #[test]
    fn test_completion_percentage_no_total() {
        let progress = DownloadProgress::new();
        assert_eq!(progress.completion_percentage(), None);
    }

    #[test]
    fn test_completion_percentage_zero_total() {
        let mut progress = DownloadProgress::new();
        progress.total_bytes = Some(0);

        assert_eq!(progress.completion_percentage(), Some(100.0));
    }

    #[test]
    fn test_completion_percentage_complete() {
        let mut progress = DownloadProgress::new();
        progress.downloaded_bytes = 1000;
        progress.total_bytes = Some(1000);

        assert_eq!(progress.completion_percentage(), Some(100.0));
    }
}