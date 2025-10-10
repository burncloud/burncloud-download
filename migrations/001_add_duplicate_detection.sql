-- Migration: Add duplicate detection fields to download_tasks table
-- Date: 2025-10-09
-- Feature: 001-burncloud-download-task

BEGIN TRANSACTION;

-- Add new columns for duplicate detection (nullable for backward compatibility)
ALTER TABLE download_tasks ADD COLUMN file_hash TEXT;
ALTER TABLE download_tasks ADD COLUMN file_size_bytes INTEGER;
ALTER TABLE download_tasks ADD COLUMN url_hash TEXT;
ALTER TABLE download_tasks ADD COLUMN last_verified_at TIMESTAMP;

-- Create indexes for fast duplicate detection
CREATE INDEX IF NOT EXISTS idx_file_hash ON download_tasks(file_hash) WHERE file_hash IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_url_hash ON download_tasks(url_hash);
CREATE INDEX IF NOT EXISTS idx_url_target ON download_tasks(url_hash, target_path);
CREATE INDEX IF NOT EXISTS idx_status_updated ON download_tasks(status, updated_at);

-- Note: url_hash population will be handled by application code
-- when tasks are loaded/created to avoid potential function dependency issues

COMMIT;