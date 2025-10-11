//! URL normalization and hashing utilities for duplicate detection
//!
//! This module provides comprehensive URL normalization functionality to ensure
//! consistent duplicate detection across different URL formats. It implements
//! the normalization strategy defined in the research phase.

use blake3;
use url::Url;
use anyhow::{Result, Context};

/// Comprehensive URL normalization for duplicate detection
///
/// This function implements the normalization strategy defined in research.md
/// and must be used consistently across all duplicate detection operations.
///
/// Normalization steps:
/// - Remove URL fragments (#section)
/// - Remove default ports (:80 for HTTP, :443 for HTTPS)
/// - Sort query parameters for consistent ordering
/// - Preserve scheme, host, and path exactly as parsed by url crate
///
/// # Arguments
/// * `input_url` - The raw URL string to normalize
///
/// # Returns
/// * `Result<String>` - The normalized URL string
///
/// # Examples
/// ```
/// use burncloud_download::utils::url_normalization::normalize_url;
///
/// let normalized = normalize_url("https://example.com/file.zip#section")?;
/// assert_eq!(normalized, "https://example.com/file.zip");
/// ```
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
///
/// Used for efficient duplicate detection and database indexing.
/// The hash format is consistent across all operations.
///
/// # Arguments
/// * `normalized_url` - The normalized URL string to hash
///
/// # Returns
/// * `String` - 64-character Blake3 hex string
///
/// # Examples
/// ```
/// use burncloud_download::utils::url_normalization::hash_normalized_url;
///
/// let hash = hash_normalized_url("https://example.com/file.zip");
/// assert_eq!(hash.len(), 64);
/// ```
pub fn hash_normalized_url(normalized_url: &str) -> String {
    blake3::hash(normalized_url.as_bytes()).to_hex().to_string()
}

/// Complete URL processing: normalize and hash in one operation
///
/// This is the primary function used throughout the application
/// for processing URLs before storage or duplicate detection.
///
/// # Arguments
/// * `input_url` - The raw URL string to process
///
/// # Returns
/// * `Result<(String, String)>` - Tuple of (normalized_url, url_hash)
///
/// # Examples
/// ```
/// use burncloud_download::utils::url_normalization::process_url_for_storage;
///
/// let (normalized, hash) = process_url_for_storage("https://example.com/file.zip#section")?;
/// assert_eq!(normalized, "https://example.com/file.zip");
/// assert_eq!(hash.len(), 64);
/// ```
pub fn process_url_for_storage(input_url: &str) -> Result<(String, String)> {
    let normalized = normalize_url(input_url)?;
    let hash = hash_normalized_url(&normalized);
    Ok((normalized, hash))
}

/// Validate that a URL hash has the correct Blake3 format
///
/// # Arguments
/// * `hash` - The hash string to validate
///
/// # Returns
/// * `bool` - true if hash is valid Blake3 hex format (64 lowercase hex characters)
pub fn is_valid_url_hash(hash: &str) -> bool {
    hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit() && (c.is_ascii_lowercase() || c.is_ascii_digit()))
}