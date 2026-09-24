#![forbid(unsafe_code)]

use std::path::PathBuf;

use dragonforge_backup_recovery::{
    BackupSummary, DiscoveredSource, RecoveryRestoreSummary, RecoveryScope, RecoverySummary,
    RepairSummary, RestoreSummary, create_backup, create_suite_recovery, discover_suite_sources,
    inspect_backup, inspect_suite_recovery, repair_recoverable_json_state, restore_backup,
    restore_suite_recovery, verify_backup, verify_suite_recovery,
};
use dragonforge_core::SuitePaths;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
struct BackupInfo {
    name: &'static str,
    format_extension: &'static str,
    overwrite_restore: bool,
    stores_passwords: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct CreateBackupRequest {
    source_paths: Vec<String>,
    destination: String,
    password: String,
}

#[derive(Debug, Clone, Deserialize)]
struct BackupPasswordRequest {
    backup_path: String,
    password: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RestoreBackupRequest {
    backup_path: String,
    destination: String,
    password: String,
}

#[derive(Debug, Clone, Deserialize)]
struct CreateRecoveryRequest {
    destination: String,
    password: String,
    scope: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RecoveryPasswordRequest {
    recovery_path: String,
    password: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RestoreRecoveryRequest {
    recovery_path: String,
    password: String,
    config_root: String,
    data_root: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct RecoveryRoots {
    config_root: String,
    data_root: String,
}

#[tauri::command]
fn backup_info() -> BackupInfo {
    BackupInfo {
        name: "DragonForge Backup & Recovery",
        format_extension: ".dfbackup",
        overwrite_restore: false,
        stores_passwords: false,
    }
}

#[tauri::command]
fn suite_sources() -> Result<Vec<DiscoveredSource>, String> {
    discover_suite_sources().map_err(|error| error.to_string())
}

#[tauri::command]
fn create_encrypted_backup(request: CreateBackupRequest) -> Result<BackupSummary, String> {
    let sources = request
        .source_paths
        .iter()
        .map(|value| validated_path(value))
        .collect::<Result<Vec<_>, _>>()?;
    let destination = validated_path(&request.destination)?;
    create_backup(&sources, &destination, &request.password).map_err(|error| error.to_string())
}

#[tauri::command]
fn inspect_encrypted_backup(request: BackupPasswordRequest) -> Result<BackupSummary, String> {
    let path = validated_path(&request.backup_path)?;
    inspect_backup(&path, &request.password).map_err(|error| error.to_string())
}

#[tauri::command]
fn verify_encrypted_backup(request: BackupPasswordRequest) -> Result<BackupSummary, String> {
    let path = validated_path(&request.backup_path)?;
    verify_backup(&path, &request.password).map_err(|error| error.to_string())
}

#[tauri::command]
fn restore_encrypted_backup(request: RestoreBackupRequest) -> Result<RestoreSummary, String> {
    let backup = validated_path(&request.backup_path)?;
    let destination = validated_path(&request.destination)?;
    restore_backup(&backup, &request.password, &destination).map_err(|error| error.to_string())
}

#[tauri::command]
fn suite_recovery_roots() -> Result<RecoveryRoots, String> {
    let paths = SuitePaths::discover().map_err(|error| error.to_string())?;
    Ok(RecoveryRoots {
        config_root: paths.config_root().to_string_lossy().into_owned(),
        data_root: paths.data_root().to_string_lossy().into_owned(),
    })
}

#[tauri::command]
fn create_recovery_package(request: CreateRecoveryRequest) -> Result<RecoverySummary, String> {
    let destination = validated_path(&request.destination)?;
    let scope = match request.scope.as_str() {
        "configuration" => RecoveryScope::Configuration,
        "full_suite" => RecoveryScope::FullSuite,
        _ => return Err("Recovery scope must be configuration or full_suite.".to_owned()),
    };
    create_suite_recovery(&destination, &request.password, scope).map_err(|error| error.to_string())
}

#[tauri::command]
fn inspect_recovery_package(request: RecoveryPasswordRequest) -> Result<RecoverySummary, String> {
    let path = validated_path(&request.recovery_path)?;
    inspect_suite_recovery(&path, &request.password).map_err(|error| error.to_string())
}

#[tauri::command]
fn verify_recovery_package(request: RecoveryPasswordRequest) -> Result<RecoverySummary, String> {
    let path = validated_path(&request.recovery_path)?;
    verify_suite_recovery(&path, &request.password).map_err(|error| error.to_string())
}

#[tauri::command]
fn restore_recovery_package(
    request: RestoreRecoveryRequest,
) -> Result<RecoveryRestoreSummary, String> {
    let package = validated_path(&request.recovery_path)?;
    let config_root = validated_path(&request.config_root)?;
    let data_root = request
        .data_root
        .as_deref()
        .map(validated_path)
        .transpose()?;
    restore_suite_recovery(
        &package,
        &request.password,
        &config_root,
        data_root.as_deref(),
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn repair_suite_state() -> Result<RepairSummary, String> {
    let paths = SuitePaths::discover().map_err(|error| error.to_string())?;
    repair_recoverable_json_state(&paths).map_err(|error| error.to_string())
}

fn validated_path(value: &str) -> Result<PathBuf, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("A required local path is empty.".to_owned());
    }
    if trimmed.chars().any(char::is_control) {
        return Err("A local path contains unsupported control characters.".to_owned());
    }
    Ok(PathBuf::from(trimmed))
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            backup_info,
            suite_sources,
            create_encrypted_backup,
            inspect_encrypted_backup,
            verify_encrypted_backup,
            restore_encrypted_backup,
            suite_recovery_roots,
            create_recovery_package,
            inspect_recovery_package,
            verify_recovery_package,
            restore_recovery_package,
            repair_suite_state
        ])
        .run(tauri::generate_context!())
        .expect("error while running DragonForge Backup & Recovery");
}

#[cfg(test)]
mod tests {
    use super::{backup_info, validated_path};

    #[test]
    fn phase9_refuses_overwrite_restore_and_password_persistence() {
        let info = backup_info();
        assert!(!info.overwrite_restore);
        assert!(!info.stores_passwords);
    }

    #[test]
    fn empty_paths_are_rejected() {
        assert!(validated_path("   ").is_err());
    }
}
