use serde::{Deserialize, Serialize};

/// Download task status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadStatus {
    /// Task is queued and waiting to start
    Waiting,
    /// Task is actively downloading
    Downloading,
    /// Task has been paused by user
    Paused,
    /// Task completed successfully
    Completed,
    /// Task failed with error
    Failed(String),
}

impl DownloadStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, DownloadStatus::Downloading)
    }

    pub fn is_finished(&self) -> bool {
        matches!(self, DownloadStatus::Completed | DownloadStatus::Failed(_))
    }

    pub fn can_resume(&self) -> bool {
        matches!(self, DownloadStatus::Paused | DownloadStatus::Failed(_))
    }

    pub fn can_pause(&self) -> bool {
        matches!(self, DownloadStatus::Downloading | DownloadStatus::Waiting)
    }
}

impl std::fmt::Display for DownloadStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadStatus::Waiting => write!(f, "Waiting"),
            DownloadStatus::Downloading => write!(f, "Downloading"),
            DownloadStatus::Paused => write!(f, "Paused"),
            DownloadStatus::Completed => write!(f, "Completed"),
            DownloadStatus::Failed(reason) => write!(f, "Failed: {}", reason),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_status_is_active() {
        assert!(DownloadStatus::Downloading.is_active());
        assert!(!DownloadStatus::Waiting.is_active());
        assert!(!DownloadStatus::Paused.is_active());
        assert!(!DownloadStatus::Completed.is_active());
        assert!(!DownloadStatus::Failed("error".to_string()).is_active());
    }

    #[test]
    fn test_download_status_is_finished() {
        assert!(!DownloadStatus::Downloading.is_finished());
        assert!(!DownloadStatus::Waiting.is_finished());
        assert!(!DownloadStatus::Paused.is_finished());
        assert!(DownloadStatus::Completed.is_finished());
        assert!(DownloadStatus::Failed("error".to_string()).is_finished());
    }

    #[test]
    fn test_download_status_can_resume() {
        assert!(!DownloadStatus::Downloading.can_resume());
        assert!(!DownloadStatus::Waiting.can_resume());
        assert!(DownloadStatus::Paused.can_resume());
        assert!(!DownloadStatus::Completed.can_resume());
        assert!(DownloadStatus::Failed("error".to_string()).can_resume());
    }

    #[test]
    fn test_download_status_can_pause() {
        assert!(DownloadStatus::Downloading.can_pause());
        assert!(DownloadStatus::Waiting.can_pause());
        assert!(!DownloadStatus::Paused.can_pause());
        assert!(!DownloadStatus::Completed.can_pause());
        assert!(!DownloadStatus::Failed("error".to_string()).can_pause());
    }

    #[test]
    fn test_download_status_display() {
        assert_eq!(format!("{}", DownloadStatus::Waiting), "Waiting");
        assert_eq!(format!("{}", DownloadStatus::Downloading), "Downloading");
        assert_eq!(format!("{}", DownloadStatus::Paused), "Paused");
        assert_eq!(format!("{}", DownloadStatus::Completed), "Completed");
        assert_eq!(format!("{}", DownloadStatus::Failed("Connection lost".to_string())), "Failed: Connection lost");
    }
}