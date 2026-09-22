use std::sync::{Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use dragonforge_core::{Component, ComponentLogger, CoreResult, EventKind, Platform, Severity};
use serde::Serialize;

use crate::agent::{AgentClient, AgentStatus};
use crate::events::{DashboardEvent, EventStore};
use crate::logging::SafeLogger;
use crate::model::{ComponentRegistry, ComponentStatus, HealthSummary};
use crate::orchestration;
use crate::settings::{SecurityCenterSettings, SettingsStore};

#[derive(Debug, Clone, Serialize)]
pub struct ComponentFailureStatus {
    pub component: &'static str,
    pub has_failure: bool,
    pub last_failure: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardSnapshot {
    pub health: HealthSummary,
    pub components: Vec<ComponentStatus>,
    pub events: Vec<DashboardEvent>,
    pub agent: AgentStatus,
    pub platform: &'static str,
    pub settings: SecurityCenterSettings,
    pub last_failure: Option<String>,
    pub component_failures: Vec<ComponentFailureStatus>,
}

pub struct AppState {
    registry: ComponentRegistry,
    events: Mutex<EventStore>,
    settings: Mutex<SecurityCenterSettings>,
    settings_store: SettingsStore,
    logger: SafeLogger,
    agent: AgentClient,
    agent_auto_start_suppressed: Mutex<bool>,
}

impl AppState {
    pub fn initialize() -> CoreResult<Self> {
        let settings_store = SettingsStore::discover()?;
        let (settings, settings_warning) = match settings_store.load() {
            Ok(settings) => (settings, false),
            Err(_) => (SecurityCenterSettings::default(), true),
        };
        let logger = SafeLogger::discover(settings.include_diagnostic_identifiers)?;
        logger.install_panic_hook();
        let mut events = EventStore::new(settings.retain_event_count);
        events.push(
            Component::SecurityCenter,
            EventKind::Lifecycle,
            Severity::Info,
            "security-center.started",
            "Security Center started",
        );
        if settings_warning {
            events.push(
                Component::SecurityCenter,
                EventKind::Configuration,
                Severity::Warning,
                "security-center.settings-fallback",
                "Invalid local settings were ignored; safe defaults are active",
            );
        }
        let state = Self {
            registry: ComponentRegistry::phase3_default(),
            events: Mutex::new(events),
            settings: Mutex::new(settings),
            settings_store,
            logger,
            agent: AgentClient::discover(),
            agent_auto_start_suppressed: Mutex::new(false),
        };
        let _ = state
            .logger
            .write("info", "security-center.started", "Security Center started");
        Ok(state)
    }

    #[cfg(test)]
    pub fn for_test(settings_store: SettingsStore, logger: SafeLogger) -> Self {
        let settings = SecurityCenterSettings::default();
        Self {
            registry: ComponentRegistry::phase3_default(),
            events: Mutex::new(EventStore::new(settings.retain_event_count)),
            settings: Mutex::new(settings),
            settings_store,
            logger,
            agent: AgentClient::unavailable(),
            agent_auto_start_suppressed: Mutex::new(false),
        }
    }

    pub fn snapshot(&self) -> Result<DashboardSnapshot, String> {
        let agent = self.agent.status();
        let mut components = self.registry.all().to_vec();
        let mut health = self.registry.health_summary();

        if agent.available {
            if let Some(component) = components.iter_mut().find(|item| item.id == "agent") {
                component.state = crate::model::ComponentState::Active;
                component.state_label = "Active";
                component.detail = "Authenticated Phase 11 background agent is running.";
            }
            health.active += 1;
            health.integrated = health.integrated.saturating_sub(1);
        } else {
            health.attention += 1;
            health.state = "attention";
            health.label = "Agent attention required";
        }

        Ok(DashboardSnapshot {
            health,
            components,
            events: self.lock_events()?.recent(12),
            agent,
            platform: Platform::current().as_str(),
            settings: self.lock_settings()?.clone(),
            last_failure: self.logger.read_last_failure(),
            component_failures: component_failure_statuses(),
        })
    }

    pub fn recent_events(&self, limit: usize) -> Result<Vec<DashboardEvent>, String> {
        Ok(self.lock_events()?.recent(limit.min(2_000)))
    }

    pub fn diagnostic_report(&self) -> Result<String, String> {
        crate::diagnostics::render(&self.snapshot()?)
    }

    pub fn create_support_bundle(&self) -> Result<String, String> {
        let snapshot = self.snapshot()?;
        let path = crate::diagnostics::write_support_bundle(&snapshot, &self.logger)?;
        self.lock_events()?.push(
            Component::SecurityCenter,
            EventKind::Lifecycle,
            Severity::Info,
            "security-center.support-bundle-created",
            "Redaction-safe support bundle created",
        );
        let _ = self.logger.write(
            "info",
            "security-center.support-bundle-created",
            "Redaction-safe support bundle created",
        );
        Ok(path)
    }

    pub fn refresh_health(&self) -> Result<DashboardSnapshot, String> {
        self.lock_events()?.push(
            Component::SecurityCenter,
            EventKind::Health,
            Severity::Info,
            "security-center.health-refreshed",
            "Suite component health refreshed",
        );
        let _ = self.logger.write(
            "info",
            "security-center.health-refreshed",
            "Suite component health refreshed",
        );
        self.snapshot()
    }

    pub fn clear_events(&self) -> Result<(), String> {
        self.lock_events()?.clear();
        let _ = self.logger.write(
            "info",
            "security-center.events-cleared",
            "In-memory activity history cleared",
        );
        Ok(())
    }

    pub fn settings(&self) -> Result<SecurityCenterSettings, String> {
        Ok(self.lock_settings()?.clone())
    }

    pub fn update_settings(
        &self,
        updated: SecurityCenterSettings,
    ) -> Result<SecurityCenterSettings, String> {
        updated.validate().map_err(|error| error.to_string())?;
        self.settings_store
            .save(&updated)
            .map_err(|error| error.to_string())?;
        *self.lock_settings()? = updated.clone();

        self.lock_events()?.set_capacity(updated.retain_event_count);
        self.lock_events()?.push(
            Component::SecurityCenter,
            EventKind::Configuration,
            Severity::Info,
            "security-center.settings-updated",
            "Security Center settings updated",
        );
        let _ = self.logger.write(
            "info",
            "security-center.settings-updated",
            "Security Center settings updated",
        );
        Ok(updated)
    }

    pub fn agent_status(&self) -> AgentStatus {
        self.agent.status()
    }

    pub fn ensure_agent_running(&self) -> AgentStatus {
        if self
            .agent_auto_start_suppressed
            .lock()
            .map(|suppressed| *suppressed)
            .unwrap_or(false)
        {
            return self.agent.status();
        }

        let current = self.agent.status();
        if current.available {
            return current;
        }

        let _ = self.record_agent_event(
            "security-center.agent-autostart",
            "DragonForge Agent automatic start requested",
        );

        if orchestration::launch_agent().is_err() {
            return self.agent.status();
        }
        if let Some(status) = self.wait_for_agent(true, Duration::from_secs(2)) {
            return status;
        }

        // A crashed process can leave its create_new lock file behind briefly.
        // Phase 11 deliberately waits five seconds before reclaiming that lock.
        thread::sleep(Duration::from_millis(3_500));
        if orchestration::launch_agent().is_ok() {
            if let Some(status) = self.wait_for_agent(true, Duration::from_secs(2)) {
                let _ = self.record_agent_event(
                    "security-center.agent-recovered",
                    "DragonForge Agent recovered after stale/crashed runtime state",
                );
                return status;
            }
        }

        self.agent.status()
    }

    pub fn stop_agent(&self) -> Result<AgentStatus, String> {
        *self
            .agent_auto_start_suppressed
            .lock()
            .map_err(|_| "Agent lifecycle state is unavailable".to_owned())? = true;
        let current = self.agent.status();
        if !current.available {
            return Ok(current);
        }
        self.agent.shutdown()?;
        let stopped = self
            .wait_for_agent(false, Duration::from_secs(2))
            .unwrap_or_else(|| self.agent.status());
        self.record_agent_event(
            "security-center.agent-stopped",
            "DragonForge Agent graceful shutdown completed",
        )?;
        Ok(stopped)
    }

    pub fn restart_agent(&self) -> Result<AgentStatus, String> {
        *self
            .agent_auto_start_suppressed
            .lock()
            .map_err(|_| "Agent lifecycle state is unavailable".to_owned())? = false;
        if self.agent.status().available {
            self.agent.shutdown()?;
            let _ = self.wait_for_agent(false, Duration::from_secs(2));
        }
        let status = self.ensure_agent_running();
        if !status.available {
            return Err("DragonForge Agent did not reconnect after restart.".to_owned());
        }
        self.record_agent_event(
            "security-center.agent-restarted",
            "DragonForge Agent restarted and reconnected",
        )?;
        Ok(status)
    }

    fn wait_for_agent(&self, available: bool, timeout: Duration) -> Option<AgentStatus> {
        let deadline = Instant::now() + timeout;
        loop {
            let status = self.agent.status();
            if status.available == available {
                return Some(status);
            }
            if Instant::now() >= deadline {
                return None;
            }
            thread::sleep(Duration::from_millis(100));
        }
    }

    fn record_agent_event(&self, code: &'static str, message: &'static str) -> Result<(), String> {
        self.lock_events()?.push(
            Component::Agent,
            EventKind::Lifecycle,
            Severity::Info,
            code,
            message,
        );
        let _ = self.logger.write("info", code, message);
        Ok(())
    }

    pub fn launch_agent(&self) -> Result<(), String> {
        *self
            .agent_auto_start_suppressed
            .lock()
            .map_err(|_| "Agent lifecycle state is unavailable".to_owned())? = false;
        orchestration::launch_agent().map_err(|error| error.to_string())?;
        self.lock_events()?.push(
            Component::SecurityCenter,
            EventKind::Lifecycle,
            Severity::Info,
            "security-center.agent-launched",
            "DragonForge Agent start requested",
        );
        let _ = self.logger.write(
            "info",
            "security-center.agent-launched",
            "DragonForge Agent start requested",
        );
        Ok(())
    }

    pub fn launch_authenticator(&self) -> Result<(), String> {
        orchestration::launch_authenticator().map_err(|error| error.to_string())?;
        self.lock_events()?.push(
            Component::SecurityCenter,
            EventKind::Lifecycle,
            Severity::Info,
            "security-center.authenticator-launched",
            "Authenticator launch requested",
        );
        let _ = self.logger.write(
            "info",
            "security-center.authenticator-launched",
            "Authenticator launch requested",
        );
        Ok(())
    }

    pub fn launch_security_scanner(&self) -> Result<(), String> {
        orchestration::launch_security_scanner().map_err(|error| error.to_string())?;
        self.lock_events()?.push(
            Component::SecurityCenter,
            EventKind::Lifecycle,
            Severity::Info,
            "security-center.security-scanner-launched",
            "Security Scanner launch requested",
        );
        let _ = self.logger.write(
            "info",
            "security-center.security-scanner-launched",
            "Security Scanner launch requested",
        );
        Ok(())
    }

    pub fn launch_integrity_monitor(&self) -> Result<(), String> {
        orchestration::launch_integrity_monitor().map_err(|error| error.to_string())?;
        self.lock_events()?.push(
            Component::SecurityCenter,
            EventKind::Lifecycle,
            Severity::Info,
            "security-center.integrity-monitor-launched",
            "Integrity Monitor launch requested",
        );
        let _ = self.logger.write(
            "info",
            "security-center.integrity-monitor-launched",
            "Integrity Monitor launch requested",
        );
        Ok(())
    }

    pub fn launch_network_guard(&self) -> Result<(), String> {
        orchestration::launch_network_guard().map_err(|error| error.to_string())?;
        self.lock_events()?.push(
            Component::SecurityCenter,
            EventKind::Lifecycle,
            Severity::Info,
            "security-center.network-guard-launched",
            "Network Guard launch requested",
        );
        let _ = self.logger.write(
            "info",
            "security-center.network-guard-launched",
            "Network Guard launch requested",
        );
        Ok(())
    }

    pub fn launch_secure_share(&self) -> Result<(), String> {
        orchestration::launch_secure_share().map_err(|error| error.to_string())?;
        self.lock_events()?.push(
            Component::SecurityCenter,
            EventKind::Lifecycle,
            Severity::Info,
            "security-center.secure-share-launched",
            "Secure Share launch requested",
        );
        let _ = self.logger.write(
            "info",
            "security-center.secure-share-launched",
            "Secure Share launch requested",
        );
        Ok(())
    }

    pub fn launch_backup_recovery(&self) -> Result<(), String> {
        orchestration::launch_backup_recovery().map_err(|error| error.to_string())?;
        self.lock_events()?.push(
            Component::SecurityCenter,
            EventKind::Lifecycle,
            Severity::Info,
            "security-center.backup-recovery-launched",
            "Backup & Recovery launch requested",
        );
        let _ = self.logger.write(
            "info",
            "security-center.backup-recovery-launched",
            "Backup & Recovery launch requested",
        );
        Ok(())
    }

    pub fn launch_file_vault(&self) -> Result<(), String> {
        orchestration::launch_file_vault().map_err(|error| error.to_string())?;
        self.lock_events()?.push(
            Component::SecurityCenter,
            EventKind::Lifecycle,
            Severity::Info,
            "security-center.file-vault-launched",
            "File Vault launch requested",
        );
        let _ = self.logger.write(
            "info",
            "security-center.file-vault-launched",
            "File Vault launch requested",
        );
        Ok(())
    }

    pub fn launch_password_manager(&self) -> Result<(), String> {
        orchestration::launch_password_manager().map_err(|error| error.to_string())?;
        self.lock_events()?.push(
            Component::SecurityCenter,
            EventKind::Lifecycle,
            Severity::Info,
            "security-center.password-manager-launched",
            "Password Manager launch requested",
        );
        let _ = self.logger.write(
            "info",
            "security-center.password-manager-launched",
            "Password Manager launch requested",
        );
        Ok(())
    }

    fn lock_events(&self) -> Result<MutexGuard<'_, EventStore>, String> {
        self.events
            .lock()
            .map_err(|_| "Security Center event state is unavailable".to_owned())
    }

    fn lock_settings(&self) -> Result<MutexGuard<'_, SecurityCenterSettings>, String> {
        self.settings
            .lock()
            .map_err(|_| "Security Center settings state is unavailable".to_owned())
    }
}


fn component_failure_statuses() -> Vec<ComponentFailureStatus> {
    Component::ALL
        .into_iter()
        .map(|component| {
            let last_failure = ComponentLogger::discover(component, false)
                .ok()
                .and_then(|logger| logger.read_last_failure());
            ComponentFailureStatus {
                component: component.as_str(),
                has_failure: last_failure.is_some(),
                last_failure,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use dragonforge_core::LogPolicy;
    use tempfile::tempdir;

    use super::AppState;
    use crate::logging::SafeLogger;
    use crate::settings::{SecurityCenterSettings, SettingsStore};

    #[test]
    fn snapshot_exposes_registry_without_claiming_agent_connection() {
        let dir = tempdir().expect("temporary directory");
        let state = AppState::for_test(
            SettingsStore::from_dir(dir.path().join("config")),
            SafeLogger::from_path(dir.path().join("center.log"), LogPolicy::default()),
        );
        let snapshot = state.snapshot().expect("snapshot");
        assert_eq!(snapshot.components.len(), 10);
        assert!(!snapshot.agent.available);
        assert_eq!(snapshot.health.attention, 1);
        let encoded = serde_json::to_string(&snapshot).expect("serialize snapshot");
        assert!(encoded.contains("security-center"));
    }

    #[test]
    fn manual_stop_suppresses_same_session_auto_restart() {
        let dir = tempdir().expect("temporary directory");
        let state = AppState::for_test(
            SettingsStore::from_dir(dir.path().join("config")),
            SafeLogger::from_path(dir.path().join("center.log"), LogPolicy::default()),
        );
        let stopped = state.stop_agent().expect("stop unavailable agent");
        assert!(!stopped.available);
        let after = state.ensure_agent_running();
        assert!(!after.available);
    }

    #[test]
    fn updating_settings_persists_and_updates_state() {
        let dir = tempdir().expect("temporary directory");
        let config = dir.path().join("config");
        let state = AppState::for_test(
            SettingsStore::from_dir(&config),
            SafeLogger::from_path(dir.path().join("center.log"), LogPolicy::default()),
        );
        let updated = SecurityCenterSettings {
            retain_event_count: 600,
            ..SecurityCenterSettings::default()
        };
        state.update_settings(updated.clone()).expect("update");
        assert_eq!(state.settings().expect("settings"), updated);
        assert_eq!(
            SettingsStore::from_dir(config).load().expect("persisted"),
            updated
        );
    }
}
