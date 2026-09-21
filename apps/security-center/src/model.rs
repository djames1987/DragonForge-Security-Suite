use dragonforge_core::Component;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentState {
    Active,
    Integrated,
    Planned,
    Unavailable,
    Attention,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ComponentStatus {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub state: ComponentState,
    pub state_label: &'static str,
    pub detail: &'static str,
}

impl ComponentStatus {
    #[must_use]
    pub const fn new(
        component: Component,
        name: &'static str,
        description: &'static str,
        state: ComponentState,
        state_label: &'static str,
        detail: &'static str,
    ) -> Self {
        Self {
            id: component.as_str(),
            name,
            description,
            state,
            state_label,
            detail,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HealthSummary {
    pub state: &'static str,
    pub label: &'static str,
    pub active: usize,
    pub integrated: usize,
    pub planned: usize,
    pub attention: usize,
}

#[derive(Debug, Clone)]
pub struct ComponentRegistry {
    components: Vec<ComponentStatus>,
}

impl ComponentRegistry {
    #[must_use]
    pub fn phase3_default() -> Self {
        Self {
            components: vec![
                ComponentStatus::new(
                    Component::SecurityCenter,
                    "Security Center",
                    "Unified suite dashboard and orchestration surface.",
                    ComponentState::Active,
                    "Active",
                    "Phase 3 dashboard is running.",
                ),
                ComponentStatus::new(
                    Component::PasswordManager,
                    "Password Manager",
                    "Encrypted local vault, sync, recovery, and browser integration.",
                    ComponentState::Integrated,
                    "Integrated",
                    "Migrated and verified in the suite workspace.",
                ),
                ComponentStatus::new(
                    Component::Agent,
                    "DragonForge Agent",
                    "Future background monitoring and protected local operations.",
                    ComponentState::Unavailable,
                    "Not installed",
                    "Agent transport and service arrive in a later phase.",
                ),
                ComponentStatus::new(
                    Component::FileVault,
                    "File Vault",
                    "Encrypted files, folders, and secure containers.",
                    ComponentState::Planned,
                    "Planned",
                    "Scheduled for Phase 4.",
                ),
                ComponentStatus::new(
                    Component::Authenticator,
                    "Authenticator",
                    "TOTP/HOTP and recovery material management.",
                    ComponentState::Planned,
                    "Planned",
                    "Scheduled for Phase 5.",
                ),
                ComponentStatus::new(
                    Component::SecurityScanner,
                    "Security Scanner",
                    "System security posture and configuration assessment.",
                    ComponentState::Planned,
                    "Planned",
                    "Scheduled for Phase 6.",
                ),
                ComponentStatus::new(
                    Component::IntegrityMonitor,
                    "Integrity Monitor",
                    "File and system configuration baseline monitoring.",
                    ComponentState::Planned,
                    "Planned",
                    "Scheduled for Phase 7.",
                ),
                ComponentStatus::new(
                    Component::NetworkGuard,
                    "Network Guard",
                    "Per-process network visibility and enforcement foundation.",
                    ComponentState::Planned,
                    "Planned",
                    "Scheduled for Phase 8.",
                ),
                ComponentStatus::new(
                    Component::BackupRecovery,
                    "Backup & Recovery",
                    "Encrypted, verifiable suite and user-data recovery.",
                    ComponentState::Planned,
                    "Planned",
                    "Scheduled for Phase 9.",
                ),
                ComponentStatus::new(
                    Component::SecureShare,
                    "Secure Share",
                    "Encrypted recipient-oriented packages and secrets.",
                    ComponentState::Planned,
                    "Planned",
                    "Scheduled for Phase 10.",
                ),
            ],
        }
    }

    #[must_use]
    pub fn all(&self) -> &[ComponentStatus] {
        &self.components
    }

    #[must_use]
    pub fn health_summary(&self) -> HealthSummary {
        let mut active = 0;
        let mut integrated = 0;
        let mut planned = 0;
        let mut attention = 0;

        for component in &self.components {
            match component.state {
                ComponentState::Active => active += 1,
                ComponentState::Integrated => integrated += 1,
                ComponentState::Planned | ComponentState::Unavailable => planned += 1,
                ComponentState::Attention => attention += 1,
            }
        }

        if attention > 0 {
            HealthSummary {
                state: "attention",
                label: "Attention required",
                active,
                integrated,
                planned,
                attention,
            }
        } else {
            HealthSummary {
                state: "healthy",
                label: "Foundation healthy",
                active,
                integrated,
                planned,
                attention,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ComponentRegistry, ComponentState};

    #[test]
    fn phase3_registry_contains_current_and_future_components() {
        let registry = ComponentRegistry::phase3_default();
        assert_eq!(registry.all().len(), 10);
        assert_eq!(registry.all()[0].state, ComponentState::Active);
        assert_eq!(registry.all()[1].state, ComponentState::Integrated);
    }

    #[test]
    fn planned_components_do_not_create_false_alarm() {
        let summary = ComponentRegistry::phase3_default().health_summary();
        assert_eq!(summary.state, "healthy");
        assert_eq!(summary.attention, 0);
    }
}
