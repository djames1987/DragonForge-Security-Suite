#![forbid(unsafe_code)]

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use dragonforge_network_guard::{
    ApplicationIdentity, NetworkSnapshot, collect_snapshot, inspect_process_application,
};
use dragonforge_windows_boundary::{FirewallAction, FirewallMutationResult};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
struct GuardInfo {
    name: &'static str,
    mode: &'static str,
    platform_scope: &'static str,
    enforcement: bool,
    packet_capture: bool,
}

#[tauri::command]
fn guard_info() -> GuardInfo {
    GuardInfo {
        name: "DragonForge Network Guard",
        mode: "visibility_plus_firewall_policy",
        platform_scope: "windows_first",
        enforcement: true,
        packet_capture: false,
    }
}

#[tauri::command]
fn inspect_application(process_id: u32) -> Result<ApplicationIdentity, String> {
    inspect_process_application(process_id)
}

#[tauri::command]
fn firewall_status(process_id: u32) -> Result<FirewallMutationResult, String> {
    let identity = inspect_process_application(process_id)?;
    invoke_agent_firewall("--firewall-status", &identity, None)
}

#[tauri::command]
fn set_firewall_policy(
    process_id: u32,
    action: FirewallAction,
) -> Result<FirewallMutationResult, String> {
    let identity = inspect_process_application(process_id)?;
    let command = match action {
        FirewallAction::Allow => "--firewall-allow",
        FirewallAction::Block => "--firewall-block",
    };
    invoke_agent_firewall(command, &identity, None)
}

#[tauri::command]
fn remove_firewall_policy(process_id: u32) -> Result<FirewallMutationResult, String> {
    let identity = inspect_process_application(process_id)?;
    invoke_agent_firewall("--firewall-remove", &identity, None)
}

#[tauri::command]
fn rollback_firewall_policy(
    process_id: u32,
    rollback_token: String,
) -> Result<FirewallMutationResult, String> {
    if rollback_token.len() != 32
        || !rollback_token.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err("rollback token is invalid".to_owned());
    }
    let identity = inspect_process_application(process_id)?;
    invoke_agent_firewall("--firewall-rollback", &identity, Some(&rollback_token))
}

fn invoke_agent_firewall(
    command_name: &str,
    identity: &ApplicationIdentity,
    rollback_token: Option<&str>,
) -> Result<FirewallMutationResult, String> {
    if !cfg!(target_os = "windows") {
        return Err("Phase 19 firewall policy requires Windows".to_owned());
    }
    let current = env::current_exe()
        .map_err(|_| "Network Guard executable path is unavailable".to_owned())?;
    let agent = agent_sibling(&current)
        .ok_or_else(|| "DragonForge Agent sibling path is unavailable".to_owned())?;
    if !agent.is_file() {
        return Err("DragonForge Agent is not installed beside Network Guard".to_owned());
    }

    let mut command = Command::new(agent);
    command.arg(command_name);
    if let Some(token) = rollback_token {
        command.arg(token);
    }
    command
        .arg(&identity.application_path)
        .arg(&identity.sha256_hex)
        .arg(&identity.process_name);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let output = command
        .output()
        .map_err(|_| "DragonForge Agent firewall request could not be started".to_owned())?;
    if !output.status.success() {
        return Err("DragonForge Agent firewall request was rejected".to_owned());
    }
    if output.stdout.len() > 64 * 1024 {
        return Err("DragonForge Agent firewall response exceeded the safe limit".to_owned());
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|_| "DragonForge Agent firewall response was malformed".to_owned())
}

#[must_use]
fn agent_sibling(current: &Path) -> Option<PathBuf> {
    let parent = current.parent()?;
    Some(parent.join(if cfg!(target_os = "windows") {
        "dragonforge-agent.exe"
    } else {
        "dragonforge-agent"
    }))
}

#[tauri::command]
fn refresh_network_snapshot() -> NetworkSnapshot {
    collect_snapshot()
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            guard_info,
            refresh_network_snapshot,
            inspect_application,
            firewall_status,
            set_firewall_policy,
            remove_firewall_policy,
            rollback_firewall_policy
        ])
        .run(tauri::generate_context!())
        .expect("error while running DragonForge Network Guard");
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::guard_info;

    #[test]
    fn phase19_network_guard_exposes_typed_firewall_policy() {
        let info = guard_info();
        assert_eq!(info.mode, "visibility_plus_firewall_policy");
        assert!(info.enforcement);
        assert!(!info.packet_capture);
    }

    #[test]
    fn agent_path_is_strictly_sibling_scoped() {
        let current = if cfg!(target_os = "windows") {
            Path::new(r"C:\DragonForge\dragonforge-network-guard.exe")
        } else {
            Path::new("/opt/dragonforge/dragonforge-network-guard")
        };
        let expected = if cfg!(target_os = "windows") {
            PathBuf::from(r"C:\DragonForge\dragonforge-agent.exe")
        } else {
            PathBuf::from("/opt/dragonforge/dragonforge-agent")
        };
        assert_eq!(super::agent_sibling(current), Some(expected));
    }
}
