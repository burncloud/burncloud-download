# Feature Specification: Database Duplicate Records and URL Recording Bug Fix

**Feature Branch**: `002-url-bug`
**Created**: 2025-10-10
**Status**: Draft
**Input**: User description: "现在在数据库里面还是出现重复的纪录，而且url也没有正确记录到数据库，需要解决这个bug"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Eliminate Duplicate Download Records (Priority: P1) 🎯 Critical

When users initiate downloads, the system should maintain data integrity by ensuring each unique download (same URL + path) exists only once in the database, preventing data corruption and inconsistent application state.

**Why this priority**: Data integrity is critical for application reliability. Duplicate records can cause inconsistent behavior, incorrect progress tracking, and confusion about which task is the "real" one.

**Independent Test**: Can be fully tested by initiating the same download multiple times and verifying only one database record exists, delivering clean data storage and preventing data corruption.

**Acceptance Scenarios**:

1. **Given** a clean database, **When** user starts download of "https://example.com/file.zip" to "/downloads/file.zip", **Then** exactly one record is created in the database
2. **Given** an existing download record, **When** user attempts to download the same URL to the same path, **Then** no new database record is created and existing record is reused
3. **Given** multiple concurrent requests for the same download, **When** all requests are processed simultaneously, **Then** only one database record exists for that download

---

### User Story 2 - Accurate URL Storage in Database (Priority: P1) 🎯 Critical

When users initiate downloads, the complete and correct source URL must be stored accurately in the database to enable proper duplicate detection, task recovery, and download resumption.

**Why this priority**: Incorrect URL storage breaks the entire duplicate detection system and prevents successful download resumption. This is fundamental data corruption.

**Independent Test**: Can be fully tested by initiating downloads with various URL formats and verifying the complete URL is stored correctly in the database, delivering reliable task persistence.

**Acceptance Scenarios**:

1. **Given** a download request with URL "https://example.com/path/file.zip?param=value", **When** the download is initiated, **Then** the complete URL including query parameters is stored in the database url field
2. **Given** a download request with special characters in URL, **When** the download is initiated, **Then** the URL is stored without corruption or truncation
3. **Given** a download with redirected URL, **When** the download is processed, **Then** the final resolved URL is stored in the database

---

### User Story 3 - Robust Database Transaction Handling (Priority: P2)

When downloads are initiated, all database operations should be properly coordinated to prevent race conditions and ensure atomic operations that maintain data consistency.

**Why this priority**: Proper transaction handling prevents the root cause of duplicate records and ensures the system remains reliable under concurrent usage.

**Independent Test**: Can be fully tested by initiating multiple downloads simultaneously and verifying database consistency is maintained, delivering reliable concurrent operation.

**Acceptance Scenarios**:

1. **Given** multiple simultaneous download requests for different files, **When** all requests are processed concurrently, **Then** each download gets a unique database record without conflicts
2. **Given** a download request that fails during processing, **When** the failure occurs, **Then** no partial or corrupted records remain in the database
3. **Given** a system restart during download initiation, **When** the system recovers, **Then** database state is consistent without orphaned records

---

### Edge Cases

- What happens when the database is locked during duplicate detection queries?
- How does the system handle URL encoding differences (e.g., encoded vs decoded characters)?
- What occurs when database constraints fail during record insertion?
- How does the system behave when URL normalization produces unexpected results?
- What happens when the same URL is requested with different capitalization?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST prevent creation of duplicate database records for downloads with identical URL and target path combinations
- **FR-002**: System MUST store complete and accurate source URLs in the database without truncation or corruption
- **FR-003**: Database operations MUST be atomic to prevent partial record creation during failures
- **FR-004**: System MUST normalize URLs consistently before duplicate detection to handle variations in formatting
- **FR-005**: System MUST handle concurrent download requests without creating race conditions that lead to duplicate records
- **FR-006**: System MUST validate URL storage integrity before marking download initiation as successful
- **FR-007**: Database schema MUST enforce uniqueness constraints on URL+path combinations to prevent duplicates at the database level
- **FR-008**: System MUST properly handle database transaction rollbacks when duplicate detection logic fails

### Key Entities *(include if feature involves data)*

- **DownloadTask**: Database record representing a download operation, must have unique constraint on URL+path combination, stores complete source URL and target path
- **TaskIdentifier**: Composite key used for duplicate detection, combines normalized URL hash with target path to ensure uniqueness

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Zero duplicate database records exist for downloads with identical URL and target path after system corrections
- **SC-002**: 100% of initiated downloads have complete and accurate URLs stored in the database without truncation
- **SC-003**: Duplicate detection operates correctly in 100% of test cases with various URL formats and special characters
- **SC-004**: System maintains data integrity under concurrent load with 0% database record corruption
- **SC-005**: Database query performance for duplicate detection remains under 100ms for 95% of operations

## Scope

### In Scope

- Fix database duplicate record creation for same URL+path combinations
- Ensure complete URL storage without truncation or corruption
- Implement proper transaction handling for download initiation
- Add database constraints to prevent duplicates at schema level
- Validate and fix existing duplicate records in database

### Out of Scope

- Modifying existing duplicate detection algorithm logic (already implemented)
- Changes to user interface or API endpoints
- Performance optimization beyond fixing the core bug
- Migration of historical data beyond duplicate cleanup

## Assumptions

- The existing duplicate detection implementation in Phase 3 provides the correct logic framework
- Database supports proper transaction isolation levels for concurrent operations
- Current URL normalization logic is functionally correct but may have storage issues
- Users expect immediate consistency - no eventual consistency requirements
- System can temporarily block concurrent requests to the same resource during fix implementation

## Dependencies

- Requires access to existing database schema and migration capabilities
- Depends on current duplicate detection models and services implemented in Phase 3
- May require coordination with aria2 integration layer for URL handling
- Database transaction isolation level configuration may need adjustment

## Constraints

- Must maintain backward compatibility with existing download manager interfaces
- Cannot break existing download functionality during fix implementation
- Must work with current SQLite database backend
- Changes should not significantly impact download initiation performance