use std::path::Path;

use dragonforge_core::{Component, ComponentLogger, CoreResult, install_safe_panic_hook};

#[derive(Debug, Clone)]
pub struct SafeLogger {
    inner: ComponentLogger,
}

impl SafeLogger {
    pub fn discover(include_identifiers: bool) -> CoreResult<Self> {
        Ok(Self {
            inner: ComponentLogger::discover(Component::SecurityCenter, include_identifiers)?,
        })
    }

    #[cfg(test)]
    #[must_use]
    pub fn from_path(path: PathBuf, policy: LogPolicy) -> Self {
        Self {
            inner: ComponentLogger::from_path(path, policy).with_rotation(4_096, 2),
        }
    }

    pub fn install_panic_hook(&self) {
        install_safe_panic_hook(self.inner.clone(), Component::SecurityCenter);
    }

    pub fn write(&self, level: &str, code: &str, public_message: &str) -> CoreResult<()> {
        self.inner.write(level, code, public_message)
    }

    #[must_use]
    pub fn read_last_failure(&self) -> Option<String> {
        self.inner.read_last_failure()
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        self.inner.path()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

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

    #[test]
    fn logger_redacts_obvious_secret_markers() {
        let dir = tempdir().expect("temporary directory");
        let path = dir.path().join("security-center.log");
        let logger = SafeLogger::from_path(path.clone(), LogPolicy::default());
        logger
            .write("error", "test.secret", "password=hunter2")
            .expect("write");
        let content = fs::read_to_string(path).expect("read");
        assert!(content.contains("[REDACTED]"));
        assert!(!content.contains("hunter2"));
    }
}
