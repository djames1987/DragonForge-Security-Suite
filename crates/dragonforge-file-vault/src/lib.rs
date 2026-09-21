#![forbid(unsafe_code)]

//! Product-owned encrypted container engine for DragonForge File Vault.
//!
//! This crate intentionally uses a format distinct from DragonForge Password
//! Manager vaults. File Vault containers encrypt filenames, directory layout,
//! and file bytes as one authenticated payload.

mod container;
mod error;
mod format;

pub use container::{
    ExtractSummary, VaultEntryInfo, VaultSummary, create_vault, extract_vault, list_vault,
    verify_vault,
};
pub use error::{FileVaultError, Result};
pub use format::{FILE_VAULT_EXTENSION, FORMAT_VERSION};
