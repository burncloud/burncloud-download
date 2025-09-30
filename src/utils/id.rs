use crate::types::TaskId;

/// Utility functions for TaskId generation and management
impl TaskId {
    /// Generate a new random TaskId
    pub fn generate() -> Self {
        Self::new()
    }

    /// Parse TaskId from string
    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        let uuid = uuid::Uuid::parse_str(s)?;
        Ok(Self(uuid))
    }

    /// Convert TaskId to string
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_id_generate() {
        let id1 = TaskId::generate();
        let id2 = TaskId::generate();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_task_id_from_string() {
        let original_id = TaskId::new();
        let id_string = original_id.to_string();
        let parsed_id = TaskId::from_string(&id_string).unwrap();
        assert_eq!(original_id, parsed_id);
    }

    #[test]
    fn test_task_id_from_invalid_string() {
        let result = TaskId::from_string("invalid-uuid");
        assert!(result.is_err());
    }

    #[test]
    fn test_task_id_to_string() {
        let id = TaskId::new();
        let id_string = id.to_string();
        assert!(!id_string.is_empty());
        assert!(uuid::Uuid::parse_str(&id_string).is_ok());
    }

    #[test]
    fn test_task_id_round_trip() {
        let original_id = TaskId::new();
        let id_string = original_id.to_string();
        let parsed_id = TaskId::from_string(&id_string).unwrap();
        assert_eq!(original_id, parsed_id);
        assert_eq!(original_id.to_string(), parsed_id.to_string());
    }
}