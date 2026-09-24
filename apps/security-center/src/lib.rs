#![forbid(unsafe_code)]

mod agent;
mod diagnostics;
mod events;
mod health_history;
mod logging;
mod model;
mod orchestration;
mod settings;
mod state;
mod update;

use agent::AgentStatus;
use dragonforge_agent::AutomationStatus;
use events::DashboardEvent;
use health_history::HealthHistoryEntry;
use settings::SecurityCenterSettings;
use state::{AppState, DashboardSnapshot};
use tauri::State;
use update::{PreparedUpdateStatus, UpdateStatus};

#[tauri::command]
fn dashboard_snapshot(state: State<'_, AppState>) -> Result<DashboardSnapshot, String> {
    let _ = state.ensure_agent_running();
    state.snapshot()
}

#[tauri::command]
fn refresh_health(state: State<'_, AppState>) -> Result<DashboardSnapshot, String> {
    let _ = state.ensure_agent_running();
    state.refresh_health()
}

#[tauri::command]
fn recent_events(
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<DashboardEvent>, String> {
    state.recent_events(limit.unwrap_or(100))
}

#[tauri::command]
fn clear_events(state: State<'_, AppState>) -> Result<(), String> {
    state.clear_events()
}

#[tauri::command]
fn notifications(
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<DashboardEvent>, String> {
    state.notifications(limit.unwrap_or(100))
}

#[tauri::command]
fn acknowledge_event(event_id: u64, state: State<'_, AppState>) -> Result<bool, String> {
    state.acknowledge_event(event_id)
}

#[tauri::command]
fn acknowledge_all_notifications(state: State<'_, AppState>) -> Result<usize, String> {
    state.acknowledge_all_notifications()
}

#[tauri::command]
fn health_history(
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<HealthHistoryEntry>, String> {
    state.health_history(limit.unwrap_or(100))
}

#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> Result<SecurityCenterSettings, String> {
    state.settings()
}

#[tauri::command]
fn save_settings(
    settings: SecurityCenterSettings,
    state: State<'_, AppState>,
) -> Result<SecurityCenterSettings, String> {
    state.update_settings(settings)
}

#[tauri::command]
fn agent_status(state: State<'_, AppState>) -> AgentStatus {
    state.agent_status()
}

#[tauri::command]
fn launch_agent(state: State<'_, AppState>) -> Result<(), String> {
    state.launch_agent()
}

#[tauri::command]
fn stop_agent(state: State<'_, AppState>) -> Result<AgentStatus, String> {
    state.stop_agent()
}

#[tauri::command]
fn restart_agent(state: State<'_, AppState>) -> Result<AgentStatus, String> {
    state.restart_agent()
}

#[tauri::command]
fn launch_authenticator(state: State<'_, AppState>) -> Result<(), String> {
    state.launch_authenticator()
}

#[tauri::command]
fn launch_security_scanner(state: State<'_, AppState>) -> Result<(), String> {
    state.launch_security_scanner()
}

#[tauri::command]
fn launch_integrity_monitor(state: State<'_, AppState>) -> Result<(), String> {
    state.launch_integrity_monitor()
}

#[tauri::command]
fn launch_network_guard(state: State<'_, AppState>) -> Result<(), String> {
    state.launch_network_guard()
}

#[tauri::command]
fn launch_secure_share(state: State<'_, AppState>) -> Result<(), String> {
    state.launch_secure_share()
}

#[tauri::command]
fn launch_backup_recovery(state: State<'_, AppState>) -> Result<(), String> {
    state.launch_backup_recovery()
}

#[tauri::command]
fn launch_file_vault(state: State<'_, AppState>) -> Result<(), String> {
    state.launch_file_vault()
}

#[tauri::command]
fn launch_password_manager(state: State<'_, AppState>) -> Result<(), String> {
    state.launch_password_manager()
}

#[tauri::command]
fn automation_status(state: State<'_, AppState>) -> Result<AutomationStatus, String> {
    state.automation_status()
}

#[tauri::command]
fn configure_automation_job(
    job: String,
    enabled: bool,
    interval_minutes: u64,
    state: State<'_, AppState>,
) -> Result<AutomationStatus, String> {
    state.configure_automation_job(&job, enabled, interval_minutes)
}

#[tauri::command]
fn run_automation_job(
    job: String,
    state: State<'_, AppState>,
) -> Result<AutomationStatus, String> {
    state.run_automation_job(&job)
}

#[tauri::command]
fn check_updates(state: State<'_, AppState>) -> Result<UpdateStatus, String> {
    state.check_updates()
}

#[tauri::command]
fn prepare_update(state: State<'_, AppState>) -> Result<PreparedUpdateStatus, String> {
    state.prepare_update()
}

#[tauri::command]
fn install_prepared_update(state: State<'_, AppState>) -> Result<(), String> {
    state.install_prepared_update()
}

#[tauri::command]
fn diagnostic_report(state: State<'_, AppState>) -> Result<String, String> {
    state.diagnostic_report()
}

#[tauri::command]
fn create_support_bundle(state: State<'_, AppState>) -> Result<String, String> {
    state.create_support_bundle()
}

pub fn run() {
    let state = AppState::initialize().unwrap_or_else(|error| {
        panic!("failed to initialize DragonForge Security Center: {error}")
    });

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            dashboard_snapshot,
            refresh_health,
            recent_events,
            clear_events,
            notifications,
            acknowledge_event,
            acknowledge_all_notifications,
            health_history,
            get_settings,
            save_settings,
            agent_status,
            launch_agent,
            stop_agent,
            restart_agent,
            launch_authenticator,
            launch_security_scanner,
            launch_integrity_monitor,
            launch_network_guard,
            launch_secure_share,
            launch_backup_recovery,
            launch_file_vault,
            launch_password_manager,
            automation_status,
            configure_automation_job,
            run_automation_job,
            check_updates,
            prepare_update,
            install_prepared_update,
            diagnostic_report,
            create_support_bundle
        ])
        .run(tauri::generate_context!())
        .expect("error while running DragonForge Security Center");
}
