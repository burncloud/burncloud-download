-- Base schema for download_tasks table
-- This creates the fundamental table structure for download management

CREATE TABLE IF NOT EXISTS download_tasks (
    id TEXT PRIMARY KEY NOT NULL,
    url TEXT NOT NULL,
    target_path TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'Waiting',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    file_size INTEGER,
    downloaded_bytes INTEGER NOT NULL DEFAULT 0,

    -- These columns may have been added by previous migrations
    file_hash TEXT,
    file_size_bytes INTEGER,
    url_hash TEXT,
    last_verified_at TIMESTAMP
);

-- Create basic indexes for performance
CREATE INDEX IF NOT EXISTS idx_status ON download_tasks(status);
CREATE INDEX IF NOT EXISTS idx_created_at ON download_tasks(created_at);
CREATE INDEX IF NOT EXISTS idx_updated_at ON download_tasks(updated_at);