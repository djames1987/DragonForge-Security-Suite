use std::path::PathBuf;

use dragonforge_core::{Component, SuitePaths};
use dragonforge_integrity_monitor::{
    ContinuousCheckOutcome, ContinuousIntegrityEvent, ContinuousMonitorStatus, continuous_events,
    continuous_status, run_continuous_check_if_due,
};

use crate::error::{AgentError, Result};

#[derive(Debug, Clone)]
pub struct AgentIntegrityRuntime {
    state_path: PathBuf,
    baseline_path: PathBuf,
}

impl AgentIntegrityRuntime {
    pub fn discover() -> Result<Self> {
        let suite = SuitePaths::discover()
            .map_err(|_| AgentError::InvalidState("DragonForge data paths are unavailable"))?;
        Ok(Self {
            state_path: suite
                .component_data_dir(Component::Agent)
                .join("continuous-integrity-v1.json"),
            baseline_path: suite
                .component_data_dir(Component::IntegrityMonitor)
                .join("baseline-v1.json"),
        })
    }

    #[must_use]
    pub fn from_paths(state_path: PathBuf, baseline_path: PathBuf) -> Self {
        Self {
            state_path,
            baseline_path,
        }
    }

    pub fn tick(&self) -> Result<ContinuousCheckOutcome> {
        run_continuous_check_if_due(&self.state_path, &self.baseline_path)
            .map_err(|_| AgentError::InvalidState("continuous integrity monitoring failed"))
    }

    pub fn status(&self) -> Result<ContinuousMonitorStatus> {
        continuous_status(&self.state_path, &self.baseline_path)
            .map_err(|_| AgentError::InvalidState("continuous integrity status is unavailable"))
    }

    pub fn events(
        &self,
        after_id: u64,
        limit: usize,
    ) -> Result<Vec<ContinuousIntegrityEvent>> {
        continuous_events(&self.state_path, after_id, limit)
            .map_err(|_| AgentError::InvalidState("continuous integrity events are unavailable"))
    }
}
