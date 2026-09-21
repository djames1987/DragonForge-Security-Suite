use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use dragonforge_core::{CoreError, CoreResult, ErrorCode};

#[must_use]
fn sibling_executable(center_executable: &Path, name: &str) -> Option<PathBuf> {
    let parent = center_executable.parent()?;
    let file_name = if cfg!(target_os = "windows") {
        format!("{name}.exe")
    } else {
        name.to_owned()
    };
    Some(parent.join(file_name))
}

#[must_use]
pub fn password_manager_sibling(center_executable: &Path) -> Option<PathBuf> {
    sibling_executable(center_executable, "dragonforge-desktop")
}

pub fn agent_sibling(center_executable: &Path) -> Option<PathBuf> {
    sibling_executable(center_executable, "dragonforge-agent")
}

pub fn file_vault_sibling(center_executable: &Path) -> Option<PathBuf> {
    sibling_executable(center_executable, "dragonforge-file-vault")
}

pub fn authenticator_sibling(center_executable: &Path) -> Option<PathBuf> {
    sibling_executable(center_executable, "dragonforge-authenticator")
}

pub fn security_scanner_sibling(center_executable: &Path) -> Option<PathBuf> {
    sibling_executable(center_executable, "dragonforge-security-scanner")
}

pub fn integrity_monitor_sibling(center_executable: &Path) -> Option<PathBuf> {
    sibling_executable(center_executable, "dragonforge-integrity-monitor")
}

pub fn network_guard_sibling(center_executable: &Path) -> Option<PathBuf> {
    sibling_executable(center_executable, "dragonforge-network-guard")
}

pub fn backup_recovery_sibling(center_executable: &Path) -> Option<PathBuf> {
    sibling_executable(center_executable, "dragonforge-backup-recovery")
}

pub fn secure_share_sibling(center_executable: &Path) -> Option<PathBuf> {
    sibling_executable(center_executable, "dragonforge-secure-share")
}

fn launch_sibling(target: PathBuf, display_name: &str) -> CoreResult<()> {
    if !target.is_file() {
        return Err(CoreError::new_safe(
            ErrorCode::InvalidConfiguration,
            format!("{display_name} is not installed beside Security Center"),
        ));
    }

    Command::new(target).spawn().map_err(|_| {
        CoreError::new_safe(
            ErrorCode::Internal,
            format!("unable to start the {display_name} application"),
        )
    })?;
    Ok(())
}

pub fn launch_agent() -> CoreResult<()> {
    let current = env::current_exe().map_err(|_| {
        CoreError::new_safe(
            ErrorCode::Internal,
            "unable to resolve the Security Center executable path",
        )
    })?;
    let target = agent_sibling(&current).ok_or_else(|| {
        CoreError::new_safe(
            ErrorCode::Internal,
            "unable to resolve the DragonForge Agent sibling path",
        )
    })?;

    if !target.is_file() {
        return Err(CoreError::new_safe(
            ErrorCode::InvalidConfiguration,
            "DragonForge Agent is not installed beside Security Center",
        ));
    }

    let mut command = Command::new(target);
    command.arg("--serve");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command.spawn().map_err(|_| {
        CoreError::new_safe(ErrorCode::Internal, "unable to start the DragonForge Agent")
    })?;
    Ok(())
}

pub fn launch_password_manager() -> CoreResult<()> {
    let current = env::current_exe().map_err(|_| {
        CoreError::new_safe(
            ErrorCode::Internal,
            "unable to resolve the Security Center executable path",
        )
    })?;
    let target = password_manager_sibling(&current).ok_or_else(|| {
        CoreError::new_safe(
            ErrorCode::Internal,
            "unable to resolve the Password Manager sibling path",
        )
    })?;

    launch_sibling(target, "Password Manager")
}

pub fn launch_authenticator() -> CoreResult<()> {
    let current = env::current_exe().map_err(|_| {
        CoreError::new_safe(
            ErrorCode::Internal,
            "unable to resolve the Security Center executable path",
        )
    })?;
    let target = authenticator_sibling(&current).ok_or_else(|| {
        CoreError::new_safe(
            ErrorCode::Internal,
            "unable to resolve the Authenticator sibling path",
        )
    })?;

    launch_sibling(target, "Authenticator")
}

pub fn launch_security_scanner() -> CoreResult<()> {
    let current = env::current_exe().map_err(|_| {
        CoreError::new_safe(
            ErrorCode::Internal,
            "unable to resolve the Security Center executable path",
        )
    })?;
    let target = security_scanner_sibling(&current).ok_or_else(|| {
        CoreError::new_safe(
            ErrorCode::Internal,
            "unable to resolve the Security Scanner sibling path",
        )
    })?;

    launch_sibling(target, "Security Scanner")
}

pub fn launch_integrity_monitor() -> CoreResult<()> {
    let current = env::current_exe().map_err(|_| {
        CoreError::new_safe(
            ErrorCode::Internal,
            "unable to resolve the Security Center executable path",
        )
    })?;
    let target = integrity_monitor_sibling(&current).ok_or_else(|| {
        CoreError::new_safe(
            ErrorCode::Internal,
            "unable to resolve the Integrity Monitor sibling path",
        )
    })?;

    launch_sibling(target, "Integrity Monitor")
}

pub fn launch_network_guard() -> CoreResult<()> {
    let current = env::current_exe().map_err(|_| {
        CoreError::new_safe(
            ErrorCode::Internal,
            "unable to resolve the Security Center executable path",
        )
    })?;
    let target = network_guard_sibling(&current).ok_or_else(|| {
        CoreError::new_safe(
            ErrorCode::Internal,
            "unable to resolve the Network Guard sibling path",
        )
    })?;

    launch_sibling(target, "Network Guard")
}

pub fn launch_secure_share() -> CoreResult<()> {
    let current = env::current_exe().map_err(|_| {
        CoreError::new_safe(
            ErrorCode::Internal,
            "unable to resolve the Security Center executable path",
        )
    })?;
    let target = secure_share_sibling(&current).ok_or_else(|| {
        CoreError::new_safe(
            ErrorCode::Internal,
            "unable to resolve the Secure Share sibling path",
        )
    })?;

    launch_sibling(target, "Secure Share")
}

pub fn launch_backup_recovery() -> CoreResult<()> {
    let current = env::current_exe().map_err(|_| {
        CoreError::new_safe(
            ErrorCode::Internal,
            "unable to resolve the Security Center executable path",
        )
    })?;
    let target = backup_recovery_sibling(&current).ok_or_else(|| {
        CoreError::new_safe(
            ErrorCode::Internal,
            "unable to resolve the Backup & Recovery sibling path",
        )
    })?;

    launch_sibling(target, "Backup & Recovery")
}

pub fn launch_file_vault() -> CoreResult<()> {
    let current = env::current_exe().map_err(|_| {
        CoreError::new_safe(
            ErrorCode::Internal,
            "unable to resolve the Security Center executable path",
        )
    })?;
    let target = file_vault_sibling(&current).ok_or_else(|| {
        CoreError::new_safe(
            ErrorCode::Internal,
            "unable to resolve the File Vault sibling path",
        )
    })?;

    launch_sibling(target, "File Vault")
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{
        agent_sibling, authenticator_sibling, backup_recovery_sibling, file_vault_sibling,
        integrity_monitor_sibling, network_guard_sibling, password_manager_sibling,
        secure_share_sibling, security_scanner_sibling,
    };

    #[test]
    fn agent_path_is_strictly_sibling_scoped() {
        let center = if cfg!(target_os = "windows") {
            Path::new(r"C:\DragonForge\dragonforge-security-center.exe")
        } else {
            Path::new("/opt/dragonforge/dragonforge-security-center")
        };

        let expected = if cfg!(target_os = "windows") {
            PathBuf::from(r"C:\DragonForge\dragonforge-agent.exe")
        } else {
            PathBuf::from("/opt/dragonforge/dragonforge-agent")
        };

        assert_eq!(agent_sibling(center), Some(expected));
    }

    #[test]
    fn password_manager_path_is_strictly_sibling_scoped() {
        let center = if cfg!(target_os = "windows") {
            Path::new(r"C:\DragonForge\dragonforge-security-center.exe")
        } else {
            Path::new("/opt/dragonforge/dragonforge-security-center")
        };

        let expected = if cfg!(target_os = "windows") {
            PathBuf::from(r"C:\DragonForge\dragonforge-desktop.exe")
        } else {
            PathBuf::from("/opt/dragonforge/dragonforge-desktop")
        };

        assert_eq!(password_manager_sibling(center), Some(expected));
    }

    #[test]
    fn authenticator_path_is_strictly_sibling_scoped() {
        let center = if cfg!(target_os = "windows") {
            Path::new(r"C:\DragonForge\dragonforge-security-center.exe")
        } else {
            Path::new("/opt/dragonforge/dragonforge-security-center")
        };

        let expected = if cfg!(target_os = "windows") {
            PathBuf::from(r"C:\DragonForge\dragonforge-authenticator.exe")
        } else {
            PathBuf::from("/opt/dragonforge/dragonforge-authenticator")
        };

        assert_eq!(authenticator_sibling(center), Some(expected));
    }

    #[test]
    fn security_scanner_path_is_strictly_sibling_scoped() {
        let center = if cfg!(target_os = "windows") {
            Path::new(r"C:\DragonForge\dragonforge-security-center.exe")
        } else {
            Path::new("/opt/dragonforge/dragonforge-security-center")
        };

        let expected = if cfg!(target_os = "windows") {
            PathBuf::from(r"C:\DragonForge\dragonforge-security-scanner.exe")
        } else {
            PathBuf::from("/opt/dragonforge/dragonforge-security-scanner")
        };

        assert_eq!(security_scanner_sibling(center), Some(expected));
    }

    #[test]
    fn integrity_monitor_path_is_strictly_sibling_scoped() {
        let center = if cfg!(target_os = "windows") {
            Path::new(r"C:\DragonForge\dragonforge-security-center.exe")
        } else {
            Path::new("/opt/dragonforge/dragonforge-security-center")
        };

        let expected = if cfg!(target_os = "windows") {
            PathBuf::from(r"C:\DragonForge\dragonforge-integrity-monitor.exe")
        } else {
            PathBuf::from("/opt/dragonforge/dragonforge-integrity-monitor")
        };

        assert_eq!(integrity_monitor_sibling(center), Some(expected));
    }

    #[test]
    fn network_guard_path_is_strictly_sibling_scoped() {
        let center = if cfg!(target_os = "windows") {
            Path::new(r"C:\DragonForge\dragonforge-security-center.exe")
        } else {
            Path::new("/opt/dragonforge/dragonforge-security-center")
        };

        let expected = if cfg!(target_os = "windows") {
            PathBuf::from(r"C:\DragonForge\dragonforge-network-guard.exe")
        } else {
            PathBuf::from("/opt/dragonforge/dragonforge-network-guard")
        };

        assert_eq!(network_guard_sibling(center), Some(expected));
    }

    #[test]
    fn secure_share_path_is_strictly_sibling_scoped() {
        let center = if cfg!(target_os = "windows") {
            Path::new(r"C:\DragonForge\dragonforge-security-center.exe")
        } else {
            Path::new("/opt/dragonforge/dragonforge-security-center")
        };

        let expected = if cfg!(target_os = "windows") {
            PathBuf::from(r"C:\DragonForge\dragonforge-secure-share.exe")
        } else {
            PathBuf::from("/opt/dragonforge/dragonforge-secure-share")
        };

        assert_eq!(secure_share_sibling(center), Some(expected));
    }

    #[test]
    fn backup_recovery_path_is_strictly_sibling_scoped() {
        let center = if cfg!(target_os = "windows") {
            Path::new(r"C:\DragonForge\dragonforge-security-center.exe")
        } else {
            Path::new("/opt/dragonforge/dragonforge-security-center")
        };

        let expected = if cfg!(target_os = "windows") {
            PathBuf::from(r"C:\DragonForge\dragonforge-backup-recovery.exe")
        } else {
            PathBuf::from("/opt/dragonforge/dragonforge-backup-recovery")
        };

        assert_eq!(backup_recovery_sibling(center), Some(expected));
    }

    #[test]
    fn file_vault_path_is_strictly_sibling_scoped() {
        let center = if cfg!(target_os = "windows") {
            Path::new(r"C:\DragonForge\dragonforge-security-center.exe")
        } else {
            Path::new("/opt/dragonforge/dragonforge-security-center")
        };

        let expected = if cfg!(target_os = "windows") {
            PathBuf::from(r"C:\DragonForge\dragonforge-file-vault.exe")
        } else {
            PathBuf::from("/opt/dragonforge/dragonforge-file-vault")
        };

        assert_eq!(file_vault_sibling(center), Some(expected));
    }
}
