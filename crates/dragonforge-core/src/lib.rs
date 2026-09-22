#![forbid(unsafe_code)]

//! Shared, non-cryptographic foundation for DragonForge Security Suite.
//!
//! This crate owns small cross-product primitives that do not belong to a
//! security-sensitive product domain. Cryptographic primitives, encrypted
//! storage formats, sync protocols, and product policy remain in dedicated
//! crates/components.

pub mod config;
pub mod diagnostics;
pub mod error;
pub mod event;
pub mod ipc;
pub mod platform;
pub mod redaction;

pub use config::{ConfigKey, ConfigSource, ResolvedConfig};
pub use diagnostics::{ComponentLogger, install_safe_panic_hook, sanitize_diagnostic_text};
pub use error::{CoreError, CoreResult, ErrorCode};
pub use event::{EventKind, EventRecord, Severity};
pub use ipc::{
    AuthenticationMechanism, CURRENT_PROTOCOL, IpcEnvelope, LocalIpcPolicy, PeerContext,
    ProtocolVersion, RequestId,
};
pub use platform::{Platform, SuitePaths};
pub use redaction::{LogPolicy, REDACTED, Secret};

/// Identifies a DragonForge suite component in logs, events and IPC metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Component {
    SecurityCenter,
    PasswordManager,
    Agent,
    FileVault,
    Authenticator,
    SecurityScanner,
    IntegrityMonitor,
    NetworkGuard,
    BackupRecovery,
    SecureShare,
}

impl Component {
    pub const ALL: [Self; 10] = [
        Self::SecurityCenter,
        Self::PasswordManager,
        Self::Agent,
        Self::FileVault,
        Self::Authenticator,
        Self::SecurityScanner,
        Self::IntegrityMonitor,
        Self::NetworkGuard,
        Self::BackupRecovery,
        Self::SecureShare,
    ];

    /// Stable machine-readable component identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SecurityCenter => "security-center",
            Self::PasswordManager => "password-manager",
            Self::Agent => "agent",
            Self::FileVault => "file-vault",
            Self::Authenticator => "authenticator",
            Self::SecurityScanner => "security-scanner",
            Self::IntegrityMonitor => "integrity-monitor",
            Self::NetworkGuard => "network-guard",
            Self::BackupRecovery => "backup-recovery",
            Self::SecureShare => "secure-share",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Component;

    #[test]
    fn component_ids_are_stable_and_nonempty() {
        for component in Component::ALL {
            assert!(!component.as_str().is_empty());
        }
    }
}
