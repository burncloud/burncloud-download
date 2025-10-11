# Research: Database Duplicate Records and URL Recording Bug Fix

**Feature Branch**: `002-url-bug`
**Date**: 2025-10-10
**Research Phase**: Phase 0 of `/speckit.plan` workflow

## Research Objectives

Investigate the root causes of database duplicate records and incorrect URL storage to determine technical solutions for the bug fix implementation.

## Investigation Findings

### 1. URL Storage Issues Analysis

**Decision**: Implement comprehensive URL normalization with hash-based storage
**Rationale**: Current implementation stores raw URLs without normalization, causing `https://example.com/file.zip` and `https://example.com/file.zip?timestamp=123` to be treated as different downloads
**Alternatives considered**:
- String-based deduplication (rejected - too fragile for URL variations)
- Custom URL fingerprinting (rejected - 'url' crate provides better standards compliance)

**Technical Issues Identified**:
- No URL normalization before database storage (`src/manager/persistent_aria2.rs:471-504`)
- Missing URL hash column in database schema (`src/schema.rs`)
- Simple string comparison in duplicate detection instead of normalized comparison
- No validation or sanitization of URLs before storage

### 2. Duplicate Detection Failure Analysis

**Decision**: Fix race conditions with atomic UPSERT operations and unique constraints
**Rationale**: Current check-then-insert pattern allows race conditions where two processes can insert the same URL+path combination simultaneously
**Alternatives considered**:
- Application-level locking (rejected - doesn't prevent database-level races)
- Retry logic (rejected - doesn't address root cause)

**Critical Problems Found**:
- `DuplicateDetector` trait has placeholder implementations only (`src/services/duplicate_detector.rs`)
- No unique constraints on `(url, target_path)` combinations in database schema
- Non-atomic duplicate checking in `find_duplicate_task` method
- Race condition window between duplicate check and task insertion

### 3. Database Schema Constraints Research

**Decision**: Add unique constraint on `(url_hash, target_path)` combination
**Rationale**: Database-level constraints provide the strongest guarantee against duplicates, even under concurrent load
**Alternatives considered**:
- Application-level duplicate prevention (rejected - insufficient for race conditions)
- Unique constraint on raw URL (rejected - prevents legitimate URL variations)

**Required Schema Changes**:
```sql
-- Add URL hash column for efficient duplicate detection
ALTER TABLE download_tasks ADD COLUMN url_hash TEXT;

-- Create unique index to prevent duplicates
CREATE UNIQUE INDEX idx_url_hash_path_unique ON download_tasks(url_hash, target_path);

-- Add index for efficient lookup
CREATE INDEX idx_url_hash ON download_tasks(url_hash);
```

### 4. URL Normalization Best Practices

**Decision**: Use Rust 'url' crate with comprehensive normalization including scheme, host, port, fragment, and query parameter standardization
**Rationale**: The 'url' crate provides standards-compliant URL parsing and normalization, essential for reliable duplicate detection
**Alternatives considered**:
- Custom URL normalization (rejected - error-prone and incomplete)
- Simple string manipulation (rejected - insufficient for URL complexity)

**Normalization Requirements**:
- Convert scheme to lowercase (`HTTP` → `http`)
- Remove default ports (`:80` for HTTP, `:443` for HTTPS)
- Remove URL fragments (`#section`)
- Sort query parameters for consistent ordering
- Normalize host case (automatic with 'url' crate)
- Handle path normalization (`/../` resolution)

### 5. SQLite Transaction Patterns Research

**Decision**: Implement atomic UPSERT with transaction isolation for duplicate prevention
**Rationale**: SQLite's `INSERT ... ON CONFLICT` provides atomic duplicate handling without race conditions
**Alternatives considered**:
- SELECT then INSERT pattern (rejected - race condition prone)
- Advisory locking (rejected - SQLite doesn't support PostgreSQL-style advisory locks)

**Recommended Transaction Pattern**:
```sql
INSERT INTO download_tasks (id, url_hash, url, target_path, status, created_at, updated_at)
VALUES (?, ?, ?, ?, ?, ?, ?)
ON CONFLICT(url_hash, target_path) DO UPDATE SET
    updated_at = excluded.updated_at
RETURNING id;
```

### 6. Database Integration Architecture

**Decision**: Integrate URL normalization into existing `FileIdentifier` and `TaskRepository` services
**Rationale**: Existing architecture has proper separation of concerns; implementation needs completion rather than redesign
**Alternatives considered**:
- New duplicate detection service (rejected - duplicates existing architecture)
- Manager-level duplicate handling (rejected - violates separation of concerns)

**Implementation Points**:
- Complete `DuplicateDetector` trait implementation in `src/services/duplicate_detector.rs`
- Update `TaskRepository` to use URL hashes for queries
- Integrate normalization into `PersistentAria2Manager.add_download()` flow
- Ensure all database operations use consistent URL hash approach

## Implementation Readiness

All NEEDS CLARIFICATION items from Technical Context have been resolved:
- **URL Storage**: Comprehensive normalization strategy defined
- **Duplicate Detection**: Race condition solutions identified
- **Database Constraints**: Schema changes specified
- **Transaction Handling**: Atomic operation patterns selected
- **Rust Integration**: 'url' crate usage patterns established

## Next Phase: Design Artifacts

Phase 1 will generate:
- `data-model.md`: Updated entity definitions with URL hash fields
- `contracts/`: Database schema migration scripts
- `quickstart.md`: Implementation guidance for developers