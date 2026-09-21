#![forbid(unsafe_code)]

use dragonforge_security_scanner::{ScanReport, scan_system};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
struct ScannerInfo {
    name: &'static str,
    mode: &'static str,
    platform_scope: &'static str,
    remediation: bool,
}

#[tauri::command]
fn scanner_info() -> ScannerInfo {
    ScannerInfo {
        name: "DragonForge Security Scanner",
        mode: "read_only",
        platform_scope: "windows_first",
        remediation: false,
    }
}

#[tauri::command]
fn run_scan() -> ScanReport {
    scan_system()
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![scanner_info, run_scan])
        .run(tauri::generate_context!())
        .expect("error while running DragonForge Security Scanner");
}

#[cfg(test)]
mod tests {
    use super::scanner_info;

    #[test]
    fn phase6_scanner_is_explicitly_read_only() {
        let info = scanner_info();
        assert_eq!(info.mode, "read_only");
        assert!(!info.remediation);
    }
}
