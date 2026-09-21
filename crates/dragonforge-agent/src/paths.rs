use std::path::{Path, PathBuf};

use dragonforge_core::{Component, SuitePaths};

use crate::error::{AgentError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentPaths {
    root: PathBuf,
}

impl AgentPaths {
    #[must_use]
    pub fn from_root(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn discover() -> Result<Self> {
        let suite = SuitePaths::discover()
            .map_err(|_| AgentError::InvalidState("DragonForge data paths are unavailable"))?;
        Ok(Self::from_root(suite.component_data_dir(Component::Agent)))
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub fn runtime_file(&self) -> PathBuf {
        self.root.join("agent-runtime.json")
    }

    #[must_use]
    pub fn credential_file(&self) -> PathBuf {
        self.root.join("agent-session.key")
    }

    #[must_use]
    pub fn lock_file(&self) -> PathBuf {
        self.root.join("agent.lock")
    }
}
