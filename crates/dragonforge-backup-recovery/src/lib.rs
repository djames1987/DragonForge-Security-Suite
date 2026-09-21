#![forbid(unsafe_code)]

//! Product-owned encrypted backup/recovery engine for DragonForge Security Suite.
//!
//! The .dfbackup format is independent from Password Manager and File Vault formats.
//! It encrypts source metadata, relative paths, per-entry hashes, and file contents.

mod container;
mod error;
mod format;

pub use container::{
    BackupSelectionSummary, BackupSummary, DiscoveredSource, RestoreSummary, create_backup,
    discover_suite_sources, inspect_backup, restore_backup, verify_backup,
};
pub use error::{BackupError, Result};
pub use format::{BACKUP_EXTENSION, FORMAT_VERSION};
