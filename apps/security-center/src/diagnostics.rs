use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use dragonforge_core::{Component, LogPolicy, SuitePaths, sanitize_diagnostic_text};
use serde::Serialize;
use serde_json::json;

use crate::logging::SafeLogger;
use crate::state::DashboardSnapshot;

const PHASE: &str = "25";
const RELEASE_CHANNEL: &str = "stable";

#[derive(Debug, Serialize)]
struct DiagnosticReport<'a> {
    schema_version: u16,
    generated_at_ms: u128,
    phase: &'static str,
    release_channel: &'static str,
    build: BuildMetadata<'a>,
    runtime: RuntimeMetadata<'a>,
    agent: AgentMetadata<'a>,
    components: Vec<ComponentMetadata<'a>>,
    settings: SettingsMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_failure: Option<&'a str>,
}

#[derive(Debug, Serialize)]
struct BuildMetadata<'a> {
    package_version: &'static str,
    git_commit: &'a str,
}

#[derive(Debug, Serialize)]
struct RuntimeMetadata<'a> {
    platform: &'a str,
    os: &'static str,
    architecture: &'static str,
    webview2_version: Option<String>,
}

#[derive(Debug, Serialize)]
struct AgentMetadata<'a> {
    available: bool,
    state: &'a str,
    transport: &'a str,
    capabilities: &'a [String],
    #[serde(skip_serializing_if = "Option::is_none")]
    pid: Option<u32>,
}

#[derive(Debug, Serialize)]
struct ComponentMetadata<'a> {
    id: &'a str,
    state: &'static str,
    version: &'static str,
}

#[derive(Debug, Serialize)]
struct SettingsMetadata {
    retain_event_count: usize,
    diagnostic_identifiers_enabled: bool,
}

pub fn render(snapshot: &DashboardSnapshot) -> Result<String, String> {
    let include_identifiers = snapshot.settings.include_diagnostic_identifiers;
    let report = DiagnosticReport {
        schema_version: 2,
        generated_at_ms: now_ms(),
        phase: PHASE,
        release_channel: RELEASE_CHANNEL,
        build: BuildMetadata {
            package_version: env!("CARGO_PKG_VERSION"),
            git_commit: option_env!("DRAGONFORGE_BUILD_COMMIT").unwrap_or("unknown"),
        },
        runtime: RuntimeMetadata {
            platform: snapshot.platform,
            os: std::env::consts::OS,
            architecture: std::env::consts::ARCH,
            webview2_version: detect_webview2_version(),
        },
        agent: AgentMetadata {
            available: snapshot.agent.available,
            state: &snapshot.agent.state,
            transport: &snapshot.agent.transport,
            capabilities: &snapshot.agent.capabilities,
            pid: if include_identifiers {
                snapshot.agent.pid
            } else {
                None
            },
        },
        components: snapshot
            .components
            .iter()
            .map(|component| ComponentMetadata {
                id: component.id,
                state: match component.state {
                    crate::model::ComponentState::Active => "active",
                    crate::model::ComponentState::Integrated => "integrated",
                },
                version: env!("CARGO_PKG_VERSION"),
            })
            .collect(),
        settings: SettingsMetadata {
            retain_event_count: snapshot.settings.retain_event_count,
            diagnostic_identifiers_enabled: include_identifiers,
        },
        last_failure: snapshot.last_failure.as_deref(),
    };

    serde_json::to_string_pretty(&report)
        .map_err(|_| "unable to create redaction-safe diagnostic report".to_owned())
}

pub fn write_support_bundle(
    snapshot: &DashboardSnapshot,
    logger: &SafeLogger,
) -> Result<String, String> {
    let diagnostic_text = render(snapshot)?;
    let diagnostics: serde_json::Value = serde_json::from_str(&diagnostic_text)
        .map_err(|_| "unable to assemble support bundle diagnostics".to_owned())?;

    let metadata = fs::metadata(logger.path()).ok();
    let log_tail = fs::read_to_string(logger.path())
        .ok()
        .map(|content| {
            content
                .lines()
                .rev()
                .take(100)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .map(|line| sanitize_diagnostic_text(LogPolicy::default(), line))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let component_logs = Component::ALL
        .into_iter()
        .map(|component| {
            let bytes = dragonforge_core::ComponentLogger::discover(component, false)
                .ok()
                .and_then(|item| fs::metadata(item.path()).ok())
                .map(|meta| meta.len())
                .unwrap_or(0);
            json!({
                "component": component.as_str(),
                "bytes": bytes
            })
        })
        .collect::<Vec<_>>();

    let bundle = json!({
        "schema_version": 1,
        "kind": "dragonforge-support-bundle",
        "generated_at_ms": now_ms(),
        "diagnostics": diagnostics,
        "component_logs": component_logs,
        "log_summary": {
            "component": "security-center",
            "bytes": metadata.map(|item| item.len()).unwrap_or(0),
            "tail": log_tail
        },
        "excluded": [
            "passwords",
            "tokens",
            "otp-seeds",
            "recovery-material",
            "private-keys",
            "vault-contents",
            "user-file-contents",
            "arbitrary-user-paths"
        ]
    });

    let paths = SuitePaths::discover().map_err(|error| error.to_string())?;
    let directory = paths
        .component_data_dir(Component::SecurityCenter)
        .join("support-bundles");
    fs::create_dir_all(&directory)
        .map_err(|_| "unable to create support bundle directory".to_owned())?;
    let path = directory.join(format!("dragonforge-support-{}.json", now_ms()));
    let encoded = serde_json::to_vec_pretty(&bundle)
        .map_err(|_| "unable to serialize support bundle".to_owned())?;
    fs::write(&path, encoded).map_err(|_| "unable to write support bundle".to_owned())?;
    Ok(path.display().to_string())
}

fn detect_webview2_version() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        const CLIENT: &str =
            r"SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8F9E-DA0A1B45327C}";
        for root in ["HKCU", "HKLM"] {
            let key = format!(r"{root}\{CLIENT}");
            let Ok(output) = Command::new("reg.exe")
                .args(["query", &key, "/v", "pv"])
                .output()
            else {
                continue;
            };
            if !output.status.success() {
                continue;
            }
            let text = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = text.lines().find(|line| line.contains("REG_SZ")) {
                if let Some(value) = line.split_whitespace().last() {
                    return Some(value.to_owned());
                }
            }
        }
    }
    None
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

    use dragonforge_core::LogPolicy;
    use tempfile::tempdir;

    use crate::{logging::SafeLogger, settings::SettingsStore, state::AppState};

    #[test]
    fn report_contains_versions_without_event_content() {
        let dir = tempdir().expect("temporary directory");
        let state = AppState::for_test(
            SettingsStore::from_dir(dir.path().join("config")),
            SafeLogger::from_path(dir.path().join("center.log"), LogPolicy::default()),
        );
        let report = super::render(&state.snapshot().expect("snapshot")).expect("report");

        assert!(report.contains("\"phase\": \"25\""));
        assert!(report.contains("\"release_channel\": \"stable\""));
        let version = env!("CARGO_PKG_VERSION");
        assert!(report.contains(&format!("\"package_version\": \"{version}\"")));
        assert!(report.contains(&format!("\"version\": \"{version}\"")));
        assert!(!report.contains("Security Center started"));
        assert!(!report.contains("sync_token"));
        assert!(!report.contains("account_secret"));
        assert!(!report.contains("recovery_code"));
    }

    #[test]
    fn support_bundle_redacts_sensitive_log_lines() {
        let dir = tempdir().expect("temporary directory");
        let logger = SafeLogger::from_path(dir.path().join("center.log"), LogPolicy::default());
        logger
            .write("error", "test.failure", "password=should-not-escape")
            .expect("write");
        let state = AppState::for_test(
            SettingsStore::from_dir(dir.path().join("config")),
            logger.clone(),
        );
        let report = super::render(&state.snapshot().expect("snapshot")).expect("report");
        assert!(!report.contains("should-not-escape"));
        let raw_log = fs::read_to_string(logger.path()).expect("read log");
        assert!(!raw_log.contains("should-not-escape"));
    }
}
