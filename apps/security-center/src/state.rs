use std::sync::{Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use dragonforge_agent::{AgentAutomationRuntime, AutomationJobKind, AutomationStatus};
use dragonforge_core::{
    Component, ComponentLogger, CoreError, CoreResult, ErrorCode, EventKind, Platform, Severity,
    SuitePaths,
};
use dragonforge_integrity_monitor::continuous_events;
use serde::Serialize;

use crate::agent::{AgentClient, AgentStatus};
use crate::events::{DashboardEvent, EventStore};
use crate::health_history::{HealthHistoryEntry, HealthHistoryStore};
use crate::logging::SafeLogger;
use crate::model::{ComponentRegistry, ComponentStatus, HealthSummary};
use crate::orchestration;
use crate::settings::{SecurityCenterSettings, SettingsStore};
use crate::update::{PreparedUpdateStatus, UpdateManager, UpdateStatus};

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
    pub notifications: Vec<DashboardEvent>,
    pub health_history: Vec<HealthHistoryEntry>,
}

pub struct AppState {
    registry: ComponentRegistry,
    events: Mutex<EventStore>,
    health_history: Mutex<HealthHistoryStore>,
    settings: Mutex<SecurityCenterSettings>,
    settings_store: SettingsStore,
    logger: SafeLogger,
    agent: AgentClient,
    agent_auto_start_suppressed: Mutex<bool>,
    updater: UpdateManager,
    automation: Option<AgentAutomationRuntime>,
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
        let mut events = EventStore::discover(settings.retain_event_count)?;
        events.push(
            Component::SecurityCenter,
            EventKind::Lifecycle,
            Severity::Info,
            "security-center.started",
            "Security Center started",
        )?;
        if settings_warning {
            events.push(
                Component::SecurityCenter,
                EventKind::Configuration,
                Severity::Warning,
                "security-center.settings-fallback",
                "Invalid local settings were ignored; safe defaults are active",
            )?;
        }
        let updater = UpdateManager::new().map_err(|_| {
            CoreError::new_safe(
                ErrorCode::Internal,
                "unable to initialize Security Center update subsystem",
            )
        })?;
        let health_history =
            HealthHistoryStore::discover(settings.suite_policy.health_history_limit)?;
        let state = Self {
            registry: ComponentRegistry::phase3_default(),
            events: Mutex::new(events),
            health_history: Mutex::new(health_history),
            settings: Mutex::new(settings),
            settings_store,
            logger,
            agent: AgentClient::discover(),
            agent_auto_start_suppressed: Mutex::new(false),
            updater,
            automation: AgentAutomationRuntime::discover().ok(),
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
            health_history: Mutex::new(HealthHistoryStore::new(
                settings.suite_policy.health_history_limit,
            )),
            settings: Mutex::new(settings),
            settings_store,
            logger,
            agent: AgentClient::unavailable(),
            agent_auto_start_suppressed: Mutex::new(false),
            updater: UpdateManager::new().expect("test update manager"),
            automation: None,
        }
    }

    pub fn snapshot(&self) -> Result<DashboardSnapshot, String> {
        let _ = self.sync_integrity_events();
        let _ = self.sync_automation_events();
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

        let _ = self
            .lock_health_history()?
            .record(&health, &components)
            .map_err(|error| error.to_string())?;
        let notification_threshold = self
            .lock_settings()?
            .suite_policy
            .notification_min_severity
            .clone();
        let notifications = self
            .lock_events()?
            .notifications(&notification_threshold, 50);
        let health_history = self.lock_health_history()?.recent(25);

        Ok(DashboardSnapshot {
            health,
            components,
            events: self.lock_events()?.recent(12),
            agent,
            platform: Platform::current().as_str(),
            settings: self.lock_settings()?.clone(),
            last_failure: self.logger.read_last_failure(),
            component_failures: component_failure_statuses(),
            notifications,
            health_history,
        })
    }

    pub fn recent_events(&self, limit: usize) -> Result<Vec<DashboardEvent>, String> {
        Ok(self.lock_events()?.recent(limit.min(2_000)))
    }

    pub fn notifications(&self, limit: usize) -> Result<Vec<DashboardEvent>, String> {
        let threshold = self
            .lock_settings()?
            .suite_policy
            .notification_min_severity
            .clone();
        Ok(self
            .lock_events()?
            .notifications(&threshold, limit.min(2_000)))
    }

    pub fn acknowledge_event(&self, event_id: u64) -> Result<bool, String> {
        self.lock_events()?
            .acknowledge(event_id)
            .map_err(|error| error.to_string())
    }

    pub fn acknowledge_all_notifications(&self) -> Result<usize, String> {
        let threshold = self
            .lock_settings()?
            .suite_policy
            .notification_min_severity
            .clone();
        self.lock_events()?
            .acknowledge_all(&threshold)
            .map_err(|error| error.to_string())
    }

    pub fn health_history(&self, limit: usize) -> Result<Vec<HealthHistoryEntry>, String> {
        Ok(self.lock_health_history()?.recent(limit.min(500)))
    }

    pub fn diagnostic_report(&self) -> Result<String, String> {
        crate::diagnostics::render(&self.snapshot()?)
    }

    pub fn create_support_bundle(&self) -> Result<String, String> {
        let snapshot = self.snapshot()?;
        let path = crate::diagnostics::write_support_bundle(&snapshot, &self.logger)?;
        self.record_event(
            Component::SecurityCenter,
            EventKind::Lifecycle,
            Severity::Info,
            "security-center.support-bundle-created",
            "Redaction-safe support bundle created",
        )?;
        let _ = self.logger.write(
            "info",
            "security-center.support-bundle-created",
            "Redaction-safe support bundle created",
        );
        Ok(path)
    }

    pub fn refresh_health(&self) -> Result<DashboardSnapshot, String> {
        self.record_event(
            Component::SecurityCenter,
            EventKind::Health,
            Severity::Info,
            "security-center.health-refreshed",
            "Suite component health refreshed",
        )?;
        let _ = self.logger.write(
            "info",
            "security-center.health-refreshed",
            "Suite component health refreshed",
        );
        self.snapshot()
    }

    pub fn clear_events(&self) -> Result<(), String> {
        self.lock_events()?
            .clear()
            .map_err(|error| error.to_string())?;
        let _ = self.logger.write(
            "info",
            "security-center.events-cleared",
            "Persistent activity history cleared",
        );
        Ok(())
    }

    pub fn settings(&self) -> Result<SecurityCenterSettings, String> {
        Ok(self.lock_settings()?.clone())
    }

    pub fn update_settings(
        &self,
        mut updated: SecurityCenterSettings,
    ) -> Result<SecurityCenterSettings, String> {
        let current = self.lock_settings()?.clone();
        updated.integrity_alert_cursor = current.integrity_alert_cursor;
        updated.automation_event_cursor = current.automation_event_cursor;
        updated.validate().map_err(|error| error.to_string())?;
        self.settings_store
            .save(&updated)
            .map_err(|error| error.to_string())?;
        *self.lock_settings()? = updated.clone();

        self.lock_events()?
            .set_capacity(updated.retain_event_count)
            .map_err(|error| error.to_string())?;
        self.lock_health_history()?
            .set_capacity(updated.suite_policy.health_history_limit)
            .map_err(|error| error.to_string())?;
        self.record_event(
            Component::SecurityCenter,
            EventKind::Configuration,
            Severity::Info,
            "security-center.settings-updated",
            "Security Center settings updated",
        )?;
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

    pub fn automation_status(&self) -> Result<AutomationStatus, String> {
        self.automation
            .as_ref()
            .ok_or_else(|| "Scheduled automation runtime is unavailable.".to_owned())?
            .status()
            .map_err(|error| error.to_string())
    }

    pub fn configure_automation_job(
        &self,
        job: &str,
        enabled: bool,
        interval_minutes: u64,
    ) -> Result<AutomationStatus, String> {
        let kind = AutomationJobKind::parse(job).map_err(|error| error.to_string())?;
        let status = self
            .automation
            .as_ref()
            .ok_or_else(|| "Scheduled automation runtime is unavailable.".to_owned())?
            .configure_job(kind, enabled, interval_minutes)
            .map_err(|error| error.to_string())?;
        self.record_event(
            Component::SecurityCenter,
            EventKind::Configuration,
            Severity::Info,
            "security-center.automation-policy-updated",
            "Scheduled protection automation policy updated",
        )?;
        Ok(status)
    }

    pub fn run_automation_job(&self, job: &str) -> Result<AutomationStatus, String> {
        let kind = AutomationJobKind::parse(job).map_err(|error| error.to_string())?;
        let status = self
            .automation
            .as_ref()
            .ok_or_else(|| "Scheduled automation runtime is unavailable.".to_owned())?
            .run_now(kind)
            .map_err(|error| error.to_string())?;
        let _ = self.sync_automation_events();
        Ok(status)
    }

    fn sync_automation_events(&self) -> Result<(), String> {
        let Some(runtime) = &self.automation else {
            return Ok(());
        };
        let status = runtime.status().map_err(|error| error.to_string())?;
        let after_id = self.lock_settings()?.automation_event_cursor;
        let mut events = status
            .history
            .into_iter()
            .filter(|event| event.id > after_id)
            .collect::<Vec<_>>();
        if events.is_empty() {
            return Ok(());
        }
        events.sort_by_key(|event| event.id);

        let mut newest = after_id;
        for event in events {
            newest = newest.max(event.id);
            let severity = match event.outcome.as_str() {
                "failed" => Severity::Critical,
                "attention" | "action_required" => Severity::Warning,
                "missed_recovered" => Severity::Notice,
                _ => Severity::Info,
            };
            self.record_event(
                Component::Agent,
                EventKind::Security,
                severity,
                format!(
                    "automation.{}.{}",
                    event.job.as_str(),
                    event.outcome.replace('_', "-")
                ),
                event.summary,
            )?;
        }

        let updated = {
            let mut settings = self.lock_settings()?;
            settings.automation_event_cursor = newest;
            settings.clone()
        };
        self.settings_store
            .save(&updated)
            .map_err(|error| error.to_string())
    }

    fn sync_integrity_events(&self) -> Result<(), String> {
        let suite = SuitePaths::discover().map_err(|error| error.to_string())?;
        let state_path = suite
            .component_data_dir(Component::Agent)
            .join("continuous-integrity-v1.json");
        let after_id = self.lock_settings()?.integrity_alert_cursor;
        let events =
            continuous_events(&state_path, after_id, 250).map_err(|error| error.to_string())?;
        if events.is_empty() {
            return Ok(());
        }

        let mut newest = after_id;
        let mut store = self.lock_events()?;
        for event in events {
            newest = newest.max(event.id);
            if event.suppressed {
                continue;
            }
            store
                .push(
                    Component::IntegrityMonitor,
                    EventKind::Security,
                    Severity::Warning,
                    "integrity-monitor.change-detected",
                    event.summary,
                )
                .map_err(|error| error.to_string())?;
        }
        drop(store);

        let updated = {
            let mut settings = self.lock_settings()?;
            settings.integrity_alert_cursor = newest;
            settings.clone()
        };
        self.settings_store
            .save(&updated)
            .map_err(|error| error.to_string())
    }

    fn record_agent_event(&self, code: &'static str, message: &'static str) -> Result<(), String> {
        self.record_event(
            Component::Agent,
            EventKind::Lifecycle,
            Severity::Info,
            code,
            message,
        )?;
        let _ = self.logger.write("info", code, message);
        Ok(())
    }

    pub fn launch_agent(&self) -> Result<(), String> {
        *self
            .agent_auto_start_suppressed
            .lock()
            .map_err(|_| "Agent lifecycle state is unavailable".to_owned())? = false;
        orchestration::launch_agent().map_err(|error| error.to_string())?;
        self.record_event(
            Component::SecurityCenter,
            EventKind::Lifecycle,
            Severity::Info,
            "security-center.agent-launched",
            "DragonForge Agent start requested",
        )?;
        let _ = self.logger.write(
            "info",
            "security-center.agent-launched",
            "DragonForge Agent start requested",
        );
        Ok(())
    }

    pub fn launch_authenticator(&self) -> Result<(), String> {
        orchestration::launch_authenticator().map_err(|error| error.to_string())?;
        self.record_event(
            Component::SecurityCenter,
            EventKind::Lifecycle,
            Severity::Info,
            "security-center.authenticator-launched",
            "Authenticator launch requested",
        )?;
        let _ = self.logger.write(
            "info",
            "security-center.authenticator-launched",
            "Authenticator launch requested",
        );
        Ok(())
    }

    pub fn launch_security_scanner(&self) -> Result<(), String> {
        orchestration::launch_security_scanner().map_err(|error| error.to_string())?;
        self.record_event(
            Component::SecurityCenter,
            EventKind::Lifecycle,
            Severity::Info,
            "security-center.security-scanner-launched",
            "Security Scanner launch requested",
        )?;
        let _ = self.logger.write(
            "info",
            "security-center.security-scanner-launched",
            "Security Scanner launch requested",
        );
        Ok(())
    }

    pub fn launch_integrity_monitor(&self) -> Result<(), String> {
        orchestration::launch_integrity_monitor().map_err(|error| error.to_string())?;
        self.record_event(
            Component::SecurityCenter,
            EventKind::Lifecycle,
            Severity::Info,
            "security-center.integrity-monitor-launched",
            "Integrity Monitor launch requested",
        )?;
        let _ = self.logger.write(
            "info",
            "security-center.integrity-monitor-launched",
            "Integrity Monitor launch requested",
        );
        Ok(())
    }

    pub fn launch_network_guard(&self) -> Result<(), String> {
        orchestration::launch_network_guard().map_err(|error| error.to_string())?;
        self.record_event(
            Component::SecurityCenter,
            EventKind::Lifecycle,
            Severity::Info,
            "security-center.network-guard-launched",
            "Network Guard launch requested",
        )?;
        let _ = self.logger.write(
            "info",
            "security-center.network-guard-launched",
            "Network Guard launch requested",
        );
        Ok(())
    }

    pub fn launch_secure_share(&self) -> Result<(), String> {
        orchestration::launch_secure_share().map_err(|error| error.to_string())?;
        self.record_event(
            Component::SecurityCenter,
            EventKind::Lifecycle,
            Severity::Info,
            "security-center.secure-share-launched",
            "Secure Share launch requested",
        )?;
        let _ = self.logger.write(
            "info",
            "security-center.secure-share-launched",
            "Secure Share launch requested",
        );
        Ok(())
    }

    pub fn launch_backup_recovery(&self) -> Result<(), String> {
        orchestration::launch_backup_recovery().map_err(|error| error.to_string())?;
        self.record_event(
            Component::SecurityCenter,
            EventKind::Lifecycle,
            Severity::Info,
            "security-center.backup-recovery-launched",
            "Backup & Recovery launch requested",
        )?;
        let _ = self.logger.write(
            "info",
            "security-center.backup-recovery-launched",
            "Backup & Recovery launch requested",
        );
        Ok(())
    }

    pub fn launch_file_vault(&self) -> Result<(), String> {
        orchestration::launch_file_vault().map_err(|error| error.to_string())?;
        self.record_event(
            Component::SecurityCenter,
            EventKind::Lifecycle,
            Severity::Info,
            "security-center.file-vault-launched",
            "File Vault launch requested",
        )?;
        let _ = self.logger.write(
            "info",
            "security-center.file-vault-launched",
            "File Vault launch requested",
        );
        Ok(())
    }

    pub fn launch_password_manager(&self) -> Result<(), String> {
        orchestration::launch_password_manager().map_err(|error| error.to_string())?;
        self.record_event(
            Component::SecurityCenter,
            EventKind::Lifecycle,
            Severity::Info,
            "security-center.password-manager-launched",
            "Password Manager launch requested",
        )?;
        let _ = self.logger.write(
            "info",
            "security-center.password-manager-launched",
            "Password Manager launch requested",
        );
        Ok(())
    }

    pub fn check_updates(&self) -> Result<UpdateStatus, String> {
        let channel = self.lock_settings()?.update_channel;
        let result = self.updater.check(channel);
        match &result {
            Ok(status) => {
                let summary = if status.available {
                    "Verified DragonForge update is available"
                } else {
                    "DragonForge update check completed"
                };
                self.record_event(
                    Component::SecurityCenter,
                    EventKind::Health,
                    Severity::Info,
                    "security-center.update-checked",
                    summary,
                )?;
                let _ = self
                    .logger
                    .write("info", "security-center.update-checked", summary);
            }
            Err(_) => {
                self.record_event(
                    Component::SecurityCenter,
                    EventKind::Health,
                    Severity::Warning,
                    "security-center.update-check-failed",
                    "Secure update check failed closed",
                )?;
                let _ = self.logger.write(
                    "warning",
                    "security-center.update-check-failed",
                    "Secure update check failed closed",
                );
            }
        }
        result
    }

    pub fn prepare_update(&self) -> Result<PreparedUpdateStatus, String> {
        let channel = self.lock_settings()?.update_channel;
        let result = self.updater.prepare(channel);
        match &result {
            Ok(_) => {
                self.record_event(
                    Component::SecurityCenter,
                    EventKind::Lifecycle,
                    Severity::Info,
                    "security-center.update-prepared",
                    "Update installer passed signature, hash, and Authenticode verification",
                )?;
                let _ = self.logger.write(
                    "info",
                    "security-center.update-prepared",
                    "Update installer passed signature, hash, and Authenticode verification",
                );
            }
            Err(_) => {
                self.record_event(
                    Component::SecurityCenter,
                    EventKind::Lifecycle,
                    Severity::Warning,
                    "security-center.update-prepare-failed",
                    "Update preparation failed closed; no installer was launched",
                )?;
                let _ = self.logger.write(
                    "warning",
                    "security-center.update-prepare-failed",
                    "Update preparation failed closed; no installer was launched",
                );
            }
        }
        result
    }

    pub fn install_prepared_update(&self) -> Result<(), String> {
        let result = self.updater.install_prepared();
        if result.is_ok() {
            self.record_event(
                Component::SecurityCenter,
                EventKind::Lifecycle,
                Severity::Info,
                "security-center.update-installer-launched",
                "User explicitly launched a re-verified DragonForge update installer",
            )?;
            let _ = self.logger.write(
                "info",
                "security-center.update-installer-launched",
                "User explicitly launched a re-verified DragonForge update installer",
            );
        }
        result
    }

    fn record_event(
        &self,
        component: Component,
        kind: EventKind,
        severity: Severity,
        code: impl Into<String>,
        summary: impl Into<String>,
    ) -> Result<(), String> {
        self.lock_events()?
            .push(component, kind, severity, code, summary)
            .map_err(|error| error.to_string())
    }

    fn lock_health_history(&self) -> Result<MutexGuard<'_, HealthHistoryStore>, String> {
        self.health_history
            .lock()
            .map_err(|_| "Security Center health history is unavailable".to_owned())
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
