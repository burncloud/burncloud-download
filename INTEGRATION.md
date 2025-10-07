# BurnCloud Download Integration Guide

## Overview

This document explains the integration between `burncloud-download`, `burncloud-download-aria2`, and `burncloud-database-download` packages.

## Architecture Issue: Cyclic Dependency

The current package structure has an architectural challenge:

```
burncloud-download (core types & traits)
    ↑
    |
    +-- burncloud-download-aria2 (depends on burncloud-download)
    |
    +-- burncloud-database-download (depends on burncloud-download)
```

The `PersistentAria2Manager` implementation (located in `burncloud-download/src/manager/persistent_aria2.rs`) requires both:
- `burncloud-download-aria2::Aria2DownloadManager`
- `burncloud-database-download::DownloadRepository`

However, adding these as dependencies to `burncloud-download` creates a cyclic dependency because `burncloud-database-download` already depends on `burncloud-download` for types.

## Solutions

### Option 1: Separate Integration Crate (Recommended)

Create a new crate `burncloud-download-persistent` that depends on all three packages:

**Directory structure:**
```
burncloud/
├── burncloud-download/          (core types & traits)
├── burncloud-download-aria2/    (aria2 implementation)
├── burncloud-database-download/ (database persistence)
└── burncloud-download-persistent/  (NEW: integration layer)
    ├── Cargo.toml
    └── src/
        └── lib.rs  (re-exports PersistentAria2Manager)
```

**Cargo.toml for burncloud-download-persistent:**
```toml
[package]
name = "burncloud-download-persistent"
version = "0.1.0"
edition = "2021"

[dependencies]
burncloud-download = { path = "../burncloud-download" }
burncloud-download-aria2 = { path = "../burncloud-download-aria2" }
burncloud-database-download = { path = "../burncloud-database-download" }
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
async-trait = "0.1"
log = "0.4"
```

**src/lib.rs:**
```rust
// Re-export the implementation from burncloud-download
pub use burncloud_download::PersistentAria2Manager;

// Re-export commonly used types
pub use burncloud_download::{
    DownloadManager, DownloadTask, DownloadProgress,
    DownloadStatus, TaskId, DownloadError
};
```

**Usage:**
```rust
use burncloud_download_persistent::PersistentAria2Manager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let manager = PersistentAria2Manager::new().await?;

    let task_id = manager.add_download(
        "https://example.com/file.zip".to_string(),
        std::path::PathBuf::from("data/file.zip")
    ).await?;

    println!("Download started: {}", task_id);
    Ok(())
}
```

### Option 2: Modify burncloud-database-download

Make the dependency on `burncloud-download` optional in `burncloud-database-download/Cargo.toml`:

```toml
[dependencies]
burncloud-download = { path = "../burncloud-download", optional = true }

[features]
default = ["types"]
types = ["burncloud-download"]
```

**Note:** This requires modifying `burncloud-database-download`, which violates the current constraints.

### Option 3: Type Extraction

Extract common types into a separate `burncloud-download-types` crate that both packages can depend on:

```
burncloud-download-types/  (NEW: just types)
    ├── TaskId
    ├── DownloadTask
    ├── DownloadProgress
    └── DownloadStatus

burncloud-download/  (traits & managers, depends on types)
burncloud-download-aria2/  (depends on types)
burncloud-database-download/  (depends on types only)
```

**Note:** This requires restructuring all three packages.

## Current Status

The implementation code is complete and located in:
- `burncloud-download/src/manager/persistent_aria2.rs`
- `burncloud-download/src/error/types.rs` (DatabaseError variant added)

However, it **cannot be compiled** within `burncloud-download` due to the cyclic dependency.

## Recommended Next Steps

1. **Immediate Solution**: Create `burncloud-download-persistent` crate as described in Option 1
2. **Long-term Solution**: Consider extracting types into `burncloud-download-types` for cleaner architecture

## Implementation Details

The `PersistentAria2Manager` provides:

- **Automatic Persistence**: Tasks and progress saved to database
- **State Restoration**: Incomplete tasks restored on startup
- **Auto-Resume**: Tasks in "Downloading" state resume automatically
- **Background Polling**: State changes saved every 1s, progress every 5s
- **Graceful Shutdown**: Clean state save on application exit

### Configuration

```rust
const ARIA2_RPC_URL: &str = "http://localhost:6800/jsonrpc";
const ARIA2_RPC_SECRET: &str = "burncloud";
const PROGRESS_SAVE_INTERVAL_SECS: u64 = 5;
const POLL_INTERVAL_SECS: u64 = 1;
```

### API

```rust
// Create manager (initializes database and aria2)
let manager = PersistentAria2Manager::new().await?;

// Add download
let task_id = manager.add_download(url, target_path).await?;

// Control downloads
manager.pause_download(task_id).await?;
manager.resume_download(task_id).await?;
manager.cancel_download(task_id).await?;

// Query state
let progress = manager.get_progress(task_id).await?;
let task = manager.get_task(task_id).await?;
let all_tasks = manager.list_tasks().await?;

// Graceful shutdown
manager.shutdown().await?;
```

## Files Modified

1. `burncloud-download/Cargo.toml` - Added log dependency
2. `burncloud-download/src/error/types.rs` - Added DatabaseError variant
3. `burncloud-download/src/manager/persistent_aria2.rs` - Complete implementation (NEW)
4. `burncloud-download/src/manager/mod.rs` - Export PersistentAria2Manager
5. `burncloud-download/src/lib.rs` - Re-export PersistentAria2Manager

## Testing

Once the cyclic dependency is resolved (via Option 1), tests can be run:

```bash
cd burncloud-download-persistent
cargo test
```

The implementation includes unit tests in `persistent_aria2.rs`:
- `test_manager_creation`
- `test_add_download_persists`

## Questions?

For questions or issues, please refer to the technical specification:
`.claude/specs/burncloud-download-integration/technical-spec.md`
