// Rust Migration Helper: URL Normalization and Hash Population
// Feature: Database Duplicate Records and URL Recording Bug Fix
// Date: 2025-10-10
// Purpose: Provide Rust code for populating url_hash during migration

use blake3;
use url::Url;
use anyhow::Result;

/// Comprehensive URL normalization for duplicate detection
///
/// This function implements the normalization strategy defined in research.md
/// and must be used consistently across all duplicate detection operations.
pub fn normalize_url(input_url: &str) -> Result<String> {
    let mut parsed = Url::parse(input_url)?;

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
///
/// Used for efficient duplicate detection and database indexing.
/// The hash format is consistent across all operations.
pub fn hash_normalized_url(normalized_url: &str) -> String {
    blake3::hash(normalized_url.as_bytes()).to_hex().to_string()
}

/// Complete URL processing: normalize and hash in one operation
///
/// This is the primary function used throughout the application
/// for processing URLs before storage or duplicate detection.
pub fn process_url_for_storage(input_url: &str) -> Result<(String, String)> {
    let normalized = normalize_url(input_url)?;
    let hash = hash_normalized_url(&normalized);
    Ok((normalized, hash))
}

/// Migration function to populate url_hash for existing records
///
/// This function should be called during database migration to populate
/// the url_hash column for all existing download_tasks records.
pub async fn populate_url_hashes(
    connection: &mut sqlx::SqliteConnection,
) -> Result<usize> {
    use sqlx::Row;

    // Fetch all records that need url_hash population
    let records = sqlx::query("SELECT id, url FROM download_tasks WHERE url_hash IS NULL")
        .fetch_all(&mut *connection)
        .await?;

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
///
/// Called after migration to ensure data integrity.
pub async fn validate_url_hash_migration(
    connection: &mut sqlx::SqliteConnection,
) -> Result<bool> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_normalization() {
        // Test basic normalization
        assert_eq!(
            normalize_url("https://example.com/file.zip#section").unwrap(),
            "https://example.com/file.zip"
        );

        // Test query parameter sorting
        assert_eq!(
            normalize_url("https://example.com/file.zip?b=2&a=1").unwrap(),
            "https://example.com/file.zip?a=1&b=2"
        );

        // Test default port removal
        assert_eq!(
            normalize_url("https://example.com:443/file.zip").unwrap(),
            "https://example.com/file.zip"
        );
    }

    #[test]
    fn test_hash_consistency() {
        let url = "https://example.com/file.zip";
        let hash1 = hash_normalized_url(url);
        let hash2 = hash_normalized_url(url);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);  // Blake3 hex length
    }
}