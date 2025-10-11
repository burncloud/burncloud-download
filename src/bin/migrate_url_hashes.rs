//! Database migration runner for URL hash population
//!
//! This binary populates the url_hash column for all existing download_tasks
//! records using the comprehensive URL normalization logic.

use anyhow::{Result, Context};
use sqlx::{sqlite::SqlitePool, Row};
use std::env;

// Include the migration helper functions
mod migration_helpers {
    use super::*;
    use url::Url;

    /// Comprehensive URL normalization for duplicate detection
    pub fn normalize_url(input_url: &str) -> Result<String> {
        let mut parsed = Url::parse(input_url)
            .with_context(|| format!("Failed to parse URL: {}", input_url))?;

        // Remove fragment (everything after #)
        parsed.set_fragment(None);

        // Remove default ports
        if (parsed.scheme() == "http" && parsed.port() == Some(80)) ||
           (parsed.scheme() == "https" && parsed.port() == Some(443)) {
            let _ = parsed.set_port(None);
        }

        // Sort query parameters for consistent ordering
        if parsed.query().is_some() {
            let mut params: Vec<(String, String)> = parsed
                .query_pairs()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
            params.sort();

            if !params.is_empty() {
                let sorted_query = params
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("&");
                parsed.set_query(Some(&sorted_query));
            } else {
                parsed.set_query(None);
            }
        }

        Ok(parsed.to_string())
    }

    /// Generate Blake3 hash of normalized URL
    pub fn hash_normalized_url(normalized_url: &str) -> String {
        blake3::hash(normalized_url.as_bytes()).to_hex().to_string()
    }

    /// Complete URL processing: normalize and hash in one operation
    pub fn process_url_for_storage(input_url: &str) -> Result<(String, String)> {
        let normalized = normalize_url(input_url)?;
        let hash = hash_normalized_url(&normalized);
        Ok((normalized, hash))
    }

    /// Migration function to populate url_hash for existing records
    pub async fn populate_url_hashes(
        pool: &SqlitePool,
    ) -> Result<usize> {
        let mut connection = pool.acquire().await?;

        // Fetch all records that need url_hash population
        let records = sqlx::query("SELECT id, url FROM download_tasks WHERE url_hash IS NULL")
            .fetch_all(&mut *connection)
            .await?;

        println!("Found {} records to update", records.len());

        let mut updated_count = 0;

        for record in records {
            let id: i64 = record.get("id");
            let url: String = record.get("url");

            // Process URL to get hash
            match process_url_for_storage(&url) {
                Ok((normalized_url, url_hash)) => {
                    // Update record with computed hash
                    sqlx::query(
                        "UPDATE download_tasks SET url_hash = ?, url = ? WHERE id = ?"
                    )
                    .bind(&url_hash)
                    .bind(&normalized_url)  // Also update URL to normalized version
                    .bind(id)
                    .execute(&mut *connection)
                    .await?;

                    updated_count += 1;
                    if updated_count % 100 == 0 {
                        println!("Updated {} records...", updated_count);
                    }
                }
                Err(e) => {
                    // Log error but continue migration
                    eprintln!("Failed to process URL for record {}: {} - Error: {}", id, url, e);
                    // Optionally mark record for manual review
                }
            }
        }

        Ok(updated_count)
    }

    /// Validate that all records have proper url_hash values
    pub async fn validate_url_hash_migration(
        pool: &SqlitePool,
    ) -> Result<bool> {
        let mut connection = pool.acquire().await?;

        // Check for any records missing url_hash
        let missing_hash_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM download_tasks WHERE url_hash IS NULL OR url_hash = ''"
        )
        .fetch_one(&mut *connection)
        .await?;

        if missing_hash_count > 0 {
            eprintln!("Warning: {} records still missing url_hash", missing_hash_count);
            return Ok(false);
        }

        // Check for invalid hash format
        let invalid_hash_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM download_tasks WHERE length(url_hash) != 64"
        )
        .fetch_one(&mut *connection)
        .await?;

        if invalid_hash_count > 0 {
            eprintln!("Warning: {} records have invalid url_hash format", invalid_hash_count);
            return Ok(false);
        }

        println!("✅ URL hash migration validation passed");
        Ok(true)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔄 Starting URL hash migration...");

    // Get database URL from environment or use default
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./data/burncloud.db".to_string());

    println!("📊 Connecting to database: {}", database_url);

    // Ensure the data directory exists before connecting
    let db_path = database_url.replace("sqlite:", "");
    println!("📁 Database path: {}", db_path);

    if let Some(parent) = std::path::Path::new(&db_path).parent() {
        println!("📁 Creating directory: {:?}", parent);
        tokio::fs::create_dir_all(parent)
            .await
            .context("Failed to create data directory")?;
        println!("✅ Directory created");
    }

    // Create database pool
    let pool = SqlitePool::connect(&database_url)
        .await
        .context("Failed to connect to database")?;

    // Initialize base schema if needed
    println!("🔧 Initializing database schema...");
    let mut connection = pool.acquire().await?;

    // Create base download_tasks table if it doesn't exist
    sqlx::query(r#"
        CREATE TABLE IF NOT EXISTS download_tasks (
            id TEXT PRIMARY KEY NOT NULL,
            url TEXT NOT NULL,
            target_path TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'Waiting',
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            file_size INTEGER,
            downloaded_bytes INTEGER NOT NULL DEFAULT 0,

            -- Duplicate detection columns (may already exist from migrations)
            file_hash TEXT,
            file_size_bytes INTEGER,
            url_hash TEXT,
            last_verified_at TIMESTAMP
        )
    "#)
    .execute(&mut *connection)
    .await
    .context("Failed to create download_tasks table")?;

    // Create indexes for duplicate detection and performance
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_status ON download_tasks(status)")
        .execute(&mut *connection)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_url_hash ON download_tasks(url_hash)")
        .execute(&mut *connection)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_url_target ON download_tasks(url_hash, target_path)")
        .execute(&mut *connection)
        .await?;

    // Add the critical UNIQUE constraint to prevent duplicates
    println!("🔐 Adding unique constraint for duplicate prevention...");
    match sqlx::query("CREATE UNIQUE INDEX idx_url_hash_path_unique ON download_tasks(url_hash, target_path)")
        .execute(&mut *connection)
        .await
    {
        Ok(_) => println!("✅ Unique constraint added successfully"),
        Err(e) if e.to_string().contains("already exists") => {
            println!("✅ Unique constraint already exists");
        }
        Err(e) => {
            eprintln!("⚠️  Warning: Failed to add unique constraint: {}", e);
            eprintln!("    This may allow duplicate records if not handled by application logic");
        }
    }

    drop(connection);
    println!("✅ Database schema initialized");

    // Run migration
    println!("🔍 Searching for records needing URL hash population...");
    let updated_count = migration_helpers::populate_url_hashes(&pool)
        .await
        .context("Failed to populate URL hashes")?;

    println!("✅ Updated {} records with URL hashes", updated_count);

    // Validate migration
    println!("🔍 Validating migration results...");
    let validation_passed = migration_helpers::validate_url_hash_migration(&pool)
        .await
        .context("Failed to validate migration")?;

    if validation_passed {
        println!("🎉 URL hash migration completed successfully!");
    } else {
        eprintln!("❌ URL hash migration validation failed - manual review required");
        std::process::exit(1);
    }

    pool.close().await;
    Ok(())
}