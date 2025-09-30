pub mod task;
pub mod progress;
pub mod status;

pub use task::{DownloadTask, TaskId};
pub use progress::DownloadProgress;
pub use status::DownloadStatus;