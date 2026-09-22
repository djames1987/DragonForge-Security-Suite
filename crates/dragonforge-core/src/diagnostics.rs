//! Shared crash-safe diagnostics primitives.
//!
//! This module intentionally records only bounded public metadata. Panic payloads,
//! secrets, arbitrary file paths, user documents, credentials, and vault contents
//! are never persisted.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{Component, CoreError, CoreResult, ErrorCode, LogPolicy, SuitePaths};

const DEFAULT_MAX_BYTES: u64 = 1_048_576;
const DEFAULT_BACKUPS: usize = 3;

#[derive(Debug, Clone)]
pub struct ComponentLogger {
    path: PathBuf,
    policy: LogPolicy,
    max_bytes: u64,
    backups: usize,
}

impl ComponentLogger {
    pub fn discover(component: Component, include_identifiers: bool) -> CoreResult<Self> {
        let paths = SuitePaths::discover()?;
        Ok(Self::from_path(
            paths.component_data_dir(component).join("logs").join(format!(
                "{}.log",
                component.as_str()
            )),
            LogPolicy {
                include_identifiers,
                ..LogPolicy::default()
            },
        ))
    }

    #[must_use]
    pub fn from_path(path: PathBuf, policy: LogPolicy) -> Self {
        Self {
            path,
            policy,
            max_bytes: DEFAULT_MAX_BYTES,
            backups: DEFAULT_BACKUPS,
        }
    }

    #[must_use]
    pub fn with_rotation(mut self, max_bytes: u64, backups: usize) -> Self {
        self.max_bytes = max_bytes.max(4_096);
        self.backups = backups.clamp(1, 8);
        self
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn write(&self, level: &str, code: &str, public_message: &str) -> CoreResult<()> {
        self.ensure_parent()?;
        self.rotate_if_needed()?;

        let timestamp_ms = now_ms();
        let level = self.policy.sanitize_public(level);
        let code = self.policy.sanitize_public(code);
        let message = sanitize_diagnostic_text(self.policy, public_message);
        let line = format!("{timestamp_ms} {level} {code} {message}\n");

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|_| CoreError::new_safe(ErrorCode::Internal, "unable to open component log"))?;
        file.write_all(line.as_bytes())
            .map_err(|_| CoreError::new_safe(ErrorCode::Internal, "unable to write component log"))
    }

    pub fn record_failure(&self, code: &str, public_summary: &str) -> CoreResult<()> {
        self.write("error", code, public_summary)?;
        let failure_path = self
            .path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("last-failure.txt");
        let body = format!(
            "schema=1\ntimestamp_ms={}\ncode={}\nsummary={}\n",
            now_ms(),
            self.policy.sanitize_public(code),
            sanitize_diagnostic_text(self.policy, public_summary)
        );
        fs::write(failure_path, body)
            .map_err(|_| CoreError::new_safe(ErrorCode::Internal, "unable to write failure record"))
    }

    #[must_use]
    pub fn read_last_failure(&self) -> Option<String> {
        let path = self
            .path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("last-failure.txt");
        fs::read_to_string(path).ok().map(|value| {
            value
                .lines()
                .take(8)
                .map(|line| sanitize_diagnostic_text(self.policy, line))
                .collect::<Vec<_>>()
                .join("\n")
        })
    }

    fn ensure_parent(&self) -> CoreResult<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|_| {
                CoreError::new_safe(ErrorCode::Internal, "unable to create component log directory")
            })?;
        }
        Ok(())
    }

    fn rotate_if_needed(&self) -> CoreResult<()> {
        let size = fs::metadata(&self.path).map(|meta| meta.len()).unwrap_or(0);
        if size < self.max_bytes {
            return Ok(());
        }

        for index in (1..=self.backups).rev() {
            let from = if index == 1 {
                self.path.clone()
            } else {
                rotated_path(&self.path, index - 1)
            };
            let to = rotated_path(&self.path, index);
            if !from.exists() {
                continue;
            }
            if to.exists() {
                let _ = fs::remove_file(&to);
            }
            fs::rename(&from, &to).map_err(|_| {
                CoreError::new_safe(ErrorCode::Internal, "unable to rotate component log")
            })?;
        }
        Ok(())
    }
}

pub fn install_safe_panic_hook(logger: ComponentLogger, component: Component) {
    std::panic::set_hook(Box::new(move |_| {
        let _ = logger.record_failure(
            "process.panic",
            &format!("{} terminated after an unexpected panic", component.as_str()),
        );
    }));
}

#[must_use]
pub fn sanitize_diagnostic_text(policy: LogPolicy, value: &str) -> String {
    let lowered = value.to_ascii_lowercase();
    const SENSITIVE_MARKERS: &[&str] = &[
        "password",
        "passwd",
        "token",
        "secret",
        "recovery",
        "otp",
        "totp",
        "hotp",
        "seed",
        "private_key",
        "private key",
        "credential",
        "authorization:",
        "bearer ",
    ];
    if SENSITIVE_MARKERS.iter().any(|marker| lowered.contains(marker)) {
        return "[REDACTED]".to_owned();
    }
    policy.sanitize_public(value)
}

fn rotated_path(path: &Path, index: usize) -> PathBuf {
    PathBuf::from(format!("{}.{}", path.display(), index))
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{ComponentLogger, sanitize_diagnostic_text};
    use crate::LogPolicy;

    fn test_dir(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dragonforge-core-{name}-{nonce}"));
        fs::create_dir_all(&path).expect("create test directory");
        path
    }

    #[test]
    fn diagnostic_sanitizer_redacts_secret_markers() {
        let policy = LogPolicy::default();
        assert_eq!(
            sanitize_diagnostic_text(policy, "password=hunter2"),
            "[REDACTED]"
        );
        assert_eq!(
            sanitize_diagnostic_text(policy, "Authorization: Bearer abc"),
            "[REDACTED]"
        );
        assert_eq!(
            sanitize_diagnostic_text(policy, "safe public status"),
            "safe public status"
        );
    }

    #[test]
    fn component_logger_rotates_bounded_files() {
        let dir = test_dir("rotation");
        let path = dir.join("component.log");
        let logger = ComponentLogger::from_path(path.clone(), LogPolicy::default())
            .with_rotation(4_096, 2);
        for _ in 0..100 {
            logger
                .write("info", "rotation.test", &"x".repeat(128))
                .expect("write");
        }
        assert!(path.exists());
        assert!(PathBuf::from(format!("{}.1", path.display())).exists());
    }

    #[test]
    fn failure_record_never_persists_sensitive_summary() {
        let dir = test_dir("failure");
        let path = dir.join("component.log");
        let logger = ComponentLogger::from_path(path, LogPolicy::default());
        logger
            .record_failure("test.failure", "token=super-secret-value")
            .expect("record");
        let failure = fs::read_to_string(dir.join("last-failure.txt")).expect("read");
        assert!(!failure.contains("super-secret-value"));
        assert!(failure.contains("[REDACTED]"));
    }
}
