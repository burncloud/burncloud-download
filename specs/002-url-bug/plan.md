# Implementation Plan: Database Duplicate Records and URL Recording Bug Fix

**Branch**: `002-url-bug` | **Date**: 2025-10-10 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-url-bug/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Fix critical database integrity issues in the Rust-based BurnCloud Download Manager where duplicate records are being created for identical downloads (same URL + target path), and URLs are not being correctly stored in the database. This breaks duplicate detection, task recovery, and download resumption functionality. Implementation must follow strict TDD methodology and maintain 100% Rust codebase as required.

## Technical Context

**Language/Version**: Rust 1.75+ (per user requirement: "使用rust开发，勿用其它语言")
**Primary Dependencies**: tokio (async runtime), burncloud-database-download (database layer), burncloud-download-aria2 (download engine), blake3 (hashing), url (URL handling)
**Storage**: SQLite database via burncloud-database-download crate for task persistence
**Testing**: cargo test with TDD methodology (NON-NEGOTIABLE per constitution)
**Target Platform**: Cross-platform Rust application (Windows/Linux/macOS)
**Project Type**: Single Rust library crate with database integration
**Performance Goals**: <100ms duplicate detection queries, maintain current download performance
**Constraints**: 100% Rust codebase, no breaking changes to existing API, TDD mandatory
**Scale/Scope**: Fix specific URL storage and duplicate detection bugs in existing ~25 source files

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

✅ **I. Duplicate Detection and Task Reuse**: CRITICAL - This bug fix directly addresses duplicate detection failures
✅ **II. Unified API Design**: No API changes planned, existing interfaces preserved
✅ **III. Persistence-First Architecture**: Bug involves improving database persistence accuracy
✅ **IV. Async-First Design**: All fixes will maintain async patterns using tokio
✅ **V. Test-Driven Development (NON-NEGOTIABLE)**: TDD compliance enforced in quickstart.md implementation plan
✅ **Download Management Constraints**: All constraints maintained (file organization, progress tracking, error handling, task lifecycle)

**GATE STATUS**: ✅ PASS - TDD compliance will be strictly enforced during implementation

**POST-DESIGN RE-CHECK**: ✅ PASS - All design artifacts maintain constitutional compliance:
- research.md: Follows TDD principles and maintains async patterns
- data-model.md: Preserves existing API design while improving persistence
- contracts/: Database changes enhance duplicate detection without breaking interfaces
- quickstart.md: Explicitly enforces TDD methodology throughout implementation

## Project Structure

### Documentation (this feature)

```
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```
src/
├── models/                  # Data models including duplicate detection
│   ├── file_identifier.rs
│   ├── task_status.rs
│   ├── duplicate_policy.rs
│   ├── duplicate_result.rs
│   └── duplicate_reason.rs
├── services/                # Business logic services
│   ├── duplicate_detector.rs
│   ├── task_repository.rs
│   ├── hash_calculator.rs
│   └── task_validation.rs
├── manager/                 # Download managers
│   ├── basic.rs
│   └── persistent_aria2.rs  # TARGET: Fix URL storage and duplicate detection
├── types/                   # Core type definitions
├── traits/                  # Interface definitions
├── queue/                   # Task queue management
├── error/                   # Error handling
└── utils/                   # Utility functions

tests/
├── unit/                    # Unit tests (TDD required)
├── integration/             # Integration tests
└── end_to_end/             # E2E tests
```

**Structure Decision**: Single Rust library crate structure is appropriate for this bug fix. The existing modular architecture with models/ and services/ directories provides good separation of concerns for duplicate detection logic.
