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

pub fn file_vault_sibling(center_executable: &Path) -> Option<PathBuf> {
    sibling_executable(center_executable, "dragonforge-file-vault")
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

    use super::{file_vault_sibling, password_manager_sibling};

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
