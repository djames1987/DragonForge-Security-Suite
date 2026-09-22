use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::state::DashboardSnapshot;

const PHASE: &str = "12.0";
const RELEASE_CHANNEL: &str = "alpha-external-test";

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
}

#[derive(Debug, Serialize)]
struct SettingsMetadata {
    retain_event_count: usize,
    diagnostic_identifiers_enabled: bool,
}

pub fn render(snapshot: &DashboardSnapshot) -> Result<String, String> {
    let include_identifiers = snapshot.settings.include_diagnostic_identifiers;
    let report = DiagnosticReport {
        schema_version: 1,
        generated_at_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
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
            })
            .collect(),
        settings: SettingsMetadata {
            retain_event_count: snapshot.settings.retain_event_count,
            diagnostic_identifiers_enabled: include_identifiers,
        },
    };

    serde_json::to_string_pretty(&report)
        .map_err(|_| "unable to create redaction-safe diagnostic report".to_owned())
}

#[cfg(test)]
mod tests {
    use dragonforge_core::LogPolicy;
    use tempfile::tempdir;

    use crate::{
        logging::SafeLogger,
        settings::SettingsStore,
        state::AppState,
    };

    #[test]
    fn report_contains_build_and_component_metadata_without_event_content() {
        let dir = tempdir().expect("temporary directory");
        let state = AppState::for_test(
            SettingsStore::from_dir(dir.path().join("config")),
            SafeLogger::from_path(dir.path().join("center.log"), LogPolicy::default()),
        );
        let report = super::render(&state.snapshot().expect("snapshot")).expect("report");

        assert!(report.contains("\"phase\": \"12.0\""));
        assert!(report.contains("\"package_version\": \"0.1.0\""));
        assert!(report.contains("\"security-center\""));
        assert!(!report.contains("Security Center started"));
        assert!(!report.contains("sync_token"));
        assert!(!report.contains("account_secret"));
        assert!(!report.contains("recovery_code"));
    }
}
