//! Unit tests for Blake3 hash calculation functionality
//!
//! Following TDD methodology: These tests are written FIRST and MUST FAIL
//! before implementation begins to ensure we're testing the actual functionality.

use burncloud_download::utils::url_normalization::{
    hash_normalized_url, is_valid_url_hash, process_url_for_storage
};

#[test]
fn test_hash_normalized_url_produces_blake3_hash() {
    // This test MUST FAIL initially (TDD requirement)
    let normalized_url = "https://example.com/file.zip";
    let hash = hash_normalized_url(normalized_url);

    // Blake3 hash should be 64 characters (32 bytes in hex)
    assert_eq!(hash.len(), 64);

    // Should contain only hex characters
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

    // Should be lowercase hex
    assert!(hash.chars().all(|c| !c.is_ascii_uppercase()));
}

#[test]
fn test_hash_normalized_url_consistent_output() {
    // Same input should always produce same hash
    let normalized_url = "https://example.com/file.zip";

    let hash1 = hash_normalized_url(normalized_url);
    let hash2 = hash_normalized_url(normalized_url);

    assert_eq!(hash1, hash2);
}

#[test]
fn test_hash_normalized_url_different_inputs_different_hashes() {
    // Different URLs should produce different hashes
    let url1 = "https://example.com/file1.zip";
    let url2 = "https://example.com/file2.zip";

    let hash1 = hash_normalized_url(url1);
    let hash2 = hash_normalized_url(url2);

    assert_ne!(hash1, hash2);
}

#[test]
fn test_hash_normalized_url_handles_empty_string() {
    // Empty string should produce a valid hash
    let hash = hash_normalized_url("");
    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_hash_normalized_url_handles_special_characters() {
    // URLs with special characters
    let normalized_url = "https://example.com/file%20name.zip?param=value&other=123";
    let hash = hash_normalized_url(normalized_url);

    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_is_valid_url_hash_accepts_valid_blake3_hash() {
    // This test MUST FAIL initially (TDD requirement)
    let valid_hash = "a".repeat(64); // 64 character lowercase hex string
    assert!(is_valid_url_hash(&valid_hash));
}

#[test]
fn test_is_valid_url_hash_rejects_invalid_length() {
    // Too short
    let short_hash = "a".repeat(63);
    assert!(!is_valid_url_hash(&short_hash));

    // Too long
    let long_hash = "a".repeat(65);
    assert!(!is_valid_url_hash(&long_hash));

    // Empty
    assert!(!is_valid_url_hash(""));
}

#[test]
fn test_is_valid_url_hash_rejects_non_hex_characters() {
    // Contains non-hex characters
    let invalid_hash = "g".repeat(64);
    assert!(!is_valid_url_hash(&invalid_hash));

    // Contains spaces
    let space_hash = format!("{} {}", "a".repeat(31), "b".repeat(32));
    assert!(!is_valid_url_hash(&space_hash));

    // Contains uppercase (should be lowercase only)
    let upper_hash = "A".repeat(64);
    assert!(!is_valid_url_hash(&upper_hash));
}

#[test]
fn test_process_url_for_storage_integration() {
    // This test MUST FAIL initially (TDD requirement)
    let raw_url = "https://example.com:443/file.zip?b=2&a=1#fragment";
    let (normalized_url, url_hash) = process_url_for_storage(raw_url).unwrap();

    // Should normalize the URL
    assert_eq!(normalized_url, "https://example.com/file.zip?a=1&b=2");

    // Should produce valid hash
    assert!(is_valid_url_hash(&url_hash));

    // Hash should be deterministic
    let (_, url_hash2) = process_url_for_storage(raw_url).unwrap();
    assert_eq!(url_hash, url_hash2);
}

#[test]
fn test_process_url_for_storage_error_handling() {
    // Invalid URLs should return errors
    let result = process_url_for_storage("not-a-valid-url");
    assert!(result.is_err());

    let result = process_url_for_storage("");
    assert!(result.is_err());
}

#[test]
fn test_blake3_hash_collision_resistance() {
    // Test that similar URLs produce different hashes
    let urls = vec![
        "https://example.com/file.zip",
        "https://example.com/file.zip?v=1",
        "https://example.com/file.zip?v=2",
        "https://example.com/File.zip",
        "https://example.org/file.zip",
    ];

    let mut hashes = std::collections::HashSet::new();

    for url in urls {
        let hash = hash_normalized_url(url);
        assert!(hashes.insert(hash), "Hash collision detected for URL: {}", url);
    }
}

#[test]
fn test_hash_normalized_url_performance() {
    // Ensure hash computation is fast enough
    let normalized_url = "https://example.com/very-long-file-name-with-many-parameters.zip?param1=value1&param2=value2&param3=value3";

    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = hash_normalized_url(normalized_url);
    }
    let duration = start.elapsed();

    // Should complete 1000 hashes in reasonable time (less than 100ms)
    assert!(duration.as_millis() < 100, "Hash computation too slow: {:?}", duration);
}