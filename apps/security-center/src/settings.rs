use std::fs;
use std::io::Write;
use std::path::PathBuf;

use dragonforge_core::{Component, CoreError, CoreResult, ErrorCode, SuitePaths};
use serde::{Deserialize, Serialize};

const SETTINGS_FILE: &str = "settings.json";
const SETTINGS_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityCenterSettings {
    pub version: u32,
    pub start_on_overview: bool,
    pub retain_event_count: usize,
    pub include_diagnostic_identifiers: bool,
}

impl Default for SecurityCenterSettings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            start_on_overview: true,
            retain_event_count: 250,
            include_diagnostic_identifiers: false,
        }
    }
}

impl SecurityCenterSettings {
    pub fn validate(&self) -> CoreResult<()> {
        if self.version != SETTINGS_VERSION {
            return Err(CoreError::new_safe(
                ErrorCode::InvalidConfiguration,
                "unsupported Security Center settings version",
            ));
        }

        if !(50..=2_000).contains(&self.retain_event_count) {
            return Err(CoreError::new_safe(
                ErrorCode::InvalidConfiguration,
                "event retention must be between 50 and 2000",
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    pub fn discover() -> CoreResult<Self> {
        let paths = SuitePaths::discover()?;
        Ok(Self::from_dir(
            paths.component_config_dir(Component::SecurityCenter),
        ))
    }

    #[must_use]
    pub fn from_dir(directory: impl Into<PathBuf>) -> Self {
        Self {
            path: directory.into().join(SETTINGS_FILE),
        }
    }

    pub fn load(&self) -> CoreResult<SecurityCenterSettings> {
        if !self.path.exists() {
            return Ok(SecurityCenterSettings::default());
        }

        let bytes = fs::read(&self.path).map_err(|_| {
            CoreError::new_safe(
                ErrorCode::InvalidConfiguration,
                "unable to read Security Center settings",
            )
        })?;

        let settings: SecurityCenterSettings = serde_json::from_slice(&bytes).map_err(|_| {
            CoreError::new_safe(
                ErrorCode::InvalidConfiguration,
                "Security Center settings are invalid JSON",
            )
        })?;
        settings.validate()?;
        Ok(settings)
    }

    pub fn save(&self, settings: &SecurityCenterSettings) -> CoreResult<()> {
        settings.validate()?;

        let parent = self.path.parent().ok_or_else(|| {
            CoreError::new_safe(
                ErrorCode::InvalidConfiguration,
                "Security Center settings path has no parent directory",
            )
        })?;

        fs::create_dir_all(parent).map_err(|_| {
            CoreError::new_safe(
                ErrorCode::Internal,
                "unable to create Security Center configuration directory",
            )
        })?;

        let encoded = serde_json::to_vec_pretty(settings).map_err(|_| {
            CoreError::new_safe(
                ErrorCode::Internal,
                "unable to serialize Security Center settings",
            )
        })?;

        let temporary = self.path.with_extension("json.tmp");
        {
            let mut file = fs::File::create(&temporary).map_err(|_| {
                CoreError::new_safe(
                    ErrorCode::Internal,
                    "unable to create temporary Security Center settings file",
                )
            })?;
            file.write_all(&encoded).map_err(|_| {
                CoreError::new_safe(
                    ErrorCode::Internal,
                    "unable to write Security Center settings",
                )
            })?;
            file.sync_all().map_err(|_| {
                CoreError::new_safe(
                    ErrorCode::Internal,
                    "unable to flush Security Center settings",
                )
            })?;
        }

        if self.path.exists() {
            fs::remove_file(&self.path).map_err(|_| {
                CoreError::new_safe(
                    ErrorCode::Internal,
                    "unable to replace existing Security Center settings",
                )
            })?;
        }

        fs::rename(&temporary, &self.path).map_err(|_| {
            CoreError::new_safe(
                ErrorCode::Internal,
                "unable to finalize Security Center settings",
            )
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{SecurityCenterSettings, SettingsStore};

    #[test]
    fn missing_settings_use_safe_defaults() {
        let dir = tempdir().expect("temporary directory");
        let store = SettingsStore::from_dir(dir.path());
        let settings = store.load().expect("defaults");
        assert_eq!(settings, SecurityCenterSettings::default());
        assert!(!settings.include_diagnostic_identifiers);
    }

    #[test]
    fn settings_round_trip() {
        let dir = tempdir().expect("temporary directory");
        let store = SettingsStore::from_dir(dir.path());
        let settings = SecurityCenterSettings {
            retain_event_count: 500,
            ..SecurityCenterSettings::default()
        };
        store.save(&settings).expect("save");
        assert_eq!(store.load().expect("load"), settings);
    }

    #[test]
    fn unreasonable_event_retention_is_rejected() {
        let settings = SecurityCenterSettings {
            retain_event_count: 5,
            ..SecurityCenterSettings::default()
        };
        assert!(settings.validate().is_err());
    }
}
