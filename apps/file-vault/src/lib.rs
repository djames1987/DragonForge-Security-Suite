#![forbid(unsafe_code)]

use std::path::PathBuf;

use dragonforge_file_vault::{
    ExtractSummary, VaultEntryInfo, VaultSummary, create_vault, extract_vault, list_vault,
    verify_vault,
};
use serde::Serialize;
use zeroize::Zeroize;

#[derive(Debug, Serialize)]
struct SummaryDto {
    entries: usize,
    files: usize,
    directories: usize,
    total_file_bytes: u64,
}

impl From<VaultSummary> for SummaryDto {
    fn from(value: VaultSummary) -> Self {
        Self {
            entries: value.entries,
            files: value.files,
            directories: value.directories,
            total_file_bytes: value.total_file_bytes,
        }
    }
}

impl From<ExtractSummary> for SummaryDto {
    fn from(value: ExtractSummary) -> Self {
        Self {
            entries: value.files + value.directories,
            files: value.files,
            directories: value.directories,
            total_file_bytes: value.total_file_bytes,
        }
    }
}

#[derive(Debug, Serialize)]
struct EntryDto {
    path: String,
    is_directory: bool,
    size: u64,
}

impl From<VaultEntryInfo> for EntryDto {
    fn from(value: VaultEntryInfo) -> Self {
        Self {
            path: value.path,
            is_directory: value.is_directory,
            size: value.size,
        }
    }
}

#[tauri::command]
fn create_container(
    output_path: String,
    source_paths: Vec<String>,
    mut password: String,
) -> Result<SummaryDto, String> {
    let output = checked_path(&output_path)?;
    let sources = source_paths
        .iter()
        .map(|value| checked_path(value))
        .collect::<Result<Vec<_>, _>>()?;

    let result = create_vault(output, &sources, password.as_bytes())
        .map(SummaryDto::from)
        .map_err(|error| error.to_string());
    password.zeroize();
    result
}

#[tauri::command]
fn inspect_container(vault_path: String, mut password: String) -> Result<Vec<EntryDto>, String> {
    let vault = checked_path(&vault_path)?;
    let result = list_vault(vault, password.as_bytes())
        .map(|entries| entries.into_iter().map(EntryDto::from).collect())
        .map_err(|error| error.to_string());
    password.zeroize();
    result
}

#[tauri::command]
fn verify_container(vault_path: String, mut password: String) -> Result<SummaryDto, String> {
    let vault = checked_path(&vault_path)?;
    let result = verify_vault(vault, password.as_bytes())
        .map(SummaryDto::from)
        .map_err(|error| error.to_string());
    password.zeroize();
    result
}

#[tauri::command]
fn extract_container(
    vault_path: String,
    destination_path: String,
    mut password: String,
) -> Result<SummaryDto, String> {
    let vault = checked_path(&vault_path)?;
    let destination = checked_path(&destination_path)?;
    let result = extract_vault(vault, destination, password.as_bytes())
        .map(SummaryDto::from)
        .map_err(|error| error.to_string());
    password.zeroize();
    result
}

fn checked_path(value: &str) -> Result<PathBuf, String> {
    let value = value.trim();
    if value.is_empty() || value.contains('\0') {
        return Err("a local file path is required".to_owned());
    }
    Ok(PathBuf::from(value))
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            create_container,
            inspect_container,
            verify_container,
            extract_container
        ])
        .run(tauri::generate_context!())
        .expect("error while running DragonForge File Vault");
}

#[cfg(test)]
mod tests {
    use super::checked_path;

    #[test]
    fn empty_paths_are_rejected() {
        assert!(checked_path("   ").is_err());
    }

    #[test]
    fn ordinary_local_paths_are_accepted() {
        assert!(checked_path(r"C:\Users\Example\file.txt").is_ok());
    }
}
