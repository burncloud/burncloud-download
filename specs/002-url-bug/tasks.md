# Tasks: Database Duplicate Records and URL Recording Bug Fix

**Input**: Design documents from `/specs/002-url-bug/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: All tests are MANDATORY following Test-Driven Development (TDD) principles. Tests MUST be written FIRST and MUST FAIL before implementation begins. This enforces the Red-Green-Refactor cycle required by the project constitution.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions
- **Single project**: `src/`, `tests/` at repository root
- Paths assume Rust single project structure as per plan.md

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and migration foundation

- [X] T001 [P] Set up database backup before schema changes for rollback capability
- [X] T002 [P] Apply database migration `contracts/001_add_url_hash_column.sql` to add url_hash column to download_tasks table
- [X] T003 Create URL normalization utility module in `src/utils/url_normalization.rs` with Blake3 hashing functions
- [X] T004 [P] Create database migration runner binary in `src/bin/migrate_url_hashes.rs` using `contracts/migration_helpers.rs` code
- [X] T005 Run Rust migration to populate url_hash for all existing download_tasks records
- [X] T006 Apply database migration `contracts/002_add_duplicate_constraints.sql` to add unique constraints and indexes

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T007 [P] Write failing unit tests for URL normalization function in `tests/unit/url_normalization_tests.rs`
- [X] T008 [P] Write failing unit tests for Blake3 hash computation in `tests/unit/hash_calculation_tests.rs`
- [X] T009 Implement URL normalization function in `src/utils/url_normalization.rs` (scheme, host, port, fragment, query param sorting)
- [X] T010 Implement Blake3 hash function for normalized URLs in `src/utils/url_normalization.rs`
- [X] T011 [P] Write failing unit tests for DuplicateDetector trait in `tests/unit/duplicate_detector_tests.rs`
- [X] T012 [P] Write failing unit tests for TaskRepository UPSERT operations in `tests/unit/task_repository_tests.rs`
- [X] T013 Update `src/models/file_identifier.rs` to use new URL normalization functions
- [X] T014 Update `src/models/task_status.rs` to handle URL hash validation

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Eliminate Duplicate Download Records (Priority: P1) 🎯 MVP

**Goal**: Prevent creation of duplicate database records for downloads with identical URL and target path combinations

**Independent Test**: Can be fully tested by initiating the same download multiple times and verifying only one database record exists

### Tests for User Story 1 (MANDATORY - TDD Required) ⚠️

**CONSTITUTION REQUIREMENT: Write these tests FIRST, ensure they FAIL before ANY implementation**

- [X] T015 [P] [US1] Write failing integration test for duplicate detection in `tests/integration/duplicate_detection_tests.rs`
- [X] T016 [P] [US1] Write failing integration test for concurrent duplicate requests in `tests/integration/concurrent_duplicate_tests.rs`
- [X] T017 [P] [US1] Write failing end-to-end test for download deduplication workflow in `tests/end_to_end/download_deduplication_tests.rs`

### Implementation for User Story 1

- [ ] T018 [P] [US1] Implement DuplicateDetector trait in `src/services/duplicate_detector.rs` with URL hash-based detection
- [ ] T019 [P] [US1] Create DuplicateResult enhanced model in `src/models/duplicate_result.rs` with proper error handling
- [ ] T020 [US1] Implement UPSERT operation in `src/services/task_repository.rs` using SQLite INSERT...ON CONFLICT pattern
- [ ] T021 [US1] Update PersistentAria2Manager in `src/manager/persistent_aria2.rs` to use new duplicate detection
- [ ] T022 [US1] Replace string-based URL comparison with hash-based lookup in duplicate detection flow
- [ ] T023 [US1] Add transaction isolation for atomic duplicate checking operations
- [ ] T024 [US1] Integrate URL normalization into download task creation workflow

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently - no duplicate records should be created

---

## Phase 4: User Story 2 - Accurate URL Storage in Database (Priority: P1)

**Goal**: Store complete and correct source URLs in the database to enable proper duplicate detection, task recovery, and download resumption

**Independent Test**: Can be fully tested by initiating downloads with various URL formats and verifying the complete URL is stored correctly in the database

### Tests for User Story 2 (MANDATORY - TDD Required) ⚠️

- [ ] T025 [P] [US2] Write failing integration test for URL storage with query parameters in `tests/integration/url_storage_tests.rs`
- [ ] T026 [P] [US2] Write failing integration test for URL storage with special characters in `tests/integration/url_encoding_tests.rs`
- [ ] T027 [P] [US2] Write failing integration test for URL storage with redirects in `tests/integration/url_redirect_tests.rs`

### Implementation for User Story 2

- [ ] T028 [P] [US2] Enhanced URL validation in `src/utils/url_normalization.rs` with comprehensive error handling
- [ ] T029 [P] [US2] Implement URL storage integrity validation in `src/services/task_validation.rs`
- [ ] T030 [US2] Update TaskRepository to store both original and normalized URLs in `src/services/task_repository.rs`
- [ ] T031 [US2] Add URL format validation before database storage operations
- [ ] T032 [US2] Implement URL truncation prevention in database schema validation
- [ ] T033 [US2] Add logging for URL processing operations with detailed error reporting

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently - accurate URL storage with no duplicates

---

## Phase 5: User Story 3 - Robust Database Transaction Handling (Priority: P2)

**Goal**: Ensure atomic operations and prevent race conditions in database operations for concurrent download requests

**Independent Test**: Can be fully tested by initiating multiple downloads simultaneously and verifying database consistency is maintained

### Tests for User Story 3 (MANDATORY - TDD Required) ⚠️

- [ ] T034 [P] [US3] Write failing integration test for concurrent database operations in `tests/integration/concurrent_operations_tests.rs`
- [ ] T035 [P] [US3] Write failing integration test for transaction rollback scenarios in `tests/integration/transaction_rollback_tests.rs`
- [ ] T036 [P] [US3] Write failing integration test for database consistency under load in `tests/integration/database_consistency_tests.rs`

### Implementation for User Story 3

- [ ] T037 [P] [US3] Implement proper transaction isolation levels in `src/services/task_repository.rs`
- [ ] T038 [P] [US3] Add transaction timeout and retry logic for database operations
- [ ] T039 [US3] Implement atomic batch operations for multiple download requests
- [ ] T040 [US3] Add database connection pooling optimization for concurrent operations
- [ ] T041 [US3] Implement proper error handling and rollback for failed transactions
- [ ] T042 [US3] Add transaction logging and monitoring for debugging

**Checkpoint**: All user stories should now be independently functional with full database integrity under concurrent load

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories and final validation

- [ ] T043 [P] Run full test suite to ensure no regressions across all user stories
- [ ] T044 [P] Performance testing with concurrent load as specified in success criteria (<100ms duplicate detection)
- [ ] T045 [P] Validate Success Criteria SC-001: Zero duplicate database records exist
- [ ] T046 [P] Validate Success Criteria SC-002: 100% of initiated downloads have complete URLs stored
- [ ] T047 [P] Validate Success Criteria SC-003: Duplicate detection operates correctly in 100% of test cases
- [ ] T048 [P] Validate Success Criteria SC-004: System maintains data integrity under concurrent load
- [ ] T049 [P] Validate Success Criteria SC-005: Database query performance under 100ms for 95% of operations
- [ ] T050 [P] Code cleanup and refactoring for improved maintainability
- [ ] T051 [P] Update documentation in `README.md` and `INTEGRATION.md`
- [ ] T052 Run quickstart.md validation steps to ensure implementation matches design

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-5)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 US1 → P1 US2 → P2 US3)
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P1)**: Can start after Foundational (Phase 2) - Builds on US1's duplicate detection but independently testable
- **User Story 3 (P2)**: Can start after Foundational (Phase 2) - Uses transaction patterns from US1/US2 but independently testable

### Within Each User Story

- Tests (MANDATORY) MUST be written FIRST and FAIL before implementation begins (TDD requirement)
- Models before services
- Services before manager integration
- Core implementation before cross-cutting concerns
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, all user stories can start in parallel (if team capacity allows)
- All tests for a user story marked [P] can run in parallel
- Models and utilities within a story marked [P] can run in parallel
- Different user stories can be worked on in parallel by different team members

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task: "Write failing integration test for duplicate detection"
Task: "Write failing integration test for concurrent duplicate requests"
Task: "Write failing end-to-end test for download deduplication workflow"

# Launch all parallel implementation tasks for User Story 1 together:
Task: "Implement DuplicateDetector trait"
Task: "Create DuplicateResult enhanced model"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (database migrations)
2. Complete Phase 2: Foundational (URL normalization and basic infrastructure)
3. Complete Phase 3: User Story 1 (duplicate detection)
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo if ready - basic duplicate prevention working

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP! - duplicate prevention)
3. Add User Story 2 → Test independently → Deploy/Demo (accurate URL storage)
4. Add User Story 3 → Test independently → Deploy/Demo (full concurrent robustness)
5. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (duplicate detection)
   - Developer B: User Story 2 (URL storage)
   - Developer C: User Story 3 (transaction handling)
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- **TDD CRITICAL**: Verify tests fail before implementing (constitution mandated)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Follow Rust 1.75+ requirements - no other languages permitted
- All database operations must maintain SQLite compatibility
- Ensure backward compatibility with existing download manager interfaces