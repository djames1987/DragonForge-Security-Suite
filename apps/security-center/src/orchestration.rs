use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use dragonforge_core::{CoreError, CoreResult, ErrorCode};

#[must_use]
pub fn password_manager_sibling(center_executable: &Path) -> Option<PathBuf> {
    let parent = center_executable.parent()?;
    let file_name = if cfg!(target_os = "windows") {
        "dragonforge-desktop.exe"
    } else {
        "dragonforge-desktop"
    };
    Some(parent.join(file_name))
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

    if !target.is_file() {
        return Err(CoreError::new_safe(
            ErrorCode::InvalidConfiguration,
            "Password Manager is not installed beside Security Center",
        ));
    }

    Command::new(target).spawn().map_err(|_| {
        CoreError::new_safe(
            ErrorCode::Internal,
            "unable to start the Password Manager application",
        )
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::password_manager_sibling;

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
}
