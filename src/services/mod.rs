//! Services for duplicate detection
//!
//! This module contains the core services that implement duplicate detection
//! logic and coordinate with the download manager.

pub mod duplicate_detector;
pub mod task_repository;
pub mod hash_calculator;
pub mod task_validation;

pub use duplicate_detector::DuplicateDetector;
pub use task_repository::TaskRepository;
pub use hash_calculator::BackgroundHashCalculator;
pub use task_validation::TaskValidation;