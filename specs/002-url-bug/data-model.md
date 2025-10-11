# Data Model: Database Duplicate Records and URL Recording Bug Fix

**Feature Branch**: `002-url-bug`
**Date**: 2025-10-10
**Design Phase**: Phase 1 of `/speckit.plan` workflow

## Entity Definitions

### Core Entities from Feature Specification

#### DownloadTask (Updated)
Primary entity representing a download operation with enhanced duplicate detection.

**Fields**:
- `id: TaskId` - Unique identifier for the download task
- `url: String` - Original URL as provided by user (for display/logging)
- `url_hash: String` - Blake3 hash of normalized URL (for efficient duplicate detection)
- `target_path: PathBuf` - Local filesystem path where file will be saved
- `status: DownloadStatus` - Current state of the download
- `created_at: DateTime` - When the task was created
- `updated_at: DateTime` - Last modification time
- `file_size: Option<u64>` - Total file size if known
- `downloaded_bytes: u64` - Bytes downloaded so far

**Validation Rules**:
- `url` must be valid HTTP/HTTPS URL (enforced by 'url' crate parsing)
- `url_hash` must be 64-character Blake3 hex string
- `target_path` must be valid filesystem path
- `status` transitions must follow valid state machine
- `downloaded_bytes` cannot exceed `file_size` when file_size is Some

**State Transitions**:
```
Waiting → Downloading → (Completed | Failed | Paused)
Paused → (Downloading | Cancelled)
Failed → (Downloading | Cancelled) // Resume on retry
```

**Relationships**:
- Unique constraint on `(url_hash, target_path)` combination
- Indexed by `url_hash` for efficient duplicate queries
- Foreign key relationships with progress tracking (existing)

#### TaskIdentifier (Enhanced)
Composite key for reliable duplicate detection using normalized URLs.

**Fields**:
- `url_hash: String` - Blake3 hash of normalized URL
- `target_path_hash: String` - Blake3 hash of canonical target path
- `combined_hash: String` - Blake3 hash of `url_hash + target_path_hash`

**Validation Rules**:
- All hash fields must be 64-character Blake3 hex strings
- `combined_hash` must equal Blake3(`url_hash` + `target_path_hash`)
- Used for efficient duplicate detection queries

**Usage**:
```rust
// Example usage in duplicate detection
let identifier = TaskIdentifier::new(normalized_url, canonical_path)?;
let existing_task = repository.find_by_identifier(&identifier).await?;
```

### Updated Entity Relationships

#### FileIdentifier → DownloadTask
- One-to-many relationship where each `FileIdentifier` can match multiple `DownloadTask` entries (different target paths)
- `FileIdentifier.url_hash` maps to `DownloadTask.url_hash`
- Used for finding all downloads of the same file to different locations

#### DuplicateResult → DownloadTask
- References specific `DownloadTask.id` when duplicate is detected
- Indicates whether existing task should be returned or new task created
- Tracks the reason for duplicate decision (same path, different path, policy override)

## Database Schema Changes

### Required Migrations

#### Migration 001: Add URL Hash Column
```sql
-- Add url_hash column to existing download_tasks table
ALTER TABLE download_tasks ADD COLUMN url_hash TEXT;

-- Populate url_hash for existing records using normalized URLs
UPDATE download_tasks SET url_hash = (
    -- This will be computed by migration script using Rust normalization
    -- Temporary placeholder for migration planning
);

-- Make url_hash non-null after population
ALTER TABLE download_tasks ALTER COLUMN url_hash SET NOT NULL;
```

#### Migration 002: Add Duplicate Prevention Constraints
```sql
-- Create unique index to prevent URL+path duplicates
CREATE UNIQUE INDEX idx_url_hash_path_unique
ON download_tasks(url_hash, target_path);

-- Add efficient lookup index for duplicate detection
CREATE INDEX idx_url_hash_lookup
ON download_tasks(url_hash);

-- Add index for status-based queries (maintain performance)
CREATE INDEX idx_status_lookup
ON download_tasks(status)
WHERE status != 'Completed';
```

### Schema Validation Rules

#### Database Constraints
- `UNIQUE(url_hash, target_path)` - Prevents exact duplicates
- `NOT NULL url_hash` - Ensures all tasks have normalized URL identifier
- `CHECK(length(url_hash) = 64)` - Validates Blake3 hash format
- `CHECK(url != '')` - Ensures original URL is preserved
- `CHECK(target_path != '')` - Ensures valid target path

#### Application-Level Validation
- URL normalization before hash computation
- Path canonicalization before storage
- Blake3 hash format validation
- Download status state machine enforcement

## Service Layer Updates

### DuplicateDetector Service
Enhanced implementation for reliable duplicate detection.

**Interface**:
```rust
#[async_trait]
pub trait DuplicateDetector {
    async fn find_duplicate(
        &self,
        url: &str,
        target_path: &Path,
    ) -> Result<DuplicateResult>;

    async fn find_by_url_hash(
        &self,
        url_hash: &str,
    ) -> Result<Vec<DownloadTask>>;
}
```

**Implementation Strategy**:
1. Normalize input URL using comprehensive normalization
2. Compute Blake3 hash of normalized URL
3. Query database for existing tasks with same `(url_hash, target_path)`
4. Return appropriate `DuplicateResult` based on findings

### TaskRepository Service
Updated to use URL hash for all duplicate-related queries.

**Key Methods**:
```rust
async fn find_by_url_hash_and_path(
    &self,
    url_hash: &str,
    target_path: &Path,
) -> Result<Option<DownloadTask>>;

async fn upsert_download_task(
    &self,
    url: &str,
    target_path: &Path,
) -> Result<TaskId>;
```

**UPSERT Implementation**:
- Use SQLite `INSERT ... ON CONFLICT ... DO UPDATE`
- Atomic operation prevents race conditions
- Returns existing `TaskId` if duplicate found
- Creates new task if no duplicate exists

## Data Flow Architecture

### Duplicate Detection Flow
1. **Input**: Raw URL + target path from user
2. **Normalization**: Apply comprehensive URL normalization
3. **Hashing**: Compute Blake3 hash of normalized URL
4. **Database Query**: Check for existing `(url_hash, target_path)` combination
5. **Decision**: Return existing task or create new task atomically
6. **Response**: Return `TaskId` to caller

### URL Storage Flow
1. **Validation**: Parse URL using 'url' crate
2. **Normalization**: Apply normalization rules from research phase
3. **Hash Computation**: Generate Blake3 hash for indexing
4. **Storage**: Store both original URL and hash in database
5. **Indexing**: Use hash for all duplicate detection queries

## Performance Considerations

### Index Strategy
- Primary lookup: `idx_url_hash_path_unique` for duplicate detection
- Secondary lookup: `idx_url_hash_lookup` for finding all downloads of same file
- Status queries: `idx_status_lookup` for active task management

### Query Optimization
- Use URL hash for all duplicate detection (O(1) hash lookup vs O(n) string comparison)
- Limit string comparison to final validation step
- Batch operations for bulk duplicate detection

### Memory Efficiency
- Store original URL for display purposes
- Use hash for all computational operations
- Canonical path representation to minimize storage overhead