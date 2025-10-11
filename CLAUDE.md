# burncloud-download Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-10-09

## Active Technologies
- Rust 1.75 (per user requirement: "使用rust编程，不要写其它语言代码") + NEEDS CLARIFICATION - current codebase dependencies for HTTP client, async runtime, serialization (001-burncloud-download-task)
- Rust 1.75+ (per user requirement: "使用rust开发，勿用其它语言") + tokio (async runtime), burncloud-database-download (database layer), burncloud-download-aria2 (download engine), blake3 (hashing), url (URL handling) (002-url-bug)
- SQLite database via burncloud-database-download crate for task persistence (002-url-bug)

## Project Structure
```
src/
tests/
```

## Commands
cargo test; cargo clippy

## Code Style
Rust 1.75 (per user requirement: "使用rust编程，不要写其它语言代码"): Follow standard conventions

## Recent Changes
- 002-url-bug: Added Rust 1.75+ (per user requirement: "使用rust开发，勿用其它语言") + tokio (async runtime), burncloud-database-download (database layer), burncloud-download-aria2 (download engine), blake3 (hashing), url (URL handling)
- 001-burncloud-download-task: Added Rust 1.75 (per user requirement: "使用rust编程，不要写其它语言代码") + NEEDS CLARIFICATION - current codebase dependencies for HTTP client, async runtime, serialization

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
