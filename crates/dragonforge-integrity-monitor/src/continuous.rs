use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    ChangeKind, IntegrityError, IntegrityResult, SurfaceKind, baseline_summary, compare_to_baseline,
};

pub const CONTINUOUS_STATE_VERSION: u16 = 1;
pub const DEFAULT_INTERVAL_SECONDS: u64 = 300;
pub const MIN_INTERVAL_SECONDS: u64 = 60;
pub const MAX_INTERVAL_SECONDS: u64 = 86_400;
pub const MAX_SUPPRESSION_RULES: usize = 256;
pub const MAX_CONTINUOUS_EVENTS: usize = 500;
const MAX_STATE_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuppressionRule {
    pub surface: SurfaceKind,
    pub key_prefix: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuousMonitorPolicy {
    pub enabled: bool,
    pub interval_seconds: u64,
    #[serde(default)]
    pub suppressions: Vec<SuppressionRule>,
}

impl Default for ContinuousMonitorPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: DEFAULT_INTERVAL_SECONDS,
            suppressions: Vec::new(),
        }
    }
}

impl ContinuousMonitorPolicy {
    pub fn validate(&self) -> IntegrityResult<()> {
        if !(MIN_INTERVAL_SECONDS..=MAX_INTERVAL_SECONDS).contains(&self.interval_seconds) {
            return Err(IntegrityError::InvalidBaseline(
                "continuous integrity interval is outside the allowed range",
            ));
        }
        if self.suppressions.len() > MAX_SUPPRESSION_RULES {
            return Err(IntegrityError::LimitExceeded(
                "continuous integrity suppression rule limit exceeded",
            ));
        }
        for rule in &self.suppressions {
            let prefix = rule.key_prefix.trim();
            if prefix.is_empty()
                || prefix.len() > 256
                || prefix
                    .chars()
                    .any(|value| matches!(value, '\r' | '\n' | '\0'))
            {
                return Err(IntegrityError::InvalidBaseline(
                    "continuous integrity suppression rule is invalid",
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuousIntegrityEvent {
    pub id: u64,
    pub timestamp_ms: u64,
    pub surface: SurfaceKind,
    pub key: String,
    pub change: String,
    pub suppressed: bool,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuousMonitorStatus {
    pub configured: bool,
    pub enabled: bool,
    pub interval_seconds: u64,
    pub baseline_sealed: bool,
    pub last_check_ms: Option<u64>,
    pub next_check_ms: Option<u64>,
    pub retained_events: usize,
    pub unsuppressed_events: usize,
    pub suppression_rules: Vec<SuppressionRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuousCheckOutcome {
    pub ran: bool,
    pub new_events: usize,
    pub status: ContinuousMonitorStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContinuousState {
    version: u16,
    policy: ContinuousMonitorPolicy,
    baseline_sha256: String,
    last_check_ms: Option<u64>,
    next_check_ms: Option<u64>,
    next_event_id: u64,
    events: Vec<ContinuousIntegrityEvent>,
}

pub fn configure_continuous_monitoring(
    state_path: &Path,
    baseline_path: &Path,
    policy: ContinuousMonitorPolicy,
) -> IntegrityResult<ContinuousMonitorStatus> {
    policy.validate()?;
    let Some(_) = baseline_summary(baseline_path)? else {
        return Err(IntegrityError::BaselineMissing);
    };

    let prior = load_state_optional(state_path)?;
    let state = ContinuousState {
        version: CONTINUOUS_STATE_VERSION,
        policy,
        baseline_sha256: file_sha256(baseline_path)?,
        last_check_ms: prior.as_ref().and_then(|state| state.last_check_ms),
        next_check_ms: Some(now_ms()),
        next_event_id: prior.as_ref().map_or(1, |state| state.next_event_id.max(1)),
        events: prior.map_or_else(Vec::new, |state| state.events),
    };
    write_state(state_path, &state)?;
    status_for(&state, baseline_path)
}

pub fn reseal_continuous_baseline(state_path: &Path, baseline_path: &Path) -> IntegrityResult<()> {
    let Some(mut state) = load_state_optional(state_path)? else {
        return Ok(());
    };
    let Some(_) = baseline_summary(baseline_path)? else {
        return Err(IntegrityError::BaselineMissing);
    };
    state.baseline_sha256 = file_sha256(baseline_path)?;
    state.next_check_ms = Some(now_ms());
    write_state(state_path, &state)
}

pub fn continuous_status(
    state_path: &Path,
    baseline_path: &Path,
) -> IntegrityResult<ContinuousMonitorStatus> {
    match load_state_optional(state_path)? {
        Some(state) => status_for(&state, baseline_path),
        None => Ok(ContinuousMonitorStatus {
            configured: false,
            enabled: false,
            interval_seconds: DEFAULT_INTERVAL_SECONDS,
            baseline_sealed: false,
            last_check_ms: None,
            next_check_ms: None,
            retained_events: 0,
            unsuppressed_events: 0,
            suppression_rules: Vec::new(),
        }),
    }
}

pub fn continuous_events(
    state_path: &Path,
    after_id: u64,
    limit: usize,
) -> IntegrityResult<Vec<ContinuousIntegrityEvent>> {
    let Some(state) = load_state_optional(state_path)? else {
        return Ok(Vec::new());
    };
    Ok(state
        .events
        .into_iter()
        .filter(|event| event.id > after_id)
        .take(limit.min(MAX_CONTINUOUS_EVENTS))
        .collect())
}

pub fn run_continuous_check_if_due(
    state_path: &Path,
    baseline_path: &Path,
) -> IntegrityResult<ContinuousCheckOutcome> {
    run_continuous_check_at(state_path, baseline_path, now_ms())
}

fn run_continuous_check_at(
    state_path: &Path,
    baseline_path: &Path,
    now: u64,
) -> IntegrityResult<ContinuousCheckOutcome> {
    let Some(mut state) = load_state_optional(state_path)? else {
        return Ok(ContinuousCheckOutcome {
            ran: false,
            new_events: 0,
            status: continuous_status(state_path, baseline_path)?,
        });
    };
    state.policy.validate()?;
    if !state.policy.enabled || state.next_check_ms.is_some_and(|next| now < next) {
        return Ok(ContinuousCheckOutcome {
            ran: false,
            new_events: 0,
            status: status_for(&state, baseline_path)?,
        });
    }

    let mut new_events = 0usize;
    let sealed = file_sha256(baseline_path)
        .map(|hash| hash == state.baseline_sha256)
        .unwrap_or(false);
    if !sealed {
        push_event(
            &mut state,
            ContinuousIntegrityEvent {
                id: 0,
                timestamp_ms: now,
                surface: SurfaceKind::SystemConfiguration,
                key: "integrity-baseline".to_owned(),
                change: "baseline_changed".to_owned(),
                suppressed: false,
                summary: "Integrity baseline changed outside the approved baseline workflow."
                    .to_owned(),
            },
        );
        new_events = 1;
    } else {
        let report = compare_to_baseline(baseline_path)?;
        for change in report.changes {
            let suppressed = state.policy.suppressions.iter().any(|rule| {
                rule.surface == change.surface && change.key.starts_with(rule.key_prefix.trim())
            });
            let change_name = match change.kind {
                ChangeKind::Added => "added",
                ChangeKind::Removed => "removed",
                ChangeKind::Changed => "changed",
            };
            let summary = format!(
                "Integrity change detected: {} {} ({change_name}).",
                change.surface.as_str(),
                change.key
            );
            push_event(
                &mut state,
                ContinuousIntegrityEvent {
                    id: 0,
                    timestamp_ms: now,
                    surface: change.surface,
                    key: change.key,
                    change: change_name.to_owned(),
                    suppressed,
                    summary,
                },
            );
            new_events = new_events.saturating_add(1);
        }
    }

    state.last_check_ms = Some(now);
    state.next_check_ms =
        Some(now.saturating_add(state.policy.interval_seconds.saturating_mul(1_000)));
    write_state(state_path, &state)?;
    let status = status_for(&state, baseline_path)?;
    Ok(ContinuousCheckOutcome {
        ran: true,
        new_events,
        status,
    })
}

fn push_event(state: &mut ContinuousState, mut event: ContinuousIntegrityEvent) {
    event.id = state.next_event_id.max(1);
    state.next_event_id = event.id.saturating_add(1);
    state.events.push(event);
    if state.events.len() > MAX_CONTINUOUS_EVENTS {
        let remove = state.events.len() - MAX_CONTINUOUS_EVENTS;
        state.events.drain(0..remove);
    }
}

fn status_for(
    state: &ContinuousState,
    baseline_path: &Path,
) -> IntegrityResult<ContinuousMonitorStatus> {
    let baseline_sealed = file_sha256(baseline_path)
        .map(|hash| hash == state.baseline_sha256)
        .unwrap_or(false);
    Ok(ContinuousMonitorStatus {
        configured: true,
        enabled: state.policy.enabled,
        interval_seconds: state.policy.interval_seconds,
        baseline_sealed,
        last_check_ms: state.last_check_ms,
        next_check_ms: state.next_check_ms,
        retained_events: state.events.len(),
        unsuppressed_events: state
            .events
            .iter()
            .filter(|event| !event.suppressed)
            .count(),
        suppression_rules: state.policy.suppressions.clone(),
    })
}

fn load_state_optional(path: &Path) -> IntegrityResult<Option<ContinuousState>> {
    if !path.exists() {
        let backup = backup_path(path);
        if backup.exists() {
            fs::rename(&backup, path)?;
        } else {
            return Ok(None);
        }
    }
    let bytes = fs::read(path)?;
    if bytes.len() > MAX_STATE_BYTES {
        return Err(IntegrityError::LimitExceeded(
            "continuous integrity state exceeded the safe size limit",
        ));
    }
    let state: ContinuousState = serde_json::from_slice(&bytes)
        .map_err(|_| IntegrityError::InvalidBaseline("continuous integrity state is malformed"))?;
    if state.version != CONTINUOUS_STATE_VERSION
        || state.events.len() > MAX_CONTINUOUS_EVENTS
        || state.next_event_id == 0
    {
        return Err(IntegrityError::InvalidBaseline(
            "continuous integrity state is incompatible",
        ));
    }
    state.policy.validate()?;
    Ok(Some(state))
}

fn write_state(path: &Path, state: &ContinuousState) -> IntegrityResult<()> {
    let parent = path.parent().ok_or(IntegrityError::InvalidBaseline(
        "continuous integrity state parent is unavailable",
    ))?;
    fs::create_dir_all(parent)?;
    let bytes = serde_json::to_vec_pretty(state).map_err(|_| {
        IntegrityError::InvalidBaseline("continuous integrity state could not be encoded")
    })?;
    if bytes.len() > MAX_STATE_BYTES {
        return Err(IntegrityError::LimitExceeded(
            "continuous integrity state exceeded the safe size limit",
        ));
    }
    let temporary = temporary_path(path);
    let backup = backup_path(path);
    fs::write(&temporary, bytes)?;
    if backup.exists() {
        fs::remove_file(&backup)?;
    }
    if path.exists() {
        fs::rename(path, &backup)?;
    }
    if let Err(error) = fs::rename(&temporary, path) {
        if backup.exists() && !path.exists() {
            let _ = fs::rename(&backup, path);
        }
        return Err(error.into());
    }
    if backup.exists() {
        fs::remove_file(backup)?;
    }
    Ok(())
}

fn file_sha256(path: &Path) -> IntegrityResult<String> {
    let metadata = fs::metadata(path)?;
    if metadata.len() > 4 * 1024 * 1024 {
        return Err(IntegrityError::LimitExceeded(
            "integrity baseline exceeded the safe seal size limit",
        ));
    }
    let bytes = fs::read(path)?;
    let digest = Sha256::digest(&bytes);
    Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
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

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| {
            duration.as_millis().min(u128::from(u64::MAX)) as u64
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BASELINE_VERSION, BaselineEntry, IntegrityBaseline};

    fn write_baseline(path: &Path, fingerprint: &str) {
        let baseline = IntegrityBaseline {
            version: BASELINE_VERSION,
            created_at_ms: 1,
            entries: vec![BaselineEntry {
                surface: SurfaceKind::SystemConfiguration,
                key: "test.key".to_owned(),
                fingerprint: fingerprint.to_owned(),
            }],
            unavailable_surfaces: Vec::new(),
        };
        fs::write(path, serde_json::to_vec_pretty(&baseline).expect("encode")).expect("write");
    }

    #[test]
    fn policy_is_bounded() {
        let policy = ContinuousMonitorPolicy {
            interval_seconds: 1,
            ..ContinuousMonitorPolicy::default()
        };
        assert!(policy.validate().is_err());

        let policy = ContinuousMonitorPolicy {
            suppressions: (0..=MAX_SUPPRESSION_RULES)
                .map(|index| SuppressionRule {
                    surface: SurfaceKind::Startup,
                    key_prefix: format!("rule-{index}"),
                })
                .collect(),
            ..ContinuousMonitorPolicy::default()
        };
        assert!(policy.validate().is_err());
    }

    #[test]
    fn state_persists_policy_and_baseline_seal() {
        let dir = tempfile::tempdir().expect("tempdir");
        let baseline = dir.path().join("baseline.json");
        let state = dir.path().join("continuous.json");
        write_baseline(&baseline, &"00".repeat(32));
        let status =
            configure_continuous_monitoring(&state, &baseline, ContinuousMonitorPolicy::default())
                .expect("configure");
        assert!(status.configured);
        assert!(status.baseline_sealed);
        assert!(
            continuous_status(&state, &baseline)
                .expect("status")
                .baseline_sealed
        );
    }

    #[test]
    fn unexpected_baseline_change_creates_alert_and_blocks_comparison() {
        let dir = tempfile::tempdir().expect("tempdir");
        let baseline = dir.path().join("baseline.json");
        let state = dir.path().join("continuous.json");
        write_baseline(&baseline, &"00".repeat(32));
        configure_continuous_monitoring(&state, &baseline, ContinuousMonitorPolicy::default())
            .expect("configure");
        write_baseline(&baseline, &"11".repeat(32));

        let outcome = run_continuous_check_at(&state, &baseline, now_ms() + 1).expect("run");
        assert!(outcome.ran);
        assert_eq!(outcome.new_events, 1);
        assert!(!outcome.status.baseline_sealed);
        let events = continuous_events(&state, 0, 10).expect("events");
        assert_eq!(events[0].change, "baseline_changed");
        assert!(!events[0].suppressed);
    }

    #[test]
    fn interrupted_state_replacement_recovers_backup() {
        let dir = tempfile::tempdir().expect("tempdir");
        let baseline = dir.path().join("baseline.json");
        let state = dir.path().join("continuous.json");
        write_baseline(&baseline, &"00".repeat(32));
        configure_continuous_monitoring(&state, &baseline, ContinuousMonitorPolicy::default())
            .expect("configure");

        let backup = backup_path(&state);
        fs::rename(&state, &backup).expect("simulate interrupted replacement");
        assert!(!state.exists());
        let status = continuous_status(&state, &baseline).expect("recover");
        assert!(status.configured);
        assert!(state.exists());
        assert!(!backup.exists());
    }

    #[test]
    fn event_history_is_bounded() {
        let mut state = ContinuousState {
            version: CONTINUOUS_STATE_VERSION,
            policy: ContinuousMonitorPolicy::default(),
            baseline_sha256: "00".repeat(32),
            last_check_ms: None,
            next_check_ms: None,
            next_event_id: 1,
            events: Vec::new(),
        };
        for index in 0..(MAX_CONTINUOUS_EVENTS + 10) {
            push_event(
                &mut state,
                ContinuousIntegrityEvent {
                    id: 0,
                    timestamp_ms: index as u64,
                    surface: SurfaceKind::Startup,
                    key: format!("item-{index}"),
                    change: "changed".to_owned(),
                    suppressed: false,
                    summary: "safe".to_owned(),
                },
            );
        }
        assert_eq!(state.events.len(), MAX_CONTINUOUS_EVENTS);
        assert!(state.events[0].id > 1);
    }
}
