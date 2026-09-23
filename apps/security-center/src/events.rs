use std::collections::VecDeque;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use dragonforge_core::{
    Component, CoreError, CoreResult, ErrorCode, EventKind, EventRecord, Severity, SuitePaths,
};
use serde::{Deserialize, Serialize};

const EVENT_HUB_VERSION: u32 = 1;
const EVENT_HUB_FILE: &str = "event-hub-v1.json";
const MAX_EVENT_HUB_BYTES: usize = 2 * 1024 * 1024;
const MAX_EVENT_CAPACITY: usize = 2_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardEvent {
    pub id: u64,
    pub timestamp_ms: u64,
    pub component: String,
    pub kind: String,
    pub severity: String,
    pub code: String,
    pub summary: String,
    pub status: String,
    pub acknowledged_at_ms: Option<u64>,
}

impl From<EventRecord> for DashboardEvent {
    fn from(event: EventRecord) -> Self {
        let timestamp_ms = event
            .timestamp()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            .try_into()
            .unwrap_or(u64::MAX);

        Self {
            id: event.event_id().try_into().unwrap_or(u64::MAX),
            timestamp_ms,
            component: event.component().as_str().to_owned(),
            kind: event.kind().as_str().to_owned(),
            severity: event.severity().as_str().to_owned(),
            code: event.code().to_owned(),
            summary: event.safe_summary().to_owned(),
            status: "open".to_owned(),
            acknowledged_at_ms: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedEventHub {
    version: u32,
    next_id: u64,
    events: VecDeque<DashboardEvent>,
}

#[derive(Debug)]
pub struct EventStore {
    events: VecDeque<DashboardEvent>,
    capacity: usize,
    next_id: u64,
    path: Option<PathBuf>,
}

impl EventStore {
    pub fn discover(capacity: usize) -> CoreResult<Self> {
        let paths = SuitePaths::discover()?;
        Self::from_path(
            paths
                .component_data_dir(Component::SecurityCenter)
                .join(EVENT_HUB_FILE),
            capacity,
        )
    }

    pub fn from_dir(directory: impl Into<PathBuf>, capacity: usize) -> CoreResult<Self> {
        Self::from_path(directory.into().join(EVENT_HUB_FILE), capacity)
    }

    fn from_path(path: PathBuf, capacity: usize) -> CoreResult<Self> {
        let capacity = normalize_capacity(capacity);
        recover_backup(&path)?;
        if !path.exists() {
            return Ok(Self {
                events: VecDeque::with_capacity(capacity),
                capacity,
                next_id: 1,
                path: Some(path),
            });
        }

        let bytes = fs::read(&path).map_err(|_| {
            CoreError::new_safe(
                ErrorCode::InvalidConfiguration,
                "unable to read Security Center event history",
            )
        })?;
        if bytes.len() > MAX_EVENT_HUB_BYTES {
            return Err(CoreError::new_safe(
                ErrorCode::InvalidConfiguration,
                "Security Center event history exceeded its safe size limit",
            ));
        }
        let persisted: PersistedEventHub = serde_json::from_slice(&bytes).map_err(|_| {
            CoreError::new_safe(
                ErrorCode::InvalidConfiguration,
                "Security Center event history is invalid",
            )
        })?;
        if persisted.version != EVENT_HUB_VERSION
            || persisted.next_id == 0
            || persisted.events.len() > MAX_EVENT_CAPACITY
            || persisted.events.iter().any(|event| !valid_event(event))
        {
            return Err(CoreError::new_safe(
                ErrorCode::InvalidConfiguration,
                "Security Center event history is incompatible",
            ));
        }

        let mut store = Self {
            events: persisted.events,
            capacity,
            next_id: persisted.next_id,
            path: Some(path),
        };
        store.trim();
        Ok(store)
    }

    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let capacity = normalize_capacity(capacity);
        Self {
            events: VecDeque::with_capacity(capacity),
            capacity,
            next_id: 1,
            path: None,
        }
    }

    pub fn push(
        &mut self,
        component: Component,
        kind: EventKind,
        severity: Severity,
        code: impl Into<String>,
        safe_summary: impl Into<String>,
    ) -> CoreResult<()> {
        let code = single_line(code.into(), 160);
        let summary = single_line(safe_summary.into(), 512);
        if code.is_empty() || summary.is_empty() {
            return Err(CoreError::new_safe(
                ErrorCode::InvalidConfiguration,
                "Security Center event metadata is invalid",
            ));
        }

        let record = EventRecord::new(
            u128::from(self.next_id),
            component,
            kind,
            severity,
            code,
            summary,
        );
        self.next_id = self.next_id.saturating_add(1).max(1);
        self.events.push_back(record.into());
        self.trim();
        self.persist()
    }

    #[must_use]
    pub fn recent(&self, limit: usize) -> Vec<DashboardEvent> {
        self.events
            .iter()
            .rev()
            .take(limit.min(MAX_EVENT_CAPACITY))
            .cloned()
            .collect()
    }

    #[must_use]
    pub fn notifications(&self, minimum_severity: &str, limit: usize) -> Vec<DashboardEvent> {
        let threshold = severity_rank(minimum_severity);
        self.events
            .iter()
            .rev()
            .filter(|event| {
                event.status == "open" && severity_rank(&event.severity) >= threshold
            })
            .take(limit.min(MAX_EVENT_CAPACITY))
            .cloned()
            .collect()
    }

    pub fn acknowledge(&mut self, event_id: u64) -> CoreResult<bool> {
        let Some(event) = self.events.iter_mut().find(|event| event.id == event_id) else {
            return Ok(false);
        };
        if event.status == "acknowledged" {
            return Ok(false);
        }
        event.status = "acknowledged".to_owned();
        event.acknowledged_at_ms = Some(now_ms());
        self.persist()?;
        Ok(true)
    }

    pub fn acknowledge_all(&mut self, minimum_severity: &str) -> CoreResult<usize> {
        let threshold = severity_rank(minimum_severity);
        let acknowledged_at_ms = now_ms();
        let mut count = 0usize;
        for event in &mut self.events {
            if event.status == "open" && severity_rank(&event.severity) >= threshold {
                event.status = "acknowledged".to_owned();
                event.acknowledged_at_ms = Some(acknowledged_at_ms);
                count = count.saturating_add(1);
            }
        }
        if count > 0 {
            self.persist()?;
        }
        Ok(count)
    }

    pub fn clear(&mut self) -> CoreResult<()> {
        self.events.clear();
        self.persist()
    }

    pub fn set_capacity(&mut self, capacity: usize) -> CoreResult<()> {
        self.capacity = normalize_capacity(capacity);
        self.trim();
        self.persist()
    }

    fn trim(&mut self) {
        while self.events.len() > self.capacity {
            self.events.pop_front();
        }
    }

    fn persist(&self) -> CoreResult<()> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        let parent = path.parent().ok_or_else(|| {
            CoreError::new_safe(
                ErrorCode::Internal,
                "Security Center event history path has no parent",
            )
        })?;
        fs::create_dir_all(parent).map_err(|_| {
            CoreError::new_safe(
                ErrorCode::Internal,
                "unable to create Security Center event history directory",
            )
        })?;

        let payload = PersistedEventHub {
            version: EVENT_HUB_VERSION,
            next_id: self.next_id,
            events: self.events.clone(),
        };
        let encoded = serde_json::to_vec_pretty(&payload).map_err(|_| {
            CoreError::new_safe(
                ErrorCode::Internal,
                "unable to encode Security Center event history",
            )
        })?;
        if encoded.len() > MAX_EVENT_HUB_BYTES {
            return Err(CoreError::new_safe(
                ErrorCode::Internal,
                "Security Center event history exceeded its safe size limit",
            ));
        }

        replace_file(path, &encoded)
    }
}

impl Default for EventStore {
    fn default() -> Self {
        Self::new(250)
    }
}

fn normalize_capacity(capacity: usize) -> usize {
    capacity.clamp(1, MAX_EVENT_CAPACITY)
}

fn valid_event(event: &DashboardEvent) -> bool {
    event.id > 0
        && event.component.len() <= 64
        && event.kind.len() <= 32
        && event.severity.len() <= 16
        && event.code.len() <= 160
        && event.summary.len() <= 512
        && matches!(event.status.as_str(), "open" | "acknowledged")
}

fn severity_rank(value: &str) -> u8 {
    match value {
        "critical" => 5,
        "warning" => 4,
        "notice" => 3,
        "info" => 2,
        "debug" => 1,
        _ => 0,
    }
}

fn single_line(value: String, max_chars: usize) -> String {
    value
        .chars()
        .filter(|value| !matches!(value, '\r' | '\n' | '\0'))
        .take(max_chars)
        .collect::<String>()
        .trim()
        .to_owned()
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            duration.as_millis().min(u128::from(u64::MAX)) as u64
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
                "unable to recover Security Center event history",
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
                "unable to create temporary Security Center event history",
            )
        })?;
        file.write_all(encoded).map_err(|_| {
            CoreError::new_safe(
                ErrorCode::Internal,
                "unable to write Security Center event history",
            )
        })?;
        file.sync_all().map_err(|_| {
            CoreError::new_safe(
                ErrorCode::Internal,
                "unable to flush Security Center event history",
            )
        })?;
    }

    if backup.exists() {
        fs::remove_file(&backup).map_err(|_| {
            CoreError::new_safe(
                ErrorCode::Internal,
                "unable to clear stale Security Center event backup",
            )
        })?;
    }
    if path.exists() {
        fs::rename(path, &backup).map_err(|_| {
            CoreError::new_safe(
                ErrorCode::Internal,
                "unable to stage Security Center event history replacement",
            )
        })?;
    }
    if let Err(_) = fs::rename(&temporary, path) {
        if backup.exists() && !path.exists() {
            let _ = fs::rename(&backup, path);
        }
        return Err(CoreError::new_safe(
            ErrorCode::Internal,
            "unable to finalize Security Center event history",
        ));
    }
    if backup.exists() {
        fs::remove_file(backup).map_err(|_| {
            CoreError::new_safe(
                ErrorCode::Internal,
                "unable to clear Security Center event history backup",
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

#[cfg(test)]
mod tests {
    use dragonforge_core::{Component, EventKind, Severity};
    use tempfile::tempdir;

    use super::EventStore;

    #[test]
    fn event_store_is_bounded_persistent_and_returns_newest_first() {
        let dir = tempdir().expect("temporary directory");
        let mut store = EventStore::from_dir(dir.path(), 2).expect("store");
        for code in ["one", "two", "three"] {
            store
                .push(
                    Component::SecurityCenter,
                    EventKind::Health,
                    Severity::Info,
                    code,
                    code,
                )
                .expect("push");
        }

        let recent = store.recent(10);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].code, "three");
        assert_eq!(recent[1].code, "two");

        let reloaded = EventStore::from_dir(dir.path(), 2).expect("reload");
        assert_eq!(reloaded.recent(10), recent);
    }

    #[test]
    fn acknowledgement_persists_and_filters_notifications() {
        let dir = tempdir().expect("temporary directory");
        let mut store = EventStore::from_dir(dir.path(), 10).expect("store");
        store
            .push(
                Component::IntegrityMonitor,
                EventKind::Security,
                Severity::Warning,
                "integrity.change",
                "Integrity change detected",
            )
            .expect("push");
        let id = store.recent(1)[0].id;
        assert_eq!(store.notifications("warning", 10).len(), 1);
        assert!(store.acknowledge(id).expect("acknowledge"));
        assert!(store.notifications("warning", 10).is_empty());

        let reloaded = EventStore::from_dir(dir.path(), 10).expect("reload");
        assert_eq!(reloaded.recent(1)[0].status, "acknowledged");
    }

    #[test]
    fn interrupted_replacement_recovers_backup() {
        let dir = tempdir().expect("temporary directory");
        let path = dir.path().join(super::EVENT_HUB_FILE);
        let mut store = EventStore::from_dir(dir.path(), 10).expect("store");
        store
            .push(
                Component::SecurityCenter,
                EventKind::Lifecycle,
                Severity::Info,
                "one",
                "one",
            )
            .expect("push");
        let backup = super::backup_path(&path);
        fs::rename(&path, &backup).expect("simulate interruption");
        assert!(!path.exists());

        let recovered = EventStore::from_dir(dir.path(), 10).expect("recover");
        assert_eq!(recovered.recent(10).len(), 1);
        assert!(path.exists());
        assert!(!backup.exists());
    }

    #[test]
    fn shrinking_capacity_evicts_oldest_events() {
        let mut store = EventStore::new(4);
        for code in ["one", "two", "three", "four"] {
            store
                .push(
                    Component::SecurityCenter,
                    EventKind::Health,
                    Severity::Info,
                    code,
                    code,
                )
                .expect("push");
        }
        store.set_capacity(2).expect("capacity");
        let recent = store.recent(10);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].code, "four");
        assert_eq!(recent[1].code, "three");
    }
}
