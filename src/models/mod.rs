//! Data models for duplicate detection
//!
//! This module contains the data structures used for identifying and managing
//! duplicate downloads in the burncloud-download system.

pub mod file_identifier;
pub mod task_status;
pub mod duplicate_policy;
pub mod duplicate_result;
pub mod duplicate_reason;

pub use file_identifier::FileIdentifier;
pub use task_status::TaskStatus;
pub use duplicate_policy::DuplicatePolicy;
pub use duplicate_result::{DuplicateResult, DuplicateAction};
pub use duplicate_reason::DuplicateReason;