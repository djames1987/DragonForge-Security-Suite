#![forbid(unsafe_code)]

use std::path::PathBuf;

use dragonforge_core::{Component, SuitePaths};
use dragonforge_integrity_monitor::{
    ComparisonReport, SnapshotSummary, baseline_summary, compare_to_baseline, create_baseline,
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
struct MonitorInfo {
    name: &'static str,
    mode: &'static str,
    platform_scope: &'static str,
    continuous_background_monitoring: bool,
}

#[derive(Debug, Clone, Serialize)]
struct BaselineStatus {
    exists: bool,
    created_at_ms: Option<u64>,
    entries: usize,
    warnings: Vec<String>,
}

fn baseline_path() -> Result<PathBuf, String> {
    SuitePaths::discover()
        .map(|paths| {
            paths
                .component_data_dir(Component::IntegrityMonitor)
                .join("baseline-v1.json")
        })
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn monitor_info() -> MonitorInfo {
    MonitorInfo {
        name: "DragonForge Integrity Monitor",
        mode: "baseline_compare",
        platform_scope: "windows_first",
        continuous_background_monitoring: false,
    }
}

#[tauri::command]
fn get_baseline_status() -> Result<BaselineStatus, String> {
    let path = baseline_path()?;
    let summary = baseline_summary(&path).map_err(|error| error.to_string())?;
    Ok(match summary {
        Some(summary) => BaselineStatus {
            exists: true,
            created_at_ms: Some(summary.timestamp_ms),
            entries: summary.entries,
            warnings: summary.warnings,
        },
        None => BaselineStatus {
            exists: false,
            created_at_ms: None,
            entries: 0,
            warnings: Vec::new(),
        },
    })
}

#[tauri::command]
fn create_integrity_baseline(replace: bool) -> Result<SnapshotSummary, String> {
    let path = baseline_path()?;
    create_baseline(&path, replace).map_err(|error| error.to_string())
}

#[tauri::command]
fn check_integrity() -> Result<ComparisonReport, String> {
    let path = baseline_path()?;
    compare_to_baseline(&path).map_err(|error| error.to_string())
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            monitor_info,
            get_baseline_status,
            create_integrity_baseline,
            check_integrity
        ])
        .run(tauri::generate_context!())
        .expect("error while running DragonForge Integrity Monitor");
}

#[cfg(test)]
mod tests {
    use super::monitor_info;

    #[test]
    fn phase7_monitor_does_not_claim_background_agent_monitoring() {
        let info = monitor_info();
        assert_eq!(info.mode, "baseline_compare");
        assert!(!info.continuous_background_monitoring);
    }
}
