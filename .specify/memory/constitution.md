<!--
Sync Impact Report:
Version change: 1.0.0 → 1.1.0
Modified principles:
- "V. Testing Excellence" → "V. Test-Driven Development (NON-NEGOTIABLE)" - materially expanded to enforce TDD
Added sections: N/A
Removed sections: N/A
Templates requiring updates:
✅ Updated plan-template.md compatibility check
✅ Updated spec-template.md compatibility check
✅ Updated tasks-template.md - made tests mandatory, emphasized TDD requirements
Follow-up TODOs: None
-->

# BurnCloud Download Manager Constitution

## Core Principles

### I. Duplicate Detection and Task Reuse
**MUST** prevent duplicate downloads by detecting existing tasks for the same URL and target path combination. When a download request matches an existing task, the system **MUST** return the existing task_id rather than creating a new download. This includes checking both active downloads and paused/failed tasks that can be resumed. The system **MUST** provide mechanisms to find and reuse existing task_ids for identical download operations.

**Rationale**: Prevents resource waste, storage duplication, and ensures consistent download state management across the system.

### II. Unified API Design
**MUST** maintain a simple, consistent interface across all download managers and backends. All download operations **MUST** be accessible through both direct manager instances and convenience functions. The API **MUST** be backend-agnostic, allowing seamless switching between aria2, qBittorrent, or other download engines without code changes.

**Rationale**: Ensures maintainability and flexibility while reducing integration complexity for consumers.

### III. Persistence-First Architecture
**MUST** persist all download tasks and progress to database storage by default. The system **MUST** automatically recover incomplete downloads on restart. Database operations **MUST** be atomic and consistent with the download engine state. Progress **MUST** be saved at regular intervals to prevent data loss.

**Rationale**: Provides reliability and continuity for long-running downloads across application restarts.

### IV. Async-First Design
**MUST** implement all operations using async/await patterns with proper error handling. Concurrent operations **MUST** respect configurable limits (default: 3 concurrent downloads). Background tasks **MUST** be properly managed with graceful shutdown capabilities.

**Rationale**: Ensures scalable, non-blocking operations suitable for high-performance applications.

### V. Test-Driven Development (NON-NEGOTIABLE)
**MUST** follow strict Test-Driven Development (TDD) methodology for ALL code changes. Tests **MUST** be written FIRST and **MUST** FAIL before implementation begins. The Red-Green-Refactor cycle is **MANDATORY**: Red (write failing test) → Green (implement minimal code to pass) → Refactor (improve code quality). **NO** production code may be written without corresponding failing tests. All public APIs **MUST** have comprehensive test coverage including unit tests, integration tests, and end-to-end scenarios. Database interactions **MUST** be tested with real database operations. Error scenarios **MUST** be explicitly tested with failing test cases written first.

**Rationale**: TDD ensures code quality, prevents regressions, validates requirements early, and produces maintainable code with complete test coverage. Critical for download management systems where reliability is paramount.

## Download Management Constraints

**File Organization**: All downloads default to `./data/` directory with customizable paths. Directory creation **MUST** be automatic and atomic.

**Progress Tracking**: Progress updates **MUST** be saved every 5 seconds. Status changes **MUST** be immediately persisted to database.

**Error Handling**: Failed downloads **MUST** be marked in database with error details. Recovery **MUST** be attempted on restart for incomplete tasks.

**Task Lifecycle**: Tasks **MUST** support states: Waiting, Downloading, Paused, Completed, Failed. State transitions **MUST** be validated and logged.

## Development Workflow

**Test-Driven Development Process**: Before implementing ANY feature or fix, developers **MUST** write failing tests that specify the exact expected behavior. Tests **MUST** fail initially, proving they test the unimplemented functionality. Only after tests fail should implementation begin. The implementation **MUST** be the minimal code needed to make tests pass. After tests pass, code **MUST** be refactored for quality while maintaining test coverage.

**Duplicate Detection Implementation**: Before adding new download tasks, **MUST** check for existing tasks with same URL and target path. **MUST** provide methods to query existing tasks by URL/path criteria. Tests for duplicate detection **MUST** be written first and fail before implementation.

**Database Schema**: **MUST** enforce unique constraints on URL+target_path combinations where appropriate. **MUST** provide efficient querying for duplicate detection. Database changes **MUST** be test-driven with failing tests written first.

**API Consistency**: All download functions **MUST** check for duplicates before creating new tasks. **MUST** return existing task_id when duplicate detected. API behavior **MUST** be specified by failing tests before implementation.

**Testing Requirements**: All new functionality **MUST** start with failing tests. **MUST** test duplicate detection scenarios in integration tests. **MUST** verify that duplicate requests return same task_id. **MUST** test error conditions and edge cases with failing tests written first.

## Governance

This constitution supersedes all other development practices for the BurnCloud Download Manager. All pull requests **MUST** verify compliance with these principles, particularly the TDD requirement and duplicate detection. Any deviation from these principles **MUST** be explicitly justified and documented.

**TDD Compliance**: Pull requests **MUST** demonstrate TDD compliance by showing test commits BEFORE implementation commits. Code reviews **MUST** verify that tests were written first and initially failed. **NO** exceptions to TDD are permitted - this principle is **NON-NEGOTIABLE**.

Implementation complexity **MUST** be justified against the benefit provided. The duplicate detection and TDD principles are both **NON-NEGOTIABLE** as they prevent critical resource waste, state inconsistencies, and code quality issues.

**Version**: 1.1.0 | **Ratified**: 2025-10-09 | **Last Amended**: 2025-10-09