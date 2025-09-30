use crate::types::DownloadTask;

/// Task scheduling logic for download queue management
pub struct TaskScheduler;

impl TaskScheduler {
    /// Determine if a task should be scheduled based on current conditions
    pub fn should_schedule_task(_task: &DownloadTask, active_count: usize, max_concurrent: usize) -> bool {
        active_count < max_concurrent
    }

    /// Get priority score for a task (lower score = higher priority)
    /// Currently uses FIFO ordering, but can be extended for priority-based scheduling
    pub fn get_task_priority(_task: &DownloadTask) -> u32 {
        0 // FIFO scheduling - all tasks have same priority
    }
}