use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use dragonforge_core::{Component, CoreError, CoreResult, ErrorCode, LogPolicy, SuitePaths};

#[derive(Debug, Clone)]
pub struct SafeLogger {
    path: PathBuf,
    policy: LogPolicy,
}

impl SafeLogger {
    pub fn discover(include_identifiers: bool) -> CoreResult<Self> {
        let paths = SuitePaths::discover()?;
        let directory = paths.component_data_dir(Component::SecurityCenter).join("logs");
        Ok(Self {
            path: directory.join("security-center.log"),
            policy: LogPolicy {
                include_identifiers,
                ..LogPolicy::default()
            },
        })
    }

    #[cfg(test)]
    #[must_use]
    pub fn from_path(path: PathBuf, policy: LogPolicy) -> Self {
        Self { path, policy }
    }

    pub fn write(&self, level: &str, code: &str, public_message: &str) -> CoreResult<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|_| {
                CoreError::new_safe(
                    ErrorCode::Internal,
                    "unable to create Security Center log directory",
                )
            })?;
        }

        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let level = self.policy.sanitize_public(level);
        let code = self.policy.sanitize_public(code);
        let message = self.policy.sanitize_public(public_message);
        let line = format!("{timestamp_ms} {level} {code} {message}\n");

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|_| {
                CoreError::new_safe(
                    ErrorCode::Internal,
                    "unable to open Security Center log",
                )
            })?;
        file.write_all(line.as_bytes())
            .map_err(|_| {
                CoreError::new_safe(
                    ErrorCode::Internal,
                    "unable to write Security Center log",
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use dragonforge_core::LogPolicy;
    use tempfile::tempdir;

    use super::SafeLogger;

    #[test]
    fn logger_sanitizes_control_characters() {
        let dir = tempdir().expect("temporary directory");
        let path = dir.path().join("security-center.log");
        let logger = SafeLogger::from_path(path.clone(), LogPolicy::default());
        logger
            .write("info", "test.event", "safe\nsecond-line")
            .expect("write");
        let content = fs::read_to_string(path).expect("read");
        assert!(content.contains("safe second-line"));
        assert!(!content.contains("safe\nsecond-line"));
    }
}
