use std::collections::{BTreeSet, VecDeque};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use dragonforge_core::{Component, SuitePaths};
use dragonforge_security_scanner::scan_system;
use serde::{Deserialize, Serialize};

use crate::error::{AgentError, Result};
use crate::AgentIntegrityRuntime;

pub const AUTOMATION_STATE_VERSION: u16 = 1;
pub const MIN_INTERVAL_MINUTES: u64 = 15;
pub const MAX_INTERVAL_MINUTES: u64 = 10_080;
pub const MAX_AUTOMATION_EVENTS: usize = 250;
const MAX_AUTOMATION_STATE_BYTES: usize = 1024 * 1024;
const RETRY_DELAY_MS: u64 = 60_000;
const MAX_CONSECUTIVE_RETRIES: u8 = 3;
const LOCK_STALE_AFTER: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutomationJobKind {
    SecurityScan,
    IntegrityCheck,
    BackupReminder,
}

impl AutomationJobKind {
    pub const ALL: [Self; 3] = [
        Self::SecurityScan,
        Self::IntegrityCheck,
        Self::BackupReminder,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SecurityScan => "security_scan",
            Self::IntegrityCheck => "integrity_check",
            Self::BackupReminder => "backup_reminder",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "security_scan" => Ok(Self::SecurityScan),
            "integrity_check" => Ok(Self::IntegrityCheck),
            "backup_reminder" => Ok(Self::BackupReminder),
            _ => Err(AgentError::InvalidState(
                "automation job is not an approved DragonForge capability",
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationJob {
    pub kind: AutomationJobKind,
    pub enabled: bool,
    pub interval_minutes: u64,
    pub next_run_ms: Option<u64>,
    pub last_run_ms: Option<u64>,
    pub consecutive_failures: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationEvent {
    pub id: u64,
    pub timestamp_ms: u64,
    pub job: AutomationJobKind,
    pub outcome: String,
    pub summary: String,
    pub attempt: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomationStatus {
    pub configured: bool,
    pub jobs: Vec<AutomationJob>,
    pub history: Vec<AutomationEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedAutomationState {
    version: u16,
    jobs: Vec<AutomationJob>,
    next_event_id: u64,
    history: VecDeque<AutomationEvent>,
}

impl Default for PersistedAutomationState {
    fn default() -> Self {
        Self {
            version: AUTOMATION_STATE_VERSION,
            jobs: vec![
                AutomationJob {
                    kind: AutomationJobKind::SecurityScan,
                    enabled: false,
                    interval_minutes: 1_440,
                    next_run_ms: None,
                    last_run_ms: None,
                    consecutive_failures: 0,
                },
                AutomationJob {
                    kind: AutomationJobKind::IntegrityCheck,
                    enabled: false,
                    interval_minutes: 60,
                    next_run_ms: None,
                    last_run_ms: None,
                    consecutive_failures: 0,
                },
                AutomationJob {
                    kind: AutomationJobKind::BackupReminder,
                    enabled: false,
                    interval_minutes: 10_080,
                    next_run_ms: None,
                    last_run_ms: None,
                    consecutive_failures: 0,
                },
            ],
            next_event_id: 1,
            history: VecDeque::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentAutomationRuntime {
    state_path: PathBuf,
    lock_path: PathBuf,
    integrity: AgentIntegrityRuntime,
}

impl AgentAutomationRuntime {
    pub fn discover() -> Result<Self> {
        let suite = SuitePaths::discover()
            .map_err(|_| AgentError::InvalidState("DragonForge data paths are unavailable"))?;
        let root = suite.component_data_dir(Component::Agent);
        Ok(Self::from_paths(
            root.join("scheduled-automation-v1.json"),
            root.join("scheduled-automation-v1.lock"),
            AgentIntegrityRuntime::discover()?,
        ))
    }

    #[must_use]
    pub fn from_paths(
        state_path: PathBuf,
        lock_path: PathBuf,
        integrity: AgentIntegrityRuntime,
    ) -> Self {
        Self {
            state_path,
            lock_path,
            integrity,
        }
    }

    pub fn status(&self) -> Result<AutomationStatus> {
        let _guard = acquire_lock(&self.lock_path)?;
        let (state, configured) = self.load_or_recover()?;
        Ok(status_from_state(&state, configured))
    }

    pub fn configure_job(
        &self,
        kind: AutomationJobKind,
        enabled: bool,
        interval_minutes: u64,
    ) -> Result<AutomationStatus> {
        validate_interval(interval_minutes)?;
        let _guard = acquire_lock(&self.lock_path)?;
        let (mut state, _) = self.load_or_recover()?;
        let now = now_ms();
        let job = state
            .jobs
            .iter_mut()
            .find(|job| job.kind == kind)
            .ok_or(AgentError::InvalidState("automation job policy is incomplete"))?;
        job.enabled = enabled;
        job.interval_minutes = interval_minutes;
        job.next_run_ms = enabled.then_some(now);
        job.consecutive_failures = 0;
        persist_state(&self.state_path, &state)?;
        Ok(status_from_state(&state, true))
    }

    pub fn run_now(&self, kind: AutomationJobKind) -> Result<AutomationStatus> {
        let _guard = acquire_lock(&self.lock_path)?;
        let (mut state, _) = self.load_or_recover()?;
        let now = now_ms();
        self.execute_and_record(&mut state, kind, now, false)?;
        persist_state(&self.state_path, &state)?;
        Ok(status_from_state(&state, true))
    }

    pub fn tick(&self) -> Result<AutomationStatus> {
        let _guard = acquire_lock(&self.lock_path)?;
        let (mut state, configured) = self.load_or_recover()?;
        if !configured {
            return Ok(status_from_state(&state, false));
        }

        let now = now_ms();
        let due = state
            .jobs
            .iter()
            .filter(|job| job.enabled && job.next_run_ms.is_some_and(|next| now >= next))
            .map(|job| job.kind)
            .collect::<Vec<_>>();
        for kind in due {
            let missed = state
                .jobs
                .iter()
                .find(|job| job.kind == kind)
                .and_then(|job| job.next_run_ms.map(|next| (job, next)))
                .is_some_and(|(job, next)| {
                    now.saturating_sub(next) > interval_ms(job.interval_minutes)
                });
            self.execute_and_record(&mut state, kind, now, missed)?;
        }
        persist_state(&self.state_path, &state)?;
        Ok(status_from_state(&state, true))
    }

    fn execute_and_record(
        &self,
        state: &mut PersistedAutomationState,
        kind: AutomationJobKind,
        now: u64,
        missed: bool,
    ) -> Result<()> {
        let result = self.execute(kind);
        let job = state
            .jobs
            .iter_mut()
            .find(|job| job.kind == kind)
            .ok_or(AgentError::InvalidState("automation job policy is incomplete"))?;
        job.last_run_ms = Some(now);

        let (outcome, summary, retryable) = match result {
            Ok(result) => {
                job.consecutive_failures = 0;
                (
                    if missed && result.outcome == "success" {
                        "missed_recovered".to_owned()
                    } else {
                        result.outcome
                    },
                    if missed {
                        format!("Recovered overdue automation job. {}", result.summary)
                    } else {
                        result.summary
                    },
                    false,
                )
            }
            Err(_) => {
                job.consecutive_failures = job
                    .consecutive_failures
                    .saturating_add(1)
                    .min(MAX_CONSECUTIVE_RETRIES.saturating_add(1));
                (
                    "failed".to_owned(),
                    "Scheduled protection job failed safely; no generic fallback was executed."
                        .to_owned(),
                    true,
                )
            }
        };

        if job.enabled {
            job.next_run_ms = Some(if retryable && job.consecutive_failures <= MAX_CONSECUTIVE_RETRIES
            {
                now.saturating_add(RETRY_DELAY_MS)
            } else {
                now.saturating_add(interval_ms(job.interval_minutes))
            });
        } else {
            job.next_run_ms = None;
        }

        let event = AutomationEvent {
            id: state.next_event_id.max(1),
            timestamp_ms: now,
            job: kind,
            outcome,
            summary: single_line(summary, 512),
            attempt: job.consecutive_failures.max(1),
        };
        state.next_event_id = event.id.saturating_add(1);
        state.history.push_back(event);
        while state.history.len() > MAX_AUTOMATION_EVENTS {
            state.history.pop_front();
        }
        Ok(())
    }

    fn execute(&self, kind: AutomationJobKind) -> Result<ExecutionResult> {
        match kind {
            AutomationJobKind::SecurityScan => {
                let report = scan_system();
                let outcome = if report.summary.attention > 0 {
                    "attention"
                } else {
                    "success"
                };
                Ok(ExecutionResult {
                    outcome: outcome.to_owned(),
                    summary: format!(
                        "Scheduled Security Scanner completed: {} pass, {} attention, {} unknown, {} informational.",
                        report.summary.pass,
                        report.summary.attention,
                        report.summary.unknown,
                        report.summary.informational
                    ),
                })
            }
            AutomationJobKind::IntegrityCheck => {
                let outcome = self.integrity.run_now()?;
                if !outcome.status.configured {
                    return Ok(ExecutionResult {
                        outcome: "attention".to_owned(),
                        summary:
                            "Scheduled integrity check needs an Integrity Monitor baseline and policy."
                                .to_owned(),
                    });
                }
                Ok(ExecutionResult {
                    outcome: if outcome.new_events > 0 {
                        "attention".to_owned()
                    } else {
                        "success".to_owned()
                    },
                    summary: format!(
                        "Scheduled integrity check completed with {} new retained change event(s).",
                        outcome.new_events
                    ),
                })
            }
            AutomationJobKind::BackupReminder => Ok(ExecutionResult {
                outcome: "action_required".to_owned(),
                summary: "Encrypted backup is due. Open Backup & Recovery to create a password-protected backup; DragonForge does not persist a backup password for unattended execution.".to_owned(),
            }),
        }
    }

    fn load_or_recover(&self) -> Result<(PersistedAutomationState, bool)> {
        match load_state(&self.state_path) {
            Ok(Some(state)) => Ok((state, true)),
            Ok(None) => Ok((PersistedAutomationState::default(), false)),
            Err(_) => {
                quarantine_invalid(&self.state_path)?;
                let state = PersistedAutomationState::default();
                persist_state(&self.state_path, &state)?;
                Ok((state, true))
            }
        }
    }
}

struct ExecutionResult {
    outcome: String,
    summary: String,
}

fn status_from_state(state: &PersistedAutomationState, configured: bool) -> AutomationStatus {
    AutomationStatus {
        configured,
        jobs: state.jobs.clone(),
        history: state.history.iter().rev().cloned().collect(),
    }
}

fn validate_interval(interval_minutes: u64) -> Result<()> {
    if (MIN_INTERVAL_MINUTES..=MAX_INTERVAL_MINUTES).contains(&interval_minutes) {
        Ok(())
    } else {
        Err(AgentError::InvalidState(
            "automation interval must be between 15 minutes and 7 days",
        ))
    }
}

fn interval_ms(interval_minutes: u64) -> u64 {
    interval_minutes.saturating_mul(60_000)
}

fn validate_state(state: &PersistedAutomationState) -> Result<()> {
    if state.version != AUTOMATION_STATE_VERSION
        || state.next_event_id == 0
        || state.jobs.len() != AutomationJobKind::ALL.len()
        || state.history.len() > MAX_AUTOMATION_EVENTS
    {
        return Err(AgentError::InvalidState(
            "automation state version or bounds are invalid",
        ));
    }

    let kinds = state.jobs.iter().map(|job| job.kind).collect::<BTreeSet<_>>();
    if kinds.len() != AutomationJobKind::ALL.len()
        || AutomationJobKind::ALL
            .iter()
            .any(|kind| !kinds.contains(kind))
    {
        return Err(AgentError::InvalidState(
            "automation state contains unknown or duplicate jobs",
        ));
    }

    for job in &state.jobs {
        validate_interval(job.interval_minutes)?;
        if job.consecutive_failures > MAX_CONSECUTIVE_RETRIES.saturating_add(1) {
            return Err(AgentError::InvalidState(
                "automation retry state is invalid",
            ));
        }
    }

    let mut previous_id = 0_u64;
    for event in &state.history {
        if event.id <= previous_id
            || event.id >= state.next_event_id
            || event.attempt == 0
            || event.summary.is_empty()
            || event.summary.chars().count() > 512
            || event
                .summary
                .chars()
                .any(|value| matches!(value, '\r' | '\n' | '\0'))
            || !matches!(
                event.outcome.as_str(),
                "success"
                    | "attention"
                    | "action_required"
                    | "failed"
                    | "missed_recovered"
            )
        {
            return Err(AgentError::InvalidState(
                "automation event history is invalid",
            ));
        }
        previous_id = event.id;
    }
    Ok(())
}

fn load_state(path: &Path) -> Result<Option<PersistedAutomationState>> {
    recover_backup(path)?;
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(AgentError::Io("automation state could not be read")),
    };
    if bytes.len() > MAX_AUTOMATION_STATE_BYTES {
        return Err(AgentError::InvalidState(
            "automation state exceeded its safe size limit",
        ));
    }
    let state: PersistedAutomationState = serde_json::from_slice(&bytes)
        .map_err(|_| AgentError::InvalidState("automation state is malformed"))?;
    validate_state(&state)?;
    Ok(Some(state))
}

fn persist_state(path: &Path, state: &PersistedAutomationState) -> Result<()> {
    validate_state(state)?;
    let parent = path
        .parent()
        .ok_or(AgentError::Io("automation state parent is unavailable"))?;
    fs::create_dir_all(parent)
        .map_err(|_| AgentError::Io("automation state directory could not be created"))?;
    let encoded = serde_json::to_vec_pretty(state)
        .map_err(|_| AgentError::InvalidState("automation state could not be serialized"))?;
    if encoded.len() > MAX_AUTOMATION_STATE_BYTES {
        return Err(AgentError::InvalidState(
            "automation state exceeded its safe size limit",
        ));
    }

    let temporary = append_suffix(path, ".tmp");
    let backup = append_suffix(path, ".bak");
    {
        let mut options = OpenOptions::new();
        options.create(true).truncate(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(&temporary)
            .map_err(|_| AgentError::Io("temporary automation state could not be created"))?;
        file.write_all(&encoded)
            .and_then(|()| file.sync_all())
            .map_err(|_| AgentError::Io("automation state could not be written"))?;
    }

    if backup.exists() {
        fs::remove_file(&backup)
            .map_err(|_| AgentError::Io("stale automation backup could not be removed"))?;
    }
    if path.exists() {
        fs::rename(path, &backup)
            .map_err(|_| AgentError::Io("automation state could not be staged"))?;
    }
    if fs::rename(&temporary, path).is_err() {
        if backup.exists() && !path.exists() {
            let _ = fs::rename(&backup, path);
        }
        return Err(AgentError::Io("automation state could not be finalized"));
    }
    if backup.exists() {
        fs::remove_file(backup)
            .map_err(|_| AgentError::Io("automation backup could not be cleared"))?;
    }
    Ok(())
}

fn recover_backup(path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    let backup = append_suffix(path, ".bak");
    if backup.exists() {
        fs::rename(&backup, path)
            .map_err(|_| AgentError::Io("automation backup could not be recovered"))?;
    }
    Ok(())
}

fn quarantine_invalid(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let invalid = append_suffix(path, ".invalid");
    if invalid.exists() {
        fs::remove_file(&invalid)
            .map_err(|_| AgentError::Io("automation quarantine could not be replaced"))?;
    }
    fs::rename(path, invalid)
        .map_err(|_| AgentError::Io("invalid automation state could not be quarantined"))
}

fn append_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(suffix);
    PathBuf::from(value)
}

fn acquire_lock(path: &Path) -> Result<AutomationLock> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|_| AgentError::Io("automation lock directory could not be created"))?;
    }
    let open = || OpenOptions::new().write(true).create_new(true).open(path);
    let file = match open() {
        Ok(file) => file,
        Err(_) => {
            let stale = fs::metadata(path)
                .ok()
                .and_then(|metadata| metadata.modified().ok())
                .and_then(|modified| modified.elapsed().ok())
                .is_some_and(|age| age >= LOCK_STALE_AFTER);
            if !stale {
                return Err(AgentError::Unavailable(
                    "scheduled automation state is busy",
                ));
            }
            let _ = fs::remove_file(path);
            open().map_err(|_| AgentError::Unavailable("scheduled automation state is busy"))?
        }
    };
    Ok(AutomationLock {
        path: path.to_path_buf(),
        _file: file,
    })
}

struct AutomationLock {
    path: PathBuf,
    _file: fs::File,
}

impl Drop for AutomationLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
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

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    fn runtime() -> (tempfile::TempDir, AgentAutomationRuntime) {
        let dir = tempdir().expect("tempdir");
        let integrity = AgentIntegrityRuntime::from_paths(
            dir.path().join("integrity-state.json"),
            dir.path().join("baseline.json"),
        );
        let runtime = AgentAutomationRuntime::from_paths(
            dir.path().join("automation.json"),
            dir.path().join("automation.lock"),
            integrity,
        );
        (dir, runtime)
    }

    #[test]
    fn defaults_are_opt_in_and_capability_scoped() {
        let (_dir, runtime) = runtime();
        let status = runtime.status().expect("status");
        assert!(!status.configured);
        assert_eq!(status.jobs.len(), 3);
        assert!(status.jobs.iter().all(|job| !job.enabled));
        assert!(AutomationJobKind::parse("shell").is_err());
        assert!(AutomationJobKind::parse("powershell").is_err());
        assert!(AutomationJobKind::parse("security_scan").is_ok());
    }

    #[test]
    fn intervals_are_bounded() {
        let (_dir, runtime) = runtime();
        assert!(
            runtime
                .configure_job(AutomationJobKind::SecurityScan, true, 1)
                .is_err()
        );
        assert!(
            runtime
                .configure_job(AutomationJobKind::SecurityScan, true, 15)
                .is_ok()
        );
        assert!(
            runtime
                .configure_job(AutomationJobKind::SecurityScan, true, 10_081)
                .is_err()
        );
    }

    #[test]
    fn backup_reminder_is_persistent_and_never_stores_a_secret() {
        let (dir, runtime) = runtime();
        runtime
            .configure_job(AutomationJobKind::BackupReminder, true, 60)
            .expect("configure");
        let status = runtime
            .run_now(AutomationJobKind::BackupReminder)
            .expect("run reminder");
        assert_eq!(status.history[0].outcome, "action_required");
        let bytes = fs::read(dir.path().join("automation.json")).expect("state");
        let encoded = String::from_utf8(bytes).expect("utf8");
        assert!(!encoded.to_ascii_lowercase().contains("\"password\":"));
        assert!(!encoded.to_ascii_lowercase().contains("\"credential\":"));
    }

    #[test]
    fn history_is_bounded_and_ids_remain_monotonic() {
        let mut state = PersistedAutomationState::default();
        for index in 0..(MAX_AUTOMATION_EVENTS + 20) {
            let event = AutomationEvent {
                id: state.next_event_id,
                timestamp_ms: index as u64,
                job: AutomationJobKind::BackupReminder,
                outcome: "action_required".to_owned(),
                summary: "Backup is due.".to_owned(),
                attempt: 1,
            };
            state.next_event_id += 1;
            state.history.push_back(event);
            while state.history.len() > MAX_AUTOMATION_EVENTS {
                state.history.pop_front();
            }
        }
        validate_state(&state).expect("valid");
        assert_eq!(state.history.len(), MAX_AUTOMATION_EVENTS);
        assert!(state.history.front().expect("first").id > 1);
    }

    #[test]
    fn interrupted_replacement_recovers_backup() {
        let (dir, runtime) = runtime();
        runtime
            .configure_job(AutomationJobKind::BackupReminder, true, 60)
            .expect("configure");
        let path = dir.path().join("automation.json");
        let backup = append_suffix(&path, ".bak");
        fs::rename(&path, &backup).expect("simulate interruption");
        let status = runtime.status().expect("recover");
        assert!(status.configured);
        assert!(path.exists());
        assert!(!backup.exists());
    }
}
