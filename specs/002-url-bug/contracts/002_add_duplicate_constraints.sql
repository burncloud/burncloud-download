-- Migration 002: Add Duplicate Prevention Constraints
-- Feature: Database Duplicate Records and URL Recording Bug Fix
-- Date: 2025-10-10
-- Purpose: Add unique constraints and indexes to prevent duplicate downloads

-- Step 1: Create unique index to prevent URL+path duplicates
-- This is the primary constraint that prevents the bug from occurring
CREATE UNIQUE INDEX idx_url_hash_path_unique
ON download_tasks(url_hash, target_path);

-- Step 2: Add efficient lookup index for duplicate detection queries
-- Used by DuplicateDetector service for finding existing downloads
CREATE INDEX idx_url_hash_lookup
ON download_tasks(url_hash);

-- Step 3: Add index for status-based queries (maintain existing performance)
-- Partial index only on non-completed tasks for efficiency
CREATE INDEX idx_status_lookup
ON download_tasks(status)
WHERE status != 'Completed';

-- Step 4: Add index for cleanup operations (find completed tasks)
CREATE INDEX idx_completed_tasks
ON download_tasks(updated_at)
WHERE status = 'Completed';

-- Expected outcome:
-- - UNIQUE constraint prevents duplicate (url_hash, target_path) combinations
-- - Fast duplicate detection queries via url_hash index
-- - Maintained performance for status-based operations
-- - Database-level guarantee against duplicate downloads