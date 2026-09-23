use std::collections::VecDeque;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use dragonforge_core::{Component, CoreError, CoreResult, ErrorCode, SuitePaths};
use serde::{Deserialize, Serialize};

use crate::model::{ComponentStatus, HealthSummary};

const HEALTH_HISTORY_VERSION: u32 = 1;
const HEALTH_HISTORY_FILE: &str = "health-history-v1.json";
const MAX_HEALTH_HISTORY: usize = 500;
const MAX_HEALTH_HISTORY_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentHealthPoint {
    pub id: String,
    pub state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthHistoryEntry {
    pub timestamp_ms: u64,
    pub suite_state: String,
    pub active: usize,
    pub integrated: usize,
    pub attention: usize,
    pub components: Vec<ComponentHealthPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedHealthHistory {
    version: u32,
    entries: VecDeque<HealthHistoryEntry>,
}

#[derive(Debug)]
pub struct HealthHistoryStore {
    entries: VecDeque<HealthHistoryEntry>,
    capacity: usize,
    path: Option<PathBuf>,
}

impl HealthHistoryStore {
    pub fn discover(capacity: usize) -> CoreResult<Self> {
        let paths = SuitePaths::discover()?;
        let path = paths
            .component_data_dir(Component::SecurityCenter)
            .join(HEALTH_HISTORY_FILE);
        match Self::from_path(path.clone(), capacity) {
            Ok(store) => Ok(store),
            Err(_) => {
                quarantine_invalid(&path)?;
                Ok(Self {
                    entries: VecDeque::with_capacity(capacity.clamp(10, MAX_HEALTH_HISTORY)),
                    capacity: capacity.clamp(10, MAX_HEALTH_HISTORY),
                    path: Some(path),
                })
            }
        }
    }

    pub fn from_dir(directory: impl Into<PathBuf>, capacity: usize) -> CoreResult<Self> {
        Self::from_path(directory.into().join(HEALTH_HISTORY_FILE), capacity)
    }

    fn from_path(path: PathBuf, capacity: usize) -> CoreResult<Self> {
        let capacity = capacity.clamp(10, MAX_HEALTH_HISTORY);
        recover_backup(&path)?;
        if !path.exists() {
            return Ok(Self {
                entries: VecDeque::with_capacity(capacity),
                capacity,
                path: Some(path),
            });
        }

        let bytes = fs::read(&path).map_err(|_| {
            CoreError::new_safe(
                ErrorCode::InvalidConfiguration,
                "unable to read Security Center health history",
            )
        })?;
        if bytes.len() > MAX_HEALTH_HISTORY_BYTES {
            return Err(CoreError::new_safe(
                ErrorCode::InvalidConfiguration,
                "Security Center health history exceeded its safe size limit",
            ));
        }
        let persisted: PersistedHealthHistory = serde_json::from_slice(&bytes).map_err(|_| {
            CoreError::new_safe(
                ErrorCode::InvalidConfiguration,
                "Security Center health history is invalid",
            )
        })?;
        if persisted.version != HEALTH_HISTORY_VERSION
            || persisted.entries.len() > MAX_HEALTH_HISTORY
        {
            return Err(CoreError::new_safe(
                ErrorCode::InvalidConfiguration,
                "Security Center health history is incompatible",
            ));
        }

        let mut store = Self {
            entries: persisted.entries,
            capacity,
            path: Some(path),
        };
        store.trim();
        Ok(store)
    }

    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.clamp(10, MAX_HEALTH_HISTORY);
        Self {
            entries: VecDeque::with_capacity(capacity),
            capacity,
            path: None,
        }
    }

    pub fn record(
        &mut self,
        health: &HealthSummary,
        components: &[ComponentStatus],
    ) -> CoreResult<bool> {
        let entry = HealthHistoryEntry {
            timestamp_ms: now_ms(),
            suite_state: health.state.to_owned(),
            active: health.active,
            integrated: health.integrated,
            attention: health.attention,
            components: components
                .iter()
                .map(|component| ComponentHealthPoint {
                    id: component.id.to_owned(),
                    state: component.state.as_str().to_owned(),
                })
                .collect(),
        };

        if self
            .entries
            .back()
            .is_some_and(|previous| equivalent(previous, &entry))
        {
            return Ok(false);
        }

        self.entries.push_back(entry);
        self.trim();
        self.persist()?;
        Ok(true)
    }

    #[must_use]
    pub fn recent(&self, limit: usize) -> Vec<HealthHistoryEntry> {
        self.entries
            .iter()
            .rev()
            .take(limit.min(MAX_HEALTH_HISTORY))
            .cloned()
            .collect()
    }

    pub fn set_capacity(&mut self, capacity: usize) -> CoreResult<()> {
        self.capacity = capacity.clamp(10, MAX_HEALTH_HISTORY);
        self.trim();
        self.persist()
    }

    fn trim(&mut self) {
        while self.entries.len() > self.capacity {
            self.entries.pop_front();
        }
    }

    fn persist(&self) -> CoreResult<()> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        let parent = path.parent().ok_or_else(|| {
            CoreError::new_safe(
                ErrorCode::Internal,
                "Security Center health history path has no parent",
            )
        })?;
        fs::create_dir_all(parent).map_err(|_| {
            CoreError::new_safe(
                ErrorCode::Internal,
                "unable to create Security Center health history directory",
            )
        })?;
        let encoded = serde_json::to_vec_pretty(&PersistedHealthHistory {
            version: HEALTH_HISTORY_VERSION,
            entries: self.entries.clone(),
        })
        .map_err(|_| {
            CoreError::new_safe(
                ErrorCode::Internal,
                "unable to encode Security Center health history",
            )
        })?;
        if encoded.len() > MAX_HEALTH_HISTORY_BYTES {
            return Err(CoreError::new_safe(
                ErrorCode::Internal,
                "Security Center health history exceeded its safe size limit",
            ));
        }
        replace_file(path, &encoded)
    }
}

fn equivalent(left: &HealthHistoryEntry, right: &HealthHistoryEntry) -> bool {
    left.suite_state == right.suite_state
        && left.active == right.active
        && left.integrated == right.integrated
        && left.attention == right.attention
        && left.components == right.components
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            duration.as_millis().min(u128::from(u64::MAX)) as u64
        })
}

fn quarantine_invalid(path: &Path) -> CoreResult<()> {
    if !path.exists() {
        return Ok(());
    }
    let invalid = invalid_path(path);
    if invalid.exists() {
        fs::remove_file(&invalid).map_err(|_| {
            CoreError::new_safe(
                ErrorCode::Internal,
                "unable to replace invalid Security Center health-history quarantine",
            )
        })?;
    }
    fs::rename(path, invalid).map_err(|_| {
        CoreError::new_safe(
            ErrorCode::Internal,
            "unable to quarantine invalid Security Center health history",
        )
    })
}

fn recover_backup(path: &Path) -> CoreResult<()> {
    if path.exists() {
        return Ok(());
    }
    let backup = backup_path(path);
    if backup.exists() {
        fs::rename(&backup, path).map_err(|_| {
            CoreError::new_safe(
                ErrorCode::Internal,
                "unable to recover Security Center health history",
            )
        })?;
    }
    Ok(())
}

fn replace_file(path: &Path, encoded: &[u8]) -> CoreResult<()> {
    let temporary = temporary_path(path);
    let backup = backup_path(path);
    {
        let mut file = fs::File::create(&temporary).map_err(|_| {
            CoreError::new_safe(
                ErrorCode::Internal,
                "unable to create temporary Security Center health history",
            )
        })?;
        file.write_all(encoded).map_err(|_| {
            CoreError::new_safe(
                ErrorCode::Internal,
                "unable to write Security Center health history",
            )
        })?;
        file.sync_all().map_err(|_| {
            CoreError::new_safe(
                ErrorCode::Internal,
                "unable to flush Security Center health history",
            )
        })?;
    }

    if backup.exists() {
        fs::remove_file(&backup).map_err(|_| {
            CoreError::new_safe(
                ErrorCode::Internal,
                "unable to clear stale Security Center health-history backup",
            )
        })?;
    }
    if path.exists() {
        fs::rename(path, &backup).map_err(|_| {
            CoreError::new_safe(
                ErrorCode::Internal,
                "unable to stage Security Center health-history replacement",
            )
        })?;
    }
    if fs::rename(&temporary, path).is_err() {
        if backup.exists() && !path.exists() {
            let _ = fs::rename(&backup, path);
        }
        return Err(CoreError::new_safe(
            ErrorCode::Internal,
            "unable to finalize Security Center health history",
        ));
    }
    if backup.exists() {
        fs::remove_file(backup).map_err(|_| {
            CoreError::new_safe(
                ErrorCode::Internal,
                "unable to clear Security Center health-history backup",
            )
        })?;
    }
    Ok(())
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(".tmp");
    PathBuf::from(value)
}

fn backup_path(path: &Path) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(".bak");
    PathBuf::from(value)
}

fn invalid_path(path: &Path) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(".invalid");
    PathBuf::from(value)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::model::ComponentRegistry;

    #[test]
    fn unchanged_health_is_deduplicated_and_persists() {
        let dir = tempdir().expect("temporary directory");
        let mut store = HealthHistoryStore::from_dir(dir.path(), 25).expect("store");
        let registry = ComponentRegistry::phase3_default();
        let health = registry.health_summary();
        assert!(store.record(&health, registry.all()).expect("record"));
        assert!(!store.record(&health, registry.all()).expect("deduplicate"));
        assert_eq!(store.recent(10).len(), 1);

        let reloaded = HealthHistoryStore::from_dir(dir.path(), 25).expect("reload");
        assert_eq!(reloaded.recent(10).len(), 1);
    }
}
