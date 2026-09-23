//! DragonForge privileged-service runtime foundations.
//!
//! Phase 17 implements the Windows service control plane only. No firewall,
//! quarantine, process-control, registry-remediation, or generic command runner
//! is enabled here.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use dragonforge_windows_boundary::{
    MAX_CLOCK_SKEW_SECONDS, PROTOCOL_MAJOR, PrivilegedRequest, PrivilegedResponse,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod firewall;
#[cfg(windows)]
pub mod windows;

pub const CONFIG_SCHEMA_VERSION: u16 = 1;
pub const DEFAULT_REQUESTS_PER_MINUTE: u32 = 120;
pub const DEFAULT_AUDIT_MAX_BYTES: u64 = 1_048_576;
pub const MAX_AUDIT_FILES: usize = 3;
pub const MAX_REPLAY_NONCES: usize = 4_096;
pub const MAX_QUOTA_IDENTITIES: usize = 1_024;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("privileged service configuration is invalid")]
    InvalidConfiguration,
    #[error("privileged service request was rejected")]
    RequestRejected,
    #[error("privileged service I/O failed")]
    Io,
    #[error("privileged service platform operation failed")]
    Platform,
}

pub type Result<T> = std::result::Result<T, ServiceError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ServiceConfig {
    pub schema_version: u16,
    pub expected_agent_path: PathBuf,
    pub expected_publisher_subject: String,
    #[serde(default = "default_requests_per_minute")]
    pub max_requests_per_minute: u32,
    #[serde(default = "default_audit_max_bytes")]
    pub audit_max_bytes: u64,
}

impl ServiceConfig {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != CONFIG_SCHEMA_VERSION {
            return Err(ServiceError::InvalidConfiguration);
        }
        let file_name = self
            .expected_agent_path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or(ServiceError::InvalidConfiguration)?;
        if !self.expected_agent_path.is_absolute()
            || !file_name.eq_ignore_ascii_case("dragonforge-agent.exe")
            || self.expected_agent_path.as_os_str().len() > 2048
        {
            return Err(ServiceError::InvalidConfiguration);
        }

        let subject = self.expected_publisher_subject.trim();
        if subject.is_empty() || subject.chars().count() > 512 {
            return Err(ServiceError::InvalidConfiguration);
        }
        if !(10..=10_000).contains(&self.max_requests_per_minute) {
            return Err(ServiceError::InvalidConfiguration);
        }
        if !(64 * 1024..=16 * 1024 * 1024).contains(&self.audit_max_bytes) {
            return Err(ServiceError::InvalidConfiguration);
        }
        Ok(())
    }
}

const fn default_requests_per_minute() -> u32 {
    DEFAULT_REQUESTS_PER_MINUTE
}

const fn default_audit_max_bytes() -> u64 {
    DEFAULT_AUDIT_MAX_BYTES
}

#[derive(Debug)]
pub struct AbuseGuard {
    request_limit: u32,
    request_window: Duration,
    quotas: HashMap<u32, RateWindow>,
    replay_order: VecDeque<String>,
    replay_seen: HashSet<String>,
}

#[derive(Debug)]
struct RateWindow {
    started: Instant,
    count: u32,
}

impl AbuseGuard {
    #[must_use]
    pub fn new(request_limit: u32) -> Self {
        Self {
            request_limit: request_limit.max(1),
            request_window: Duration::from_secs(60),
            quotas: HashMap::new(),
            replay_order: VecDeque::new(),
            replay_seen: HashSet::new(),
        }
    }

    pub fn authorize(&mut self, client_pid: u32, request: &PrivilegedRequest) -> Result<()> {
        validate_request_shape(request)?;

        if self.replay_seen.contains(&request.nonce_hex) {
            return Err(ServiceError::RequestRejected);
        }

        let now = Instant::now();
        self.quotas
            .retain(|_, window| now.duration_since(window.started) < self.request_window);

        if self.quotas.len() >= MAX_QUOTA_IDENTITIES && !self.quotas.contains_key(&client_pid) {
            return Err(ServiceError::RequestRejected);
        }

        let window = self.quotas.entry(client_pid).or_insert(RateWindow {
            started: now,
            count: 0,
        });
        if now.duration_since(window.started) >= self.request_window {
            window.started = now;
            window.count = 0;
        }
        if window.count >= self.request_limit {
            return Err(ServiceError::RequestRejected);
        }
        window.count += 1;

        self.replay_seen.insert(request.nonce_hex.clone());
        self.replay_order.push_back(request.nonce_hex.clone());
        while self.replay_order.len() > MAX_REPLAY_NONCES {
            if let Some(oldest) = self.replay_order.pop_front() {
                self.replay_seen.remove(&oldest);
            }
        }

        Ok(())
    }
}

pub fn validate_request_shape(request: &PrivilegedRequest) -> Result<()> {
    if request.protocol_major != PROTOCOL_MAJOR {
        return Err(ServiceError::RequestRejected);
    }
    if request.protocol_minor > dragonforge_windows_boundary::PROTOCOL_MINOR {
        return Err(ServiceError::RequestRejected);
    }
    if request.action.is_empty() || request.action.len() > 64 {
        return Err(ServiceError::RequestRejected);
    }
    match request.action.as_str() {
        "health" | "describe-policy" => {
            if request.firewall.is_some() {
                return Err(ServiceError::RequestRejected);
            }
        }
        "firewall-status" | "firewall-apply" | "firewall-remove" | "firewall-rollback" => {
            let firewall = request
                .firewall
                .as_ref()
                .ok_or(ServiceError::RequestRejected)?;
            firewall
                .validate()
                .map_err(|_| ServiceError::RequestRejected)?;
            match request.action.as_str() {
                "firewall-rollback" if firewall.rollback_token.is_none() => {
                    return Err(ServiceError::RequestRejected);
                }
                "firewall-status" | "firewall-apply" | "firewall-remove"
                    if firewall.rollback_token.is_some() =>
                {
                    return Err(ServiceError::RequestRejected);
                }
                _ => {}
            }
        }
        _ => {}
    }
    if request.nonce_hex.len() != 32
        || !request
            .nonce_hex
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(ServiceError::RequestRejected);
    }

    let now = now_ms();
    let maximum_skew_ms = MAX_CLOCK_SKEW_SECONDS.saturating_mul(1_000);
    if now.abs_diff(request.timestamp_ms) > maximum_skew_ms {
        return Err(ServiceError::RequestRejected);
    }
    Ok(())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AuditEvent<'a> {
    timestamp_ms: u64,
    event: &'a str,
    client_pid: Option<u32>,
    action: Option<&'a str>,
    outcome: &'a str,
}

#[derive(Debug, Clone)]
pub struct AuditLogger {
    path: PathBuf,
    max_bytes: u64,
}

impl AuditLogger {
    #[must_use]
    pub fn new(path: impl Into<PathBuf>, max_bytes: u64) -> Self {
        Self {
            path: path.into(),
            max_bytes,
        }
    }

    pub fn lifecycle(&self, event: &str, outcome: &str) -> Result<()> {
        self.write_event(AuditEvent {
            timestamp_ms: now_ms(),
            event,
            client_pid: None,
            action: None,
            outcome,
        })
    }

    pub fn request(&self, client_pid: u32, action: &str, outcome: &str) -> Result<()> {
        self.write_event(AuditEvent {
            timestamp_ms: now_ms(),
            event: "request",
            client_pid: Some(client_pid),
            action: Some(action),
            outcome,
        })
    }

    fn write_event(&self, event: AuditEvent<'_>) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|_| ServiceError::Io)?;
        }
        self.rotate_if_needed()?;

        let mut encoded = serde_json::to_vec(&event).map_err(|_| ServiceError::Io)?;
        if encoded.len() > 2_048 {
            return Err(ServiceError::Io);
        }
        encoded.push(b'\n');

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|_| ServiceError::Io)?;
        file.write_all(&encoded).map_err(|_| ServiceError::Io)?;
        file.flush().map_err(|_| ServiceError::Io)
    }

    fn rotate_if_needed(&self) -> Result<()> {
        let size = fs::metadata(&self.path)
            .map(|metadata| metadata.len())
            .unwrap_or(0);
        if size < self.max_bytes {
            return Ok(());
        }

        for index in (1..MAX_AUDIT_FILES).rev() {
            let from = rotated_path(&self.path, index);
            let to = rotated_path(&self.path, index + 1);
            if from.exists() {
                let _ = fs::remove_file(&to);
                fs::rename(&from, &to).map_err(|_| ServiceError::Io)?;
            }
        }

        let first = rotated_path(&self.path, 1);
        let _ = fs::remove_file(&first);
        if self.path.exists() {
            fs::rename(&self.path, first).map_err(|_| ServiceError::Io)?;
        }
        Ok(())
    }
}

fn rotated_path(path: &Path, index: usize) -> PathBuf {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("audit.jsonl");
    path.with_file_name(format!("{name}.{index}"))
}

#[must_use]
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            duration.as_millis().min(u128::from(u64::MAX)) as u64
        })
}

pub fn load_config(path: &Path) -> Result<ServiceConfig> {
    let bytes = fs::read(path).map_err(|_| ServiceError::InvalidConfiguration)?;
    if bytes.len() > 16 * 1024 {
        return Err(ServiceError::InvalidConfiguration);
    }
    let config: ServiceConfig =
        serde_json::from_slice(&bytes).map_err(|_| ServiceError::InvalidConfiguration)?;
    config.validate()?;
    Ok(config)
}

#[must_use]
pub fn rejected_response(request_id: u64, code: &str) -> PrivilegedResponse {
    PrivilegedResponse::error(request_id, code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dragonforge_windows_boundary::{PROTOCOL_MAJOR, PROTOCOL_MINOR};
    use tempfile::tempdir;

    fn request(nonce: &str) -> PrivilegedRequest {
        PrivilegedRequest {
            protocol_major: PROTOCOL_MAJOR,
            protocol_minor: PROTOCOL_MINOR,
            request_id: 7,
            action: "health".to_owned(),
            timestamp_ms: now_ms(),
            nonce_hex: nonce.to_owned(),
            firewall: None,
        }
    }

    #[test]
    fn config_is_bounded_and_requires_publisher_pin() {
        let valid = ServiceConfig {
            schema_version: CONFIG_SCHEMA_VERSION,
            expected_agent_path: PathBuf::from(
                r"C:\Program Files\DragonForge\dragonforge-agent.exe",
            ),
            expected_publisher_subject: "CN=DragonForge".to_owned(),
            max_requests_per_minute: 120,
            audit_max_bytes: DEFAULT_AUDIT_MAX_BYTES,
        };
        assert!(valid.validate().is_ok());

        let mut invalid = valid.clone();
        invalid.expected_publisher_subject.clear();
        assert!(invalid.validate().is_err());

        let mut invalid = valid;
        invalid.max_requests_per_minute = 0;
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn replay_and_rate_abuse_fail_closed() {
        let mut guard = AbuseGuard::new(2);
        let first = request("00112233445566778899aabbccddeeff");
        assert!(guard.authorize(100, &first).is_ok());
        assert!(guard.authorize(100, &first).is_err());

        let second = request("10112233445566778899aabbccddeeff");
        assert!(guard.authorize(100, &second).is_ok());
        let third = request("20112233445566778899aabbccddeeff");
        assert!(guard.authorize(100, &third).is_err());
    }

    #[test]
    fn malformed_and_stale_requests_are_rejected() {
        let mut malformed = request("bad");
        assert!(validate_request_shape(&malformed).is_err());

        malformed = request("30112233445566778899aabbccddeeff");
        malformed.protocol_major = PROTOCOL_MAJOR + 1;
        assert!(validate_request_shape(&malformed).is_err());

        let mut stale = request("40112233445566778899aabbccddeeff");
        stale.timestamp_ms = 1;
        assert!(validate_request_shape(&stale).is_err());
    }

    #[test]
    fn audit_is_jsonl_and_rotates_at_bound() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("audit.jsonl");
        let logger = AuditLogger::new(&path, 1);
        logger.lifecycle("start", "ok").expect("first audit");
        logger.lifecycle("stop", "ok").expect("second audit");
        assert!(path.exists());
        assert!(rotated_path(&path, 1).exists());
    }
}
