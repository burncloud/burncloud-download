-- Migration 001: Add URL Hash Column for Duplicate Detection
-- Feature: Database Duplicate Records and URL Recording Bug Fix
-- Date: 2025-10-10
-- Purpose: Add url_hash column to support efficient duplicate detection

-- Step 1: Add url_hash column (nullable initially)
ALTER TABLE download_tasks ADD COLUMN url_hash TEXT;

-- Step 2: Create temporary index for migration performance
CREATE INDEX IF NOT EXISTS tmp_migration_url_index ON download_tasks(url);

-- Step 3: Populate url_hash for existing records
-- Note: This will need to be done programmatically in Rust using the URL normalization logic
-- The SQL here is a placeholder for the migration script structure

-- Update statement will be replaced by Rust migration code:
-- UPDATE download_tasks SET url_hash = normalize_and_hash_url(url) WHERE url_hash IS NULL;

-- Step 4: Make url_hash non-null after population
-- ALTER TABLE download_tasks ALTER COLUMN url_hash SET NOT NULL;

-- Step 5: Add validation constraint for hash format
-- CREATE CONSTRAINT check_url_hash_format CHECK (length(url_hash) = 64);

-- Step 6: Clean up temporary index
DROP INDEX IF EXISTS tmp_migration_url_index;

-- Expected outcome:
-- - All existing download_tasks have populated url_hash column
-- - url_hash contains Blake3 hash of normalized URL
-- - Column is non-null and validates hash format