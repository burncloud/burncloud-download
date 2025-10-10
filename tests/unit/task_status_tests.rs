//! Unit tests for TaskStatus enum extensions
//!
//! These tests MUST FAIL FIRST following TDD methodology required by the project constitution.

#[cfg(test)]
mod tests {
    use burncloud_download::{TaskStatus, TaskId};

    #[test]
    fn test_duplicate_status_creation() {
        // This test will fail until Duplicate variant is added to TaskStatus
        let original_task_id = TaskId::new();
        let status = TaskStatus::Duplicate(original_task_id);

        match status {
            TaskStatus::Duplicate(id) => {
                assert_eq!(id, original_task_id);
            }
            _ => panic!("Expected Duplicate variant"),
        }
    }

    #[test]
    fn test_task_status_transitions() {
        // This test will fail until valid transitions are implemented
        let original_task_id = TaskId::new();

        // Test valid transitions to Duplicate
        assert!(TaskStatus::Waiting.can_transition_to_duplicate());
        assert!(TaskStatus::Paused.can_transition_to_duplicate());
        assert!(TaskStatus::Failed("error".to_string()).can_transition_to_duplicate());

        // Test invalid transitions to Duplicate
        assert!(!TaskStatus::Downloading.can_transition_to_duplicate());
        assert!(!TaskStatus::Completed.can_transition_to_duplicate());
        assert!(!TaskStatus::Duplicate(original_task_id).can_transition_to_duplicate());
    }

    #[test]
    fn test_duplicate_status_serialization() {
        // This test will fail until serialization is implemented
        let original_task_id = TaskId::new();
        let status = TaskStatus::Duplicate(original_task_id);

        let serialized = serde_json::to_string(&status).expect("Should serialize");
        let deserialized: TaskStatus = serde_json::from_str(&serialized).expect("Should deserialize");

        match deserialized {
            TaskStatus::Duplicate(id) => {
                assert_eq!(id, original_task_id);
            }
            _ => panic!("Expected Duplicate variant after deserialization"),
        }
    }
}