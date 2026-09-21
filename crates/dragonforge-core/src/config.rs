//! Shared configuration primitives.

use crate::Component;
use crate::error::{CoreError, CoreResult, ErrorCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigSource {
    Default,
    File,
    Environment,
    Runtime,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConfigKey(String);

impl ConfigKey {
    pub fn parse(value: impl Into<String>) -> CoreResult<Self> {
        let value = value.into();
        if value.is_empty() || value.len() > 96 {
            return Err(CoreError::new_safe(
                ErrorCode::InvalidConfiguration,
                "configuration key length is invalid",
            ));
        }

        let bytes = value.as_bytes();
        let edge_valid = |byte: u8| byte.is_ascii_lowercase() || byte.is_ascii_digit();
        if !edge_valid(bytes[0]) || !edge_valid(bytes[bytes.len() - 1]) {
            return Err(CoreError::new_safe(
                ErrorCode::InvalidConfiguration,
                "configuration key must start and end with an alphanumeric character",
            ));
        }

        if !bytes.iter().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(*byte, b'.' | b'_' | b'-')
        }) {
            return Err(CoreError::new_safe(
                ErrorCode::InvalidConfiguration,
                "configuration key contains unsupported characters",
            ));
        }

        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn environment_variable(&self, component: Component) -> String {
        let component = component.as_str().replace('-', "_").to_ascii_uppercase();
        let key = self
            .as_str()
            .chars()
            .map(|character| match character {
                '.' | '-' => '_',
                other => other.to_ascii_uppercase(),
            })
            .collect::<String>();

        format!("DRAGONFORGE_{component}_{key}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedConfig<T> {
    value: T,
    source: ConfigSource,
}

impl<T> ResolvedConfig<T> {
    #[must_use]
    pub const fn new(value: T, source: ConfigSource) -> Self {
        Self { value, source }
    }

    #[must_use]
    pub const fn value(&self) -> &T {
        &self.value
    }

    #[must_use]
    pub const fn source(&self) -> ConfigSource {
        self.source
    }

    #[must_use]
    pub fn into_value(self) -> T {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::{ConfigKey, ConfigSource, ResolvedConfig};
    use crate::Component;

    #[test]
    fn validates_and_maps_config_keys() {
        let key = ConfigKey::parse("network.timeout-seconds").expect("valid key");
        assert_eq!(
            key.environment_variable(Component::SecurityCenter),
            "DRAGONFORGE_SECURITY_CENTER_NETWORK_TIMEOUT_SECONDS"
        );
    }

    #[test]
    fn rejects_invalid_keys() {
        assert!(ConfigKey::parse("Bad Key").is_err());
        assert!(ConfigKey::parse(".hidden").is_err());
        assert!(ConfigKey::parse("trailing.").is_err());
    }

    #[test]
    fn resolved_config_retains_source() {
        let config = ResolvedConfig::new(30_u64, ConfigSource::Default);
        assert_eq!(config.value(), &30);
        assert_eq!(config.source(), ConfigSource::Default);
    }
}
