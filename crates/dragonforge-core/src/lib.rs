#![forbid(unsafe_code)]

//! Shared, non-cryptographic foundation for DragonForge Security Suite.
//!
//! Keep this crate intentionally small. Cryptographic primitives, encrypted
//! storage formats and product-specific policy belong in dedicated crates.

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
        let components = [
            Component::SecurityCenter,
            Component::PasswordManager,
            Component::Agent,
            Component::FileVault,
            Component::Authenticator,
            Component::SecurityScanner,
            Component::IntegrityMonitor,
            Component::NetworkGuard,
            Component::BackupRecovery,
            Component::SecureShare,
        ];

        for component in components {
            assert!(!component.as_str().is_empty());
        }
    }
}
