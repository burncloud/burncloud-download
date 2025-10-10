# Tasks: Duplicate Download Detection

**Input**: Design documents from `/specs/001-burncloud-download-task/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: All tests are MANDATORY following Test-Driven Development (TDD) principles. Tests MUST be written FIRST and MUST FAIL before implementation begins. This enforces the Red-Green-Refactor cycle required by the project constitution.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions
- **Single project**: `src/`, `tests/` at repository root
- All paths assume Rust single project structure from plan.md

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [ ] T001 Add new dependencies to Cargo.toml (blake3 = "1.5", url = "2.5")
- [ ] T002 [P] Create module structure in src/lib.rs for duplicate detection exports
- [ ] T003 [P] Setup error handling types in src/lib/error.rs for duplicate detection errors

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T004 Create database migration script for duplicate detection fields (ALTER TABLE download_tasks)
- [ ] T005 [P] Write unit tests for FileIdentifier struct in tests/unit/file_identifier_tests.rs (MUST FAIL FIRST)
- [ ] T006 [P] Write unit tests for TaskStatus enum extensions in tests/unit/task_status_tests.rs (MUST FAIL FIRST)
- [ ] T007 [P] Write unit tests for DuplicatePolicy enum in tests/unit/duplicate_policy_tests.rs (MUST FAIL FIRST)
- [ ] T008 [P] Implement FileIdentifier struct in src/models/file_identifier.rs (make T005 pass)
- [ ] T009 [P] Extend TaskStatus enum with Duplicate variant in src/models/task_status.rs (make T006 pass)
- [ ] T010 [P] Implement DuplicatePolicy enum in src/models/duplicate_policy.rs (make T007 pass)
- [ ] T011 Write unit tests for DownloadTask extensions in tests/unit/download_task_tests.rs (MUST FAIL FIRST)
- [ ] T012 Extend DownloadTask struct with duplicate detection fields in src/models/download_task.rs (make T011 pass)
- [ ] T013 Write unit tests for database operations in tests/unit/task_repository_tests.rs (MUST FAIL FIRST)
- [ ] T014 Implement database operations for duplicate detection in src/services/task_repository.rs (make T013 pass)

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Resume Existing Download (Priority: P1) 🎯 MVP

**Goal**: When users request downloads of incomplete files, system automatically resumes existing task instead of creating new one

**Independent Test**: Start a download, interrupt it, request same file again - should resume original task with same task_id

### Tests for User Story 1 (MANDATORY - TDD Required) ⚠️

**CONSTITUTION REQUIREMENT: Write these tests FIRST, ensure they FAIL before ANY implementation**

- [ ] T015 [P] [US1] Contract test for find_duplicate_task method in tests/contract/duplicate_detection_tests.rs (MUST FAIL FIRST)
- [ ] T016 [P] [US1] Integration test for resume existing download workflow in tests/integration/download_resume_tests.rs (MUST FAIL FIRST)
- [ ] T017 [P] [US1] Unit test for DuplicateDetector service core logic in tests/unit/duplicate_detector_tests.rs (MUST FAIL FIRST)

### Implementation for User Story 1

- [ ] T018 [P] [US1] Implement DuplicateDetector service with find_by_url_and_path method in src/services/duplicate_detector.rs (make T017 pass)
- [ ] T019 [US1] Add find_duplicate_task method to DownloadManager trait in src/services/download_manager.rs (make T015 pass)
- [ ] T020 [US1] Implement URL normalization and hashing functions in src/models/file_identifier.rs
- [ ] T021 [US1] Update PersistentAria2Manager to use duplicate detection in add_download method (make T016 pass)
- [ ] T022 [US1] Add logging for duplicate detection operations in resume workflow
- [ ] T023 [US1] Add validation to ensure task status transitions are valid for resume operations

**Checkpoint**: At this point, User Story 1 should be fully functional - users can resume interrupted downloads automatically

---

## Phase 4: User Story 2 - Detect Completed Downloads (Priority: P2)

**Goal**: When users request files that are already completed, system returns existing completed task instead of creating new download

**Independent Test**: Complete a download, request same file again - should return existing completed task_id without starting new download

### Tests for User Story 2 (MANDATORY - TDD Required) ⚠️

**CONSTITUTION REQUIREMENT: Write these tests FIRST, ensure they FAIL before ANY implementation**

- [ ] T024 [P] [US2] Contract test for add_download_with_policy method in tests/contract/duplicate_detection_tests.rs (MUST FAIL FIRST)
- [ ] T025 [P] [US2] Integration test for completed download reuse workflow in tests/integration/completed_download_tests.rs (MUST FAIL FIRST)
- [ ] T026 [P] [US2] Unit test for DuplicateResult type handling in tests/unit/duplicate_result_tests.rs (MUST FAIL FIRST)

### Implementation for User Story 2

- [ ] T027 [P] [US2] Create DuplicateResult enum in src/models/duplicate_result.rs (make T026 pass)
- [ ] T028 [P] [US2] Create DuplicateReason enum in src/models/duplicate_reason.rs
- [ ] T029 [US2] Implement add_download_with_policy method in DownloadManager trait (make T024 pass)
- [ ] T030 [US2] Add policy application logic to DuplicateDetector service for completed downloads
- [ ] T031 [US2] Update PersistentAria2Manager to handle completed download detection (make T025 pass)
- [ ] T032 [US2] Add user choice handling for re-download vs reuse scenarios
- [ ] T033 [US2] Add logging for completed download detection operations

**Checkpoint**: At this point, User Story 2 should be fully functional - users get immediate access to completed downloads

---

## Phase 5: User Story 3 - Handle Failed Download Recovery (Priority: P3)

**Goal**: When users request files that previously failed, system offers retry using existing task based on failure reason

**Independent Test**: Cause a download to fail, request same file again - should offer retry with existing task_id and preserve failure history

### Tests for User Story 3 (MANDATORY - TDD Required) ⚠️

**CONSTITUTION REQUIREMENT: Write these tests FIRST, ensure they FAIL before ANY implementation**

- [ ] T034 [P] [US3] Contract test for verify_task_validity method in tests/contract/duplicate_detection_tests.rs (MUST FAIL FIRST)
- [ ] T035 [P] [US3] Integration test for failed download recovery workflow in tests/integration/failed_download_recovery_tests.rs (MUST FAIL FIRST)
- [ ] T036 [P] [US3] Unit test for task validity verification logic in tests/unit/task_validation_tests.rs (MUST FAIL FIRST)

### Implementation for User Story 3

- [ ] T037 [P] [US3] Implement verify_task_validity method in DownloadManager trait (make T034 pass)
- [ ] T038 [P] [US3] Create TaskValidation service for checking file existence and source accessibility in src/services/task_validation.rs (make T036 pass)
- [ ] T039 [US3] Add failed task recovery logic to DuplicateDetector service
- [ ] T040 [US3] Implement HTTP HEAD request validation for source URL accessibility
- [ ] T041 [US3] Update PersistentAria2Manager to handle failed download recovery (make T035 pass)
- [ ] T042 [US3] Add failure reason analysis and recovery decision logic
- [ ] T043 [US3] Add logging for failed download recovery operations

**Checkpoint**: At this point, User Story 3 should be fully functional - users can recover from failed downloads efficiently

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Optimization, advanced features, and system-wide improvements

### Background Hash Calculation

- [ ] T044 [P] Write unit tests for background hash calculation in tests/unit/hash_calculator_tests.rs (MUST FAIL FIRST)
- [ ] T045 [P] Implement BackgroundHashCalculator service in src/services/hash_calculator.rs (make T044 pass)
- [ ] T046 Integrate background hash calculation with completed download events
- [ ] T047 Add file-based duplicate detection using calculated hashes

### Advanced Duplicate Detection

- [ ] T048 [P] Write unit tests for get_duplicate_candidates method in tests/unit/duplicate_candidates_tests.rs (MUST FAIL FIRST)
- [ ] T049 [P] Implement get_duplicate_candidates method in DownloadManager trait (make T048 pass)
- [ ] T050 Add fuzzy filename matching for similar downloads
- [ ] T051 Implement duplicate detection performance optimization with caching

### CLI Integration

- [ ] T052 [P] Write integration tests for CLI duplicate detection commands in tests/integration/cli_duplicate_tests.rs (MUST FAIL FIRST)
- [ ] T053 [P] Update CLI commands to support duplicate detection policies in src/cli/download_commands.rs (make T052 pass)
- [ ] T054 Add CLI flags for force-new-download and duplicate handling options

### Event System Enhancement

- [ ] T055 [P] Write unit tests for duplicate detection events in tests/unit/duplicate_events_tests.rs (MUST FAIL FIRST)
- [ ] T056 [P] Add DuplicateDetected and TaskReused events to event system (make T055 pass)
- [ ] T057 Integrate duplicate detection events with existing notification system

### Performance Optimization

- [ ] T058 [P] Add database indexes for duplicate detection queries (part of migration script)
- [ ] T059 [P] Implement query performance monitoring and metrics
- [ ] T060 Add concurrent duplicate detection for batch operations

### Documentation & Examples

- [ ] T061 [P] Update API documentation with duplicate detection methods
- [ ] T062 [P] Create code examples for common duplicate detection scenarios
- [ ] T063 [P] Update README with duplicate detection feature overview

---

## Dependencies and Execution Strategy

### User Story Dependencies
- **US1 (Resume)**: Independent - can be implemented first (MVP)
- **US2 (Completed)**: Depends on US1 duplicate detection foundation
- **US3 (Failed Recovery)**: Depends on US1 foundation, can be parallel with US2

### Parallel Execution Opportunities

**Phase 2 Foundation (can run in parallel after T004):**
- T005-T007: Test writing (different test files)
- T008-T010: Model implementation (different model files)

**User Story 1 (can run in parallel after Phase 2):**
- T015-T017: Test writing (different test files)
- T018, T020: Service implementation (different service files)

**User Story 2 (can run in parallel with US3 after US1):**
- T024-T026: Test writing (different test files)
- T027-T028: Model creation (different model files)

**Cross-Cutting Concerns (can run in parallel after all user stories):**
- T044, T048, T052, T055: Test writing (different test files)
- T045, T053, T056: Implementation (different service files)

### MVP Strategy
1. **Phase 1-2**: Setup and foundation (required for everything)
2. **Phase 3 (US1)**: Minimum viable product - resume existing downloads
3. **Phase 4 (US2)**: Enhanced value - detect completed downloads
4. **Phase 5 (US3)**: Complete functionality - handle failed downloads
5. **Phase 6**: Optimization and polish

### Success Criteria Validation
- **SC-001** (1-second response): Achieved through database indexing (T058) and efficient queries (T014)
- **SC-002** (80% bandwidth reduction): Achieved through US1-US3 duplicate detection
- **SC-003** (95% resume success): Achieved through US1 resume functionality and US3 validation
- **SC-004** (Zero new tasks for duplicates): Achieved through core duplicate detection in US1-US2
- **SC-005** (99% resume when source available): Achieved through US3 validation and retry logic

## Summary

**Total Tasks**: 63 tasks across 6 phases
**Task Breakdown by User Story**:
- Setup & Foundation: 14 tasks (T001-T014)
- US1 (Resume Existing): 9 tasks (T015-T023)
- US2 (Detect Completed): 10 tasks (T024-T033)
- US3 (Failed Recovery): 10 tasks (T034-T043)
- Polish & Cross-cutting: 20 tasks (T044-T063)

**Parallel Opportunities**: 45 tasks marked with [P] can run in parallel
**MVP Scope**: Phases 1-3 (T001-T023) provide core duplicate detection functionality
**TDD Compliance**: 21 test tasks ensure all implementation is test-driven per constitution