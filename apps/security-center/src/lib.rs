#![forbid(unsafe_code)]

mod agent;
mod events;
mod logging;
mod model;
mod settings;
mod state;

use agent::AgentStatus;
use events::DashboardEvent;
use settings::SecurityCenterSettings;
use state::{AppState, DashboardSnapshot};
use tauri::State;

#[tauri::command]
fn dashboard_snapshot(state: State<'_, AppState>) -> Result<DashboardSnapshot, String> {
    state.snapshot()
}

#[tauri::command]
fn refresh_health(state: State<'_, AppState>) -> Result<DashboardSnapshot, String> {
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
            get_settings,
            save_settings,
            agent_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running DragonForge Security Center");
}
