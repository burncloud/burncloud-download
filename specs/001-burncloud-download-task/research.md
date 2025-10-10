# Research: Duplicate Download Detection Implementation

**Date**: 2025-10-09
**Feature**: 001-burncloud-download-task
**Phase**: 0 - Research & Technical Clarification

## Technical Context Resolution

This research resolves the NEEDS CLARIFICATION items identified in the Technical Context section of the implementation plan.

### Primary Dependencies ✅ RESOLVED

**Current Technology Stack:**
- **HTTP Client**: `reqwest = "0.12"` with `["json"]` features
- **Async Runtime**: `tokio = "1.47"` with `["full"]` features
- **Serialization**: `serde = "1.0"` with `["derive"]` + `serde_json = "1.0"`
- **Database**: `sqlx = "0.7"` with SQLite backend, features: `["runtime-tokio-rustls", "sqlite", "chrono", "uuid"]`

**Decision**: Extend existing dependency stack without adding new major dependencies. Use existing `reqwest` for any additional HTTP operations, `tokio` for async duplicate detection, `serde` for task serialization, and `sqlx` for database queries.

### Storage System ✅ RESOLVED

**Current Implementation:**
- **Database**: SQLite via `sqlx` with full ACID compliance
- **Schema**: Existing `download_tasks` table with task persistence
- **Features**: Task lifecycle tracking, progress persistence, recovery on restart

**Decision**: Extend existing SQLite schema with duplicate detection columns:
- `file_hash TEXT` - Blake3 hash for file content comparison
- `file_size_bytes INTEGER` - File size for quick duplicate filtering
- `url_hash TEXT` - Normalized URL hash for URL-based duplicate detection
- Add indexes on hash columns for fast duplicate lookups

**Rationale**: Leverages existing database infrastructure, maintains ACID properties, and provides efficient duplicate detection queries.

### Target Platform ✅ RESOLVED

**Current Platform Support:**
- Cross-platform Rust library/CLI application
- Production deployment via `PersistentAria2Manager`
- SQLite for local persistence (cross-platform)
- Tokio async runtime (cross-platform)

**Decision**: Maintain cross-platform compatibility. Duplicate detection will work on all platforms supported by current codebase.

### Scale/Scope ✅ RESOLVED

**Current Limits:**
- **Concurrent Downloads**: Maximum 3 concurrent downloads (configurable)
- **Task Storage**: SQLite database (practically unlimited for typical usage)
- **Queue Management**: `TaskQueueManager` handles concurrency control

**Decision**: Design duplicate detection to handle current scale:
- Support 3+ concurrent duplicate checks
- Efficient database queries for existing task volumes
- Background hash calculation that doesn't impact download performance

## Architecture Analysis

### Current Codebase Structure

```rust
// Core Types (existing)
pub struct TaskId(uuid::Uuid);
pub struct DownloadTask {
    pub id: TaskId,
    pub url: String,
    pub target_path: PathBuf,
    pub status: DownloadStatus,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

pub enum DownloadStatus {
    Waiting,
    Downloading,
    Paused,
    Completed,
    Failed(String),
}
```

### Existing Manager Hierarchy

1. **DownloadManager Trait**: Core interface for all managers
2. **BasicDownloadManager**: Testing/mock implementation
3. **PersistentAria2Manager**: Production implementation with database
4. **TaskQueueManager**: Concurrency control wrapper

### Integration Strategy

**Extend Existing Components:**
- Add duplicate detection methods to `DownloadManager` trait
- Enhance `PersistentAria2Manager` with duplicate checking before task creation
- Extend `DownloadTask` model with hash/identifier fields
- Use existing event system for duplicate detection notifications

## Recommended Dependencies for Duplicate Detection

### New Dependencies (Minimal Additions)

```toml
# For fast file hashing
blake3 = "1.5"

# For URL normalization
url = "2.5"

# For fuzzy string matching (optional - for advanced duplicate detection)
strsim = "0.11"
```

**Decision Rationale:**
- **Blake3**: Fastest available hashing algorithm, significantly faster than SHA-256
- **url**: Standard Rust crate for URL parsing and normalization
- **strsim**: Lightweight crate for fuzzy filename matching

### Testing Dependencies (Extend Existing)

Current testing stack is comprehensive:
- `tokio-test = "0.4"` for async testing
- `mockito = "1.0"` for HTTP mocking
- `tempfile = "3.8"` for temporary files

**Decision**: Use existing testing infrastructure. Current patterns support TDD methodology required by constitution.

## Implementation Approach

### 1. Database Schema Extension

```sql
-- Add duplicate detection columns to existing table
ALTER TABLE download_tasks ADD COLUMN file_hash TEXT;
ALTER TABLE download_tasks ADD COLUMN file_size_bytes INTEGER;
ALTER TABLE download_tasks ADD COLUMN url_hash TEXT;

-- Add indexes for fast duplicate detection
CREATE INDEX idx_file_hash ON download_tasks(file_hash);
CREATE INDEX idx_url_hash ON download_tasks(url_hash);
CREATE INDEX idx_url_target ON download_tasks(url_hash, target_path);
```

### 2. Core Services

**DuplicateDetector Service:**
- Check for existing tasks by URL hash
- Check for existing tasks by file hash (when available)
- Return existing task_id for duplicates
- Support various duplicate resolution policies

**Background Hash Calculator:**
- Calculate file hashes for completed downloads
- Update task records with computed hashes
- Use async background processing to avoid blocking downloads

### 3. API Integration

**Extend DownloadManager trait:**
```rust
async fn find_duplicate_task(&self, url: &str, target_path: &Path) -> Result<Option<TaskId>>;
async fn add_download_with_duplicate_check(&self, url: &str, target_path: &Path) -> Result<TaskId>;
```

## Performance Considerations

### Duplicate Detection Performance

**Fast Path (URL-based):**
- URL normalization + hashing: ~1ms
- Database index lookup: ~1ms
- **Total**: <5ms (well under 1-second requirement)

**Slow Path (File content-based):**
- File hash calculation: depends on file size
- Use background processing for large files
- Cache computed hashes in database

### Database Performance

**Index Strategy:**
- Primary index on `url_hash` for fast URL duplicate detection
- Secondary index on `file_hash` for content-based detection
- Composite index on `(url_hash, target_path)` for exact duplicate detection

**Query Optimization:**
- Use prepared statements for duplicate detection queries
- Leverage SQLite FTS for fuzzy filename matching if needed
- Keep duplicate detection queries simple and indexed

## Alternatives Considered

### Alternative 1: In-Memory Duplicate Tracking
**Rejected**: Doesn't persist across application restarts, doesn't leverage existing database infrastructure.

### Alternative 2: External Deduplication Service
**Rejected**: Adds complexity, requires additional dependencies, violates "simple" architecture principle.

### Alternative 3: File-system Based Detection
**Rejected**: Platform-dependent, doesn't handle different target paths, performance issues with large directories.

## Risk Assessment

### Low Risk
- **Database Schema Changes**: Additive only, backward compatible
- **Dependency Additions**: Minimal, well-established crates
- **Performance Impact**: Duplicate detection adds <5ms overhead

### Mitigation Strategies
- **Schema Migration**: Use SQLx migrations for safe database updates
- **Backward Compatibility**: New fields nullable, graceful degradation
- **Performance Monitoring**: Add metrics for duplicate detection timing

## Success Criteria Validation

**SC-001**: Users receive existing download task_id within 1 second
- ✅ **Achievable**: URL-based duplicate detection <5ms

**SC-002**: System reduces redundant download bandwidth usage by 80%
- ✅ **Achievable**: Perfect duplicate detection prevents 100% redundant downloads

**SC-003**: 95% of resumed downloads continue from previous progress
- ✅ **Achievable**: Existing download manager already supports resume

**SC-004**: Zero new task creation when valid existing task exists
- ✅ **Achievable**: Duplicate detection prevents new task creation

**SC-005**: 99% successful resume rate when file source remains available
- ✅ **Achievable**: Current architecture already provides this capability

## Next Steps (Phase 1)

1. Create data model with duplicate detection fields
2. Generate API contracts for duplicate detection methods
3. Create quickstart guide for duplicate detection usage
4. Update agent context with resolved technical details