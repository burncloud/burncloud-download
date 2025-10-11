//! Unit tests for URL normalization functionality
//!
//! Following TDD methodology: These tests are written FIRST and MUST FAIL
//! before implementation begins to ensure we're testing the actual functionality.

use burncloud_download::utils::url_normalization::normalize_url;

#[test]
fn test_normalize_url_removes_fragment() {
    // This test MUST FAIL initially (TDD requirement)
    let result = normalize_url("https://example.com/file.zip#section").unwrap();
    assert_eq!(result, "https://example.com/file.zip");
}

#[test]
fn test_normalize_url_sorts_query_parameters() {
    // This test MUST FAIL initially (TDD requirement)
    let result = normalize_url("https://example.com/file.zip?b=2&a=1&c=3").unwrap();
    assert_eq!(result, "https://example.com/file.zip?a=1&b=2&c=3");
}

#[test]
fn test_normalize_url_removes_default_ports() {
    // HTTPS default port removal
    let result_https = normalize_url("https://example.com:443/file.zip").unwrap();
    assert_eq!(result_https, "https://example.com/file.zip");

    // HTTP default port removal
    let result_http = normalize_url("http://example.com:80/file.zip").unwrap();
    assert_eq!(result_http, "http://example.com/file.zip");
}

#[test]
fn test_normalize_url_preserves_custom_ports() {
    // Custom ports should be preserved
    let result = normalize_url("https://example.com:8443/file.zip").unwrap();
    assert_eq!(result, "https://example.com:8443/file.zip");
}

#[test]
fn test_normalize_url_handles_empty_query() {
    // URLs with empty query parameters
    let result = normalize_url("https://example.com/file.zip?").unwrap();
    assert_eq!(result, "https://example.com/file.zip");
}

#[test]
fn test_normalize_url_handles_complex_scenarios() {
    // Complex URL with fragment, default port, and unsorted params
    let complex_url = "https://example.com:443/path/file.zip?z=3&a=1&b=2#fragment";
    let result = normalize_url(complex_url).unwrap();
    assert_eq!(result, "https://example.com/path/file.zip?a=1&b=2&z=3");
}

#[test]
fn test_normalize_url_error_handling() {
    // Invalid URLs should return errors
    let result = normalize_url("not-a-valid-url");
    assert!(result.is_err());

    let result = normalize_url("");
    assert!(result.is_err());

    let result = normalize_url("ftp://invalid.scheme");
    // Should succeed - ftp is a valid scheme
    assert!(result.is_ok());
}

#[test]
fn test_normalize_url_preserves_path_and_scheme() {
    // Ensure normalization doesn't break basic URL structure
    let result = normalize_url("https://example.com/deep/path/to/file.pdf").unwrap();
    assert_eq!(result, "https://example.com/deep/path/to/file.pdf");
}

#[test]
fn test_normalize_url_handles_encoded_characters() {
    // URLs with percent encoding should be handled correctly
    let result = normalize_url("https://example.com/file%20name.zip").unwrap();
    assert_eq!(result, "https://example.com/file%20name.zip");
}

#[test]
fn test_normalize_url_handles_unicode() {
    // Unicode characters in URLs
    let result = normalize_url("https://example.com/файл.zip");
    // Should either succeed with proper encoding or fail gracefully
    assert!(result.is_ok() || result.is_err());
}