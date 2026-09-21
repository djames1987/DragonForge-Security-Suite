#![forbid(unsafe_code)]

use dragonforge_network_guard::{NetworkSnapshot, collect_snapshot};
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
        mode: "visibility_only",
        platform_scope: "windows_first",
        enforcement: false,
        packet_capture: false,
    }
}

#[tauri::command]
fn refresh_network_snapshot() -> NetworkSnapshot {
    collect_snapshot()
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![guard_info, refresh_network_snapshot])
        .run(tauri::generate_context!())
        .expect("error while running DragonForge Network Guard");
}

#[cfg(test)]
mod tests {
    use super::guard_info;

    #[test]
    fn phase8_is_visibility_only() {
        let info = guard_info();
        assert_eq!(info.mode, "visibility_only");
        assert!(!info.enforcement);
        assert!(!info.packet_capture);
    }
}
