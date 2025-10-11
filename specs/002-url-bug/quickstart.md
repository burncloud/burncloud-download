# Quick Start: Database Duplicate Records and URL Recording Bug Fix

**Feature Branch**: `002-url-bug`
**Date**: 2025-10-10
**Implementation Phase**: Phase 1 Design Artifacts

## Overview

This guide provides implementation guidance for fixing the database duplicate records and URL recording bugs in the BurnCloud Download Manager. The fix involves adding URL normalization, hash-based duplicate detection, and atomic database operations.

## Prerequisites

- Rust 1.75+ (as specified by user requirement)
- Existing burncloud-download codebase
- SQLite database backend
- TDD methodology (NON-NEGOTIABLE per constitution)

## Implementation Strategy

### Phase 1: Database Schema Migration

**Required Steps**:
1. Add `url_hash` column to `download_tasks` table
2. Populate existing records with normalized URL hashes
3. Add unique constraint on `(url_hash, target_path)`
4. Create performance indexes

**Migration Order**:
```bash
# Apply migrations in sequence
sqlite3 database.db < contracts/001_add_url_hash_column.sql
cargo run --bin migrate-url-hashes  # Run Rust migration helper
sqlite3 database.db < contracts/002_add_duplicate_constraints.sql
```

### Phase 2: Service Layer Implementation

**Key Components to Update**:

1. **URL Normalization Service** (`src/utils/url_normalization.rs` - new file)
   - Implement comprehensive URL normalization
   - Use Blake3 for consistent hashing
   - Handle edge cases (query params, fragments, default ports)

2. **DuplicateDetector Service** (`src/services/duplicate_detector.rs`)
   - Complete trait implementation (currently placeholder)
   - Use URL hash for efficient database queries
   - Return appropriate DuplicateResult

3. **TaskRepository Service** (`src/services/task_repository.rs`)
   - Add UPSERT operations for atomic duplicate handling
   - Update queries to use url_hash for lookups
   - Implement transaction isolation

### Phase 3: Manager Integration

**PersistentAria2Manager Updates** (`src/manager/persistent_aria2.rs`):
- Update `add_download()` to use new duplicate detection
- Replace string-based URL comparison with hash-based lookup
- Ensure atomic operations prevent race conditions

## Test-Driven Development Plan

### Required Test Categories

1. **Unit Tests** (Write tests FIRST, following TDD):
   ```rust
   // Example failing test to write first
   #[tokio::test]
   async fn test_url_normalization_prevents_duplicates() {
       // This test should FAIL initially
       let url1 = "https://example.com/file.zip?timestamp=123";
       let url2 = "https://example.com/file.zip?timestamp=456";

       let hash1 = hash_normalized_url(&normalize_url(url1).unwrap());
       let hash2 = hash_normalized_url(&normalize_url(url2).unwrap());

       // Should be different due to different query params
       assert_ne!(hash1, hash2);
   }
   ```

2. **Integration Tests**:
   - Database constraint enforcement
   - Concurrent duplicate detection
   - Migration validation

3. **End-to-End Tests**:
   - Full download workflow with duplicate detection
   - Error handling and recovery scenarios

### TDD Implementation Flow

```bash
# 1. Write failing test
cargo test test_url_normalization -- --nocapture
# Test should FAIL

# 2. Implement minimal code to pass test
# Add normalization logic

# 3. Run test again
cargo test test_url_normalization -- --nocapture
# Test should PASS

# 4. Refactor and repeat for next feature
```

## Key Implementation Files

### New Files to Create
- `src/utils/url_normalization.rs` - URL processing utilities
- `migrations/002_url_hash_migration.rs` - Database migration runner

### Existing Files to Modify
- `src/services/duplicate_detector.rs` - Complete implementation
- `src/services/task_repository.rs` - Add UPSERT operations
- `src/manager/persistent_aria2.rs` - Use new duplicate detection
- `src/models/file_identifier.rs` - Update with hash storage

## Database Operations

### UPSERT Pattern for Duplicate Prevention
```rust
// Example atomic operation to prevent race conditions
async fn add_download_atomic(
    &self,
    url: &str,
    target_path: &Path,
) -> Result<TaskId> {
    let (normalized_url, url_hash) = process_url_for_storage(url)?;

    let result = sqlx::query_as!(
        DownloadTask,
        r#"
        INSERT INTO download_tasks (id, url, url_hash, target_path, status, created_at, updated_at)
        VALUES (?, ?, ?, ?, 'Waiting', datetime('now'), datetime('now'))
        ON CONFLICT(url_hash, target_path) DO UPDATE SET
            updated_at = datetime('now')
        RETURNING id, url, url_hash, target_path, status, created_at, updated_at
        "#,
        generate_task_id(),
        normalized_url,
        url_hash,
        target_path.to_string_lossy()
    )
    .fetch_one(&mut *tx)
    .await?;

    Ok(result.id)
}
```

## Error Handling Strategy

### URL Processing Errors
- Invalid URLs: Return clear error messages
- Normalization failures: Log and use fallback strategy
- Hash computation errors: Should never occur with Blake3

### Database Constraint Violations
- Duplicate key errors: Return existing TaskId
- Migration failures: Rollback and retry
- Transaction conflicts: Implement retry logic

## Performance Considerations

### Index Usage
- Primary queries use `url_hash` index (O(1) lookup)
- Avoid full table scans on URL strings
- Use partial indexes for status filtering

### Memory Management
- Stream large migration operations
- Batch URL hash computations
- Use connection pooling for concurrent operations

## Success Criteria Validation

After implementation, verify:

1. **Zero duplicate records**: Query database for duplicate (url_hash, target_path) combinations
2. **Complete URL storage**: Verify all URLs are stored with proper normalization
3. **Performance**: Duplicate detection queries complete in <100ms
4. **Concurrency**: Multiple simultaneous requests don't create duplicates

## Rollback Plan

If implementation issues occur:
1. Disable unique constraints temporarily
2. Roll back to previous schema version
3. Clean up duplicate records manually
4. Re-apply migration with fixes

## Next Steps

After completing this implementation:
1. Run full test suite to ensure no regressions
2. Performance testing with concurrent load
3. Code review focusing on TDD compliance
4. Deploy to staging environment for validation

## Common Pitfalls to Avoid

1. **Skipping TDD**: Write tests FIRST - this is NON-NEGOTIABLE
2. **Incomplete URL normalization**: Use comprehensive normalization from research.md
3. **Race conditions**: Always use atomic database operations
4. **Migration without backup**: Backup database before schema changes
5. **Performance regression**: Monitor query performance after adding constraints