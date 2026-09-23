#![forbid(unsafe_code)]

use std::path::PathBuf;

use dragonforge_core::{Component, SuitePaths};
use dragonforge_integrity_monitor::{
    ComparisonReport, ContinuousIntegrityEvent, ContinuousMonitorPolicy, ContinuousMonitorStatus,
    SnapshotSummary, SuppressionRule, baseline_summary, compare_to_baseline,
    configure_continuous_monitoring, continuous_events, continuous_status, create_baseline,
    reseal_continuous_baseline,
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

fn continuous_state_path() -> Result<PathBuf, String> {
    SuitePaths::discover()
        .map(|paths| {
            paths
                .component_data_dir(Component::Agent)
                .join("continuous-integrity-v1.json")
        })
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn monitor_info() -> MonitorInfo {
    MonitorInfo {
        name: "DragonForge Integrity Monitor",
        mode: "baseline_compare",
        platform_scope: "windows_first",
        continuous_background_monitoring: true,
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
    let summary = create_baseline(&path, replace).map_err(|error| error.to_string())?;
    let state = continuous_state_path()?;
    reseal_continuous_baseline(&state, &path).map_err(|error| error.to_string())?;
    Ok(summary)
}

#[tauri::command]
fn check_integrity() -> Result<ComparisonReport, String> {
    let path = baseline_path()?;
    compare_to_baseline(&path).map_err(|error| error.to_string())
}

#[tauri::command]
fn get_continuous_monitor_status() -> Result<ContinuousMonitorStatus, String> {
    let state = continuous_state_path()?;
    let baseline = baseline_path()?;
    continuous_status(&state, &baseline).map_err(|error| error.to_string())
}

#[tauri::command]
fn configure_continuous_integrity(
    enabled: bool,
    interval_seconds: u64,
    suppressions: Vec<SuppressionRule>,
) -> Result<ContinuousMonitorStatus, String> {
    let state = continuous_state_path()?;
    let baseline = baseline_path()?;
    configure_continuous_monitoring(
        &state,
        &baseline,
        ContinuousMonitorPolicy {
            enabled,
            interval_seconds,
            suppressions,
        },
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn get_continuous_integrity_events(
    after_id: u64,
    limit: usize,
) -> Result<Vec<ContinuousIntegrityEvent>, String> {
    let state = continuous_state_path()?;
    continuous_events(&state, after_id, limit).map_err(|error| error.to_string())
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            monitor_info,
            get_baseline_status,
            create_integrity_baseline,
            check_integrity,
            get_continuous_monitor_status,
            configure_continuous_integrity,
            get_continuous_integrity_events
        ])
        .run(tauri::generate_context!())
        .expect("error while running DragonForge Integrity Monitor");
}

#[cfg(test)]
mod tests {
    use super::monitor_info;

    #[test]
    fn phase18_monitor_reports_background_agent_monitoring() {
        let info = monitor_info();
        assert_eq!(info.mode, "baseline_compare");
        assert!(info.continuous_background_monitoring);
    }
}
