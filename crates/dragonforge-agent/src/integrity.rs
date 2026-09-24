use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::Duration;

use dragonforge_core::{Component, SuitePaths};
use dragonforge_integrity_monitor::{
    ContinuousCheckOutcome, ContinuousIntegrityEvent, ContinuousMonitorStatus, continuous_events,
    continuous_status, run_continuous_check_if_due, run_continuous_check_now,
};

use crate::error::{AgentError, Result};

#[derive(Debug, Clone)]
pub struct AgentIntegrityRuntime {
    state_path: PathBuf,
    baseline_path: PathBuf,
    lock_path: PathBuf,
}

impl AgentIntegrityRuntime {
    pub fn discover() -> Result<Self> {
        let suite = SuitePaths::discover()
            .map_err(|_| AgentError::InvalidState("DragonForge data paths are unavailable"))?;
        let state_path = suite
            .component_data_dir(Component::Agent)
            .join("continuous-integrity-v1.json");
        let baseline_path = suite
            .component_data_dir(Component::IntegrityMonitor)
            .join("baseline-v1.json");
        Ok(Self::from_paths(state_path, baseline_path))
    }

    #[must_use]
    pub fn from_paths(state_path: PathBuf, baseline_path: PathBuf) -> Self {
        let lock_path = append_suffix(&state_path, ".agent.lock");
        Self {
            state_path,
            baseline_path,
            lock_path,
        }
    }

    pub fn tick(&self) -> Result<ContinuousCheckOutcome> {
        let _guard = acquire_integrity_lock(&self.lock_path)?;
        run_continuous_check_if_due(&self.state_path, &self.baseline_path)
            .map_err(|_| AgentError::InvalidState("continuous integrity monitoring failed"))
    }

    pub fn run_now(&self) -> Result<ContinuousCheckOutcome> {
        let _guard = acquire_integrity_lock(&self.lock_path)?;
        run_continuous_check_now(&self.state_path, &self.baseline_path)
            .map_err(|_| AgentError::InvalidState("scheduled integrity check failed"))
    }

    pub fn status(&self) -> Result<ContinuousMonitorStatus> {
        continuous_status(&self.state_path, &self.baseline_path)
            .map_err(|_| AgentError::InvalidState("continuous integrity status is unavailable"))
    }

    pub fn events(&self, after_id: u64, limit: usize) -> Result<Vec<ContinuousIntegrityEvent>> {
        continuous_events(&self.state_path, after_id, limit)
            .map_err(|_| AgentError::InvalidState("continuous integrity events are unavailable"))
    }
}


const INTEGRITY_LOCK_STALE_AFTER: Duration = Duration::from_secs(30);

fn acquire_integrity_lock(path: &Path) -> Result<IntegrityRuntimeLock> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|_| AgentError::Io("integrity runtime lock directory could not be created"))?;
    }
    let open = || OpenOptions::new().write(true).create_new(true).open(path);
    let file = match open() {
        Ok(file) => file,
        Err(_) => {
            let stale = fs::metadata(path)
                .ok()
                .and_then(|metadata| metadata.modified().ok())
                .and_then(|modified| modified.elapsed().ok())
                .is_some_and(|age| age >= INTEGRITY_LOCK_STALE_AFTER);
            if !stale {
                return Err(AgentError::Unavailable("integrity runtime state is busy"));
            }
            let _ = fs::remove_file(path);
            open().map_err(|_| AgentError::Unavailable("integrity runtime state is busy"))?
        }
    };
    Ok(IntegrityRuntimeLock {
        path: path.to_path_buf(),
        _file: file,
    })
}

struct IntegrityRuntimeLock {
    path: PathBuf,
    _file: fs::File,
}

impl Drop for IntegrityRuntimeLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn append_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(suffix);
    PathBuf::from(value)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::AgentIntegrityRuntime;

    #[test]
    fn concurrent_mutating_checks_fail_closed_when_lock_is_held() {
        let dir = tempdir().expect("tempdir");
        let runtime = AgentIntegrityRuntime::from_paths(
            dir.path().join("continuous.json"),
            dir.path().join("baseline.json"),
        );
        let lock_path = dir.path().join("continuous.json.agent.lock");
        std::fs::write(&lock_path, b"held").expect("hold lock");
        assert!(runtime.tick().is_err());
        assert!(runtime.run_now().is_err());
    }
}
