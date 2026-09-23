#![forbid(unsafe_code)]

//! Phase 16 privileged Windows service-boundary policy.
//!
//! This crate deliberately contains policy only. It does not install, start,
//! elevate, impersonate, or execute privileged operations.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const SERVICE_NAME: &str = "DragonForgePrivilegedService";
pub const SERVICE_DISPLAY_NAME: &str = "DragonForge Privileged Service";
pub const SERVICE_ACCOUNT: &str = r"NT SERVICE\DragonForgePrivilegedService";
pub const PIPE_NAME: &str = r"\\.\pipe\DragonForgePrivilegedService-v1";
pub const PROTOCOL_MAJOR: u16 = 1;
pub const PROTOCOL_MINOR: u16 = 1;
pub const MAX_MESSAGE_BYTES: usize = 16 * 1024;
pub const MAX_CLOCK_SKEW_SECONDS: u64 = 60;
pub const EXPECTED_AGENT_EXE: &str = "dragonforge-agent.exe";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceCommand {
    Health,
    DescribePolicy,
    FirewallStatus,
    FirewallApply,
    FirewallRemove,
    FirewallRollback,
}

impl ServiceCommand {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Health => "health",
            Self::DescribePolicy => "describe-policy",
            Self::FirewallStatus => "firewall-status",
            Self::FirewallApply => "firewall-apply",
            Self::FirewallRemove => "firewall-remove",
            Self::FirewallRollback => "firewall-rollback",
        }
    }

    #[must_use]
    pub const fn is_privileged(self) -> bool {
        matches!(
            self,
            Self::FirewallApply | Self::FirewallRemove | Self::FirewallRollback
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrivilegedCapability {
    FirewallPolicyMutation,
    ProtectedProcessControl,
    ProtectedFileQuarantine,
    ProtectedRegistryRemediation,
    SystemIntegrityRemediation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityPolicy;

impl CapabilityPolicy {
    #[must_use]
    pub const fn phase16_allows(_capability: PrivilegedCapability) -> bool {
        false
    }

    #[must_use]
    pub const fn phase17_allows(_capability: PrivilegedCapability) -> bool {
        false
    }

    #[must_use]
    pub const fn phase19_allows(capability: PrivilegedCapability) -> bool {
        matches!(capability, PrivilegedCapability::FirewallPolicyMutation)
    }
}


#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FirewallAction {
    Allow,
    Block,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FirewallApplicationIdentity {
    pub application_path: PathBuf,
    pub sha256_hex: String,
    pub display_name: String,
}

impl FirewallApplicationIdentity {
    pub fn validate(&self) -> Result<(), BoundaryError> {
        let file_name = self
            .application_path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or(BoundaryError::InvalidFirewallRequest)?;
        let normalized_file_name = file_name.to_ascii_lowercase();
        let path_text = self.application_path.to_string_lossy();
        if !self.application_path.is_absolute()
            || !normalized_file_name.ends_with(".exe")
            || path_text.starts_with(r"\")
            || self.application_path.as_os_str().len() > 2048
            || matches!(
                normalized_file_name.as_str(),
                "dragonforge-agent.exe" | "dragonforge-privileged-service.exe"
            )
        {
            return Err(BoundaryError::InvalidFirewallRequest);
        }
        if self.sha256_hex.len() != 64
            || !self.sha256_hex.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(BoundaryError::InvalidFirewallRequest);
        }
        let name = self.display_name.trim();
        if name.is_empty()
            || name.chars().count() > 120
            || name.chars().any(char::is_control)
        {
            return Err(BoundaryError::InvalidFirewallRequest);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FirewallMutationRequest {
    pub identity: FirewallApplicationIdentity,
    pub action: FirewallAction,
    #[serde(default)]
    pub rollback_token: Option<String>,
}

impl FirewallMutationRequest {
    pub fn validate(&self) -> Result<(), BoundaryError> {
        self.identity.validate()?;
        if let Some(token) = &self.rollback_token {
            if token.len() != 32 || !token.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err(BoundaryError::InvalidFirewallRequest);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FirewallPolicyState {
    pub rule_name: String,
    pub application_path: PathBuf,
    pub sha256_hex: String,
    pub action: FirewallAction,
    pub enabled: bool,
    pub identity_matches: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FirewallMutationResult {
    pub changed: bool,
    pub rollback_token: Option<String>,
    pub policy: Option<FirewallPolicyState>,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PrivilegedRequest {
    pub protocol_major: u16,
    pub protocol_minor: u16,
    pub request_id: u64,
    pub action: String,
    pub timestamp_ms: u64,
    pub nonce_hex: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firewall: Option<FirewallMutationRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PrivilegedServiceHealth {
    pub state: String,
    pub pid: u32,
    pub uptime_ms: u64,
    pub service_name: String,
    pub service_account: String,
    pub privileged_capabilities_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PrivilegedPolicyDescription {
    pub allowed_commands: Vec<String>,
    pub privileged_capabilities_enabled: bool,
    pub arbitrary_command_execution_prohibited: bool,
    pub generic_shell_execution_prohibited: bool,
    pub max_message_bytes: usize,
    pub firewall_policy_mutation_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PrivilegedResponse {
    pub request_id: u64,
    pub ok: bool,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<PrivilegedServiceHealth>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<PrivilegedPolicyDescription>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firewall: Option<FirewallMutationResult>,
}

impl PrivilegedResponse {
    #[must_use]
    pub fn error(request_id: u64, code: impl Into<String>) -> Self {
        Self {
            request_id,
            ok: false,
            code: code.into(),
            health: None,
            policy: None,
            firewall: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallerIdentity {
    pub executable_path: PathBuf,
    pub authenticode_valid: bool,
    pub publisher_subject: Option<String>,
    pub operating_system_peer_verified: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundaryPolicy {
    expected_agent_path: PathBuf,
    expected_publisher_subject: String,
}

impl BoundaryPolicy {
    #[must_use]
    pub fn new(
        install_directory: impl Into<PathBuf>,
        expected_publisher_subject: impl Into<String>,
    ) -> Self {
        Self {
            expected_agent_path: install_directory.into().join(EXPECTED_AGENT_EXE),
            expected_publisher_subject: expected_publisher_subject.into(),
        }
    }

    #[must_use]
    pub fn from_expected_agent_path(
        expected_agent_path: impl Into<PathBuf>,
        expected_publisher_subject: impl Into<String>,
    ) -> Self {
        Self {
            expected_agent_path: expected_agent_path.into(),
            expected_publisher_subject: expected_publisher_subject.into(),
        }
    }

    #[must_use]
    pub fn expected_agent_path(&self) -> PathBuf {
        self.expected_agent_path.clone()
    }

    pub fn authorize_caller(&self, caller: &CallerIdentity) -> Result<(), BoundaryError> {
        if !caller.operating_system_peer_verified {
            return Err(BoundaryError::OperatingSystemPeerRequired);
        }
        if !same_path(&caller.executable_path, &self.expected_agent_path()) {
            return Err(BoundaryError::UnexpectedExecutable);
        }
        if !caller.authenticode_valid {
            return Err(BoundaryError::InvalidAuthenticode);
        }
        let Some(subject) = caller.publisher_subject.as_deref() else {
            return Err(BoundaryError::PublisherMismatch);
        };
        if subject != self.expected_publisher_subject {
            return Err(BoundaryError::PublisherMismatch);
        }
        Ok(())
    }

    pub fn authorize_command(&self, command: &str) -> Result<ServiceCommand, BoundaryError> {
        match command {
            "health" => Ok(ServiceCommand::Health),
            "describe-policy" => Ok(ServiceCommand::DescribePolicy),
            "firewall-status" => Ok(ServiceCommand::FirewallStatus),
            "firewall-apply" => Ok(ServiceCommand::FirewallApply),
            "firewall-remove" => Ok(ServiceCommand::FirewallRemove),
            "firewall-rollback" => Ok(ServiceCommand::FirewallRollback),
            _ => Err(BoundaryError::CommandDenied),
        }
    }
}

fn same_path(left: &Path, right: &Path) -> bool {
    #[cfg(target_os = "windows")]
    {
        left.to_string_lossy()
            .eq_ignore_ascii_case(&right.to_string_lossy())
    }
    #[cfg(not(target_os = "windows"))]
    {
        left == right
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryError {
    OperatingSystemPeerRequired,
    UnexpectedExecutable,
    InvalidAuthenticode,
    PublisherMismatch,
    CommandDenied,
    InvalidFirewallRequest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowsSecurityBoundary {
    pub service_runs_as_virtual_account: bool,
    pub named_pipe_is_local_only: bool,
    pub service_sid_restricted: bool,
    pub os_peer_identity_required: bool,
    pub exact_agent_path_required: bool,
    pub authenticode_required: bool,
    pub publisher_pin_required: bool,
    pub replay_protection_required: bool,
    pub bounded_messages_required: bool,
    pub arbitrary_command_execution_prohibited: bool,
    pub generic_shell_execution_prohibited: bool,
}

impl Default for WindowsSecurityBoundary {
    fn default() -> Self {
        Self {
            service_runs_as_virtual_account: true,
            named_pipe_is_local_only: true,
            service_sid_restricted: true,
            os_peer_identity_required: true,
            exact_agent_path_required: true,
            authenticode_required: true,
            publisher_pin_required: true,
            replay_protection_required: true,
            bounded_messages_required: true,
            arbitrary_command_execution_prohibited: true,
            generic_shell_execution_prohibited: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> BoundaryPolicy {
        BoundaryPolicy::new(r"C:\Program Files\DragonForge", "CN=DragonForge Software")
    }

    fn valid_caller() -> CallerIdentity {
        CallerIdentity {
            executable_path: PathBuf::from(r"C:\Program Files\DragonForge\dragonforge-agent.exe"),
            authenticode_valid: true,
            publisher_subject: Some("CN=DragonForge Software".to_owned()),
            operating_system_peer_verified: true,
        }
    }

    #[test]
    fn phase16_exposes_no_privileged_capabilities() {
        for capability in [
            PrivilegedCapability::FirewallPolicyMutation,
            PrivilegedCapability::ProtectedProcessControl,
            PrivilegedCapability::ProtectedFileQuarantine,
            PrivilegedCapability::ProtectedRegistryRemediation,
            PrivilegedCapability::SystemIntegrityRemediation,
        ] {
            assert!(!CapabilityPolicy::phase16_allows(capability));
            assert!(!CapabilityPolicy::phase17_allows(capability));
            assert_eq!(
                CapabilityPolicy::phase19_allows(capability),
                capability == PrivilegedCapability::FirewallPolicyMutation
            );
        }
    }

    #[test]
    fn only_fixed_non_privileged_commands_are_authorized() {
        assert_eq!(
            policy().authorize_command("health"),
            Ok(ServiceCommand::Health)
        );
        assert_eq!(
            policy().authorize_command("describe-policy"),
            Ok(ServiceCommand::DescribePolicy)
        );
        for allowed in [
            "firewall-status",
            "firewall-apply",
            "firewall-remove",
            "firewall-rollback",
        ] {
            assert!(policy().authorize_command(allowed).is_ok());
        }
        for denied in ["exec", "shell", "powershell", "cmd", "run", "firewall-add"] {
            assert_eq!(
                policy().authorize_command(denied),
                Err(BoundaryError::CommandDenied)
            );
        }
    }

    #[test]
    fn firewall_identity_rejects_control_plane_and_network_paths() {
        for path in [
            r"C:\DragonForge\dragonforge-agent.exe",
            r"C:\DragonForge\dragonforge-privileged-service.exe",
            r"\\server\share\application.exe",
        ] {
            let identity = FirewallApplicationIdentity {
                application_path: PathBuf::from(path),
                sha256_hex: "00".repeat(32),
                display_name: "test".to_owned(),
            };
            assert_eq!(
                identity.validate(),
                Err(BoundaryError::InvalidFirewallRequest)
            );
        }
    }

    #[test]
    fn caller_requires_os_identity_exact_path_signature_and_publisher() {
        let policy = policy();
        assert!(policy.authorize_caller(&valid_caller()).is_ok());

        let mut caller = valid_caller();
        caller.operating_system_peer_verified = false;
        assert_eq!(
            policy.authorize_caller(&caller),
            Err(BoundaryError::OperatingSystemPeerRequired)
        );

        let mut caller = valid_caller();
        caller.executable_path = PathBuf::from(r"C:\Temp\dragonforge-agent.exe");
        assert_eq!(
            policy.authorize_caller(&caller),
            Err(BoundaryError::UnexpectedExecutable)
        );

        let mut caller = valid_caller();
        caller.authenticode_valid = false;
        assert_eq!(
            policy.authorize_caller(&caller),
            Err(BoundaryError::InvalidAuthenticode)
        );

        let mut caller = valid_caller();
        caller.publisher_subject = Some("CN=Other Publisher".to_owned());
        assert_eq!(
            policy.authorize_caller(&caller),
            Err(BoundaryError::PublisherMismatch)
        );
    }

    #[test]
    fn boundary_defaults_fail_closed_against_generic_execution() {
        let boundary = WindowsSecurityBoundary::default();
        assert!(boundary.service_runs_as_virtual_account);
        assert!(boundary.named_pipe_is_local_only);
        assert!(boundary.service_sid_restricted);
        assert!(boundary.os_peer_identity_required);
        assert!(boundary.exact_agent_path_required);
        assert!(boundary.authenticode_required);
        assert!(boundary.publisher_pin_required);
        assert!(boundary.replay_protection_required);
        assert!(boundary.bounded_messages_required);
        assert!(boundary.arbitrary_command_execution_prohibited);
        assert!(boundary.generic_shell_execution_prohibited);
    }
}
