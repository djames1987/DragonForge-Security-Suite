#![forbid(unsafe_code)]

//! Product-owned encrypted backup/recovery engine for DragonForge Security Suite.
//!
//! The .dfbackup format is independent from Password Manager and File Vault formats.
//! It encrypts source metadata, relative paths, per-entry hashes, and file contents.

mod container;
mod error;
mod format;
mod recovery;

pub use container::{
    BackupSelectionSummary, BackupSummary, DiscoveredSource, RestoreSummary, create_backup,
    discover_suite_sources, inspect_backup, restore_backup, verify_backup,
};
pub use error::{BackupError, Result};
pub use format::{
    BACKUP_EXTENSION, FORMAT_VERSION, RECOVERY_EXTENSION, RECOVERY_FORMAT_VERSION,
    RECOVERY_SCHEMA_VERSION,
};
pub use recovery::{
    RecoveryRestoreSummary, RecoveryScope, RecoverySummary, RepairSummary, create_suite_recovery,
    create_suite_recovery_from_paths, inspect_suite_recovery, repair_recoverable_json_state,
    restore_suite_recovery, verify_suite_recovery,
};
