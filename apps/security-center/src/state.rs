use std::sync::{Mutex, MutexGuard};

use dragonforge_core::{Component, CoreResult, EventKind, Platform, Severity};
use serde::Serialize;

use crate::agent::{
    AgentClient, AgentStatus, UnavailableAgentClient, validate_future_agent_request,
};
use crate::events::{DashboardEvent, EventStore};
use crate::logging::SafeLogger;
use crate::model::{ComponentRegistry, ComponentStatus, HealthSummary};
use crate::orchestration;
use crate::settings::{SecurityCenterSettings, SettingsStore};

#[derive(Debug, Clone, Serialize)]
pub struct DashboardSnapshot {
    pub health: HealthSummary,
    pub components: Vec<ComponentStatus>,
    pub events: Vec<DashboardEvent>,
    pub agent: AgentStatus,
    pub platform: &'static str,
    pub settings: SecurityCenterSettings,
}

pub struct AppState {
    registry: ComponentRegistry,
    events: Mutex<EventStore>,
    settings: Mutex<SecurityCenterSettings>,
    settings_store: SettingsStore,
    logger: SafeLogger,
    agent: Box<dyn AgentClient>,
}

impl AppState {
    pub fn initialize() -> CoreResult<Self> {
        let settings_store = SettingsStore::discover()?;
        let (settings, settings_warning) = match settings_store.load() {
            Ok(settings) => (settings, false),
            Err(_) => (SecurityCenterSettings::default(), true),
        };
        let logger = SafeLogger::discover(settings.include_diagnostic_identifiers)?;
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
        if !validate_future_agent_request(1) {
            events.push(
                Component::SecurityCenter,
                EventKind::Ipc,
                Severity::Critical,
                "security-center.agent-policy-invalid",
                "Future Agent IPC policy validation failed",
            );
        }

        let state = Self {
            registry: ComponentRegistry::phase3_default(),
            events: Mutex::new(events),
            settings: Mutex::new(settings),
            settings_store,
            logger,
            agent: Box::<UnavailableAgentClient>::default(),
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
            agent: Box::<UnavailableAgentClient>::default(),
        }
    }

    pub fn snapshot(&self) -> Result<DashboardSnapshot, String> {
        Ok(DashboardSnapshot {
            health: self.registry.health_summary(),
            components: self.registry.all().to_vec(),
            events: self.lock_events()?.recent(12),
            agent: self.agent.status(),
            platform: Platform::current().as_str(),
            settings: self.lock_settings()?.clone(),
        })
    }

    pub fn recent_events(&self, limit: usize) -> Result<Vec<DashboardEvent>, String> {
        Ok(self.lock_events()?.recent(limit.min(2_000)))
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
        let encoded = serde_json::to_string(&snapshot).expect("serialize snapshot");
        assert!(encoded.contains("security-center"));
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
