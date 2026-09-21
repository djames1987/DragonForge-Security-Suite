//! Minimal cross-platform runtime and path abstraction.

use std::{env, path::PathBuf};

use crate::error::{CoreError, CoreResult, ErrorCode};
use crate::Component;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform {
    Windows,
    Linux,
    MacOs,
    Other,
}

impl Platform {
    #[must_use]
    pub const fn current() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "linux") {
            Self::Linux
        } else if cfg!(target_os = "macos") {
            Self::MacOs
        } else {
            Self::Other
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::Linux => "linux",
            Self::MacOs => "macos",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuitePaths {
    config_root: PathBuf,
    data_root: PathBuf,
    cache_root: PathBuf,
}

impl SuitePaths {
    #[must_use]
    pub fn from_roots(
        config_root: impl Into<PathBuf>,
        data_root: impl Into<PathBuf>,
        cache_root: impl Into<PathBuf>,
    ) -> Self {
        Self {
            config_root: config_root.into(),
            data_root: data_root.into(),
            cache_root: cache_root.into(),
        }
    }

    pub fn discover() -> CoreResult<Self> {
        match Platform::current() {
            Platform::Windows => {
                let roaming = env_path("APPDATA")?;
                let local = env_path("LOCALAPPDATA")?;
                Ok(Self::from_roots(
                    roaming.join("DragonForge"),
                    local.join("DragonForge"),
                    local.join("DragonForge").join("Cache"),
                ))
            }
            Platform::Linux => {
                let home = env::var_os("HOME").map(PathBuf::from);
                let config = optional_env_path("XDG_CONFIG_HOME")
                    .or_else(|| home.as_ref().map(|path| path.join(".config")))
                    .ok_or_else(missing_home_error)?;
                let data = optional_env_path("XDG_DATA_HOME")
                    .or_else(|| home.as_ref().map(|path| path.join(".local").join("share")))
                    .ok_or_else(missing_home_error)?;
                let cache = optional_env_path("XDG_CACHE_HOME")
                    .or_else(|| home.as_ref().map(|path| path.join(".cache")))
                    .ok_or_else(missing_home_error)?;
                Ok(Self::from_roots(
                    config.join("dragonforge"),
                    data.join("dragonforge"),
                    cache.join("dragonforge"),
                ))
            }
            Platform::MacOs => {
                let home = env_path("HOME")?;
                Ok(Self::from_roots(
                    home.join("Library")
                        .join("Application Support")
                        .join("DragonForge"),
                    home.join("Library")
                        .join("Application Support")
                        .join("DragonForge"),
                    home.join("Library").join("Caches").join("DragonForge"),
                ))
            }
            Platform::Other => Err(CoreError::new_safe(
                ErrorCode::UnsupportedPlatform,
                "automatic DragonForge path discovery is unsupported on this platform",
            )),
        }
    }

    #[must_use]
    pub fn config_root(&self) -> &std::path::Path {
        &self.config_root
    }

    #[must_use]
    pub fn data_root(&self) -> &std::path::Path {
        &self.data_root
    }

    #[must_use]
    pub fn cache_root(&self) -> &std::path::Path {
        &self.cache_root
    }

    #[must_use]
    pub fn component_config_dir(&self, component: Component) -> PathBuf {
        self.config_root.join(component.as_str())
    }

    #[must_use]
    pub fn component_data_dir(&self, component: Component) -> PathBuf {
        self.data_root.join(component.as_str())
    }

    #[must_use]
    pub fn component_cache_dir(&self, component: Component) -> PathBuf {
        self.cache_root.join(component.as_str())
    }
}

fn optional_env_path(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn env_path(name: &str) -> CoreResult<PathBuf> {
    optional_env_path(name).ok_or_else(|| {
        CoreError::new_safe(
            ErrorCode::InvalidConfiguration,
            format!("required environment path {name} is unavailable"),
        )
    })
}

fn missing_home_error() -> CoreError {
    CoreError::new_safe(
        ErrorCode::InvalidConfiguration,
        "HOME or XDG path variables are unavailable",
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{Platform, SuitePaths};
    use crate::Component;

    #[test]
    fn current_platform_has_stable_identifier() {
        assert!(!Platform::current().as_str().is_empty());
    }

    #[test]
    fn component_paths_are_namespaced() {
        let paths = SuitePaths::from_roots("config", "data", "cache");
        assert_eq!(
            paths.component_config_dir(Component::SecurityCenter),
            PathBuf::from("config").join("security-center")
        );
        assert_eq!(
            paths.component_data_dir(Component::PasswordManager),
            PathBuf::from("data").join("password-manager")
        );
    }
}
