//! Redaction primitives and safe logging policy.

use std::fmt;

pub const REDACTED: &str = "[REDACTED]";

/// Wrapper for values that must not be emitted through Debug or Display.
///
/// This prevents accidental formatting disclosure. It does not zeroize memory
/// and is not a replacement for a dedicated secret-memory container.
pub struct Secret<T>(T);

impl<T> Secret<T> {
    #[must_use]
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn expose_secret(&self) -> &T {
        &self.0
    }

    #[must_use]
    pub fn into_secret(self) -> T {
        self.0
    }
}

impl<T> fmt::Debug for Secret<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(REDACTED)
    }
}

impl<T> fmt::Display for Secret<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(REDACTED)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogPolicy {
    pub max_public_field_chars: usize,
    pub include_identifiers: bool,
}

impl Default for LogPolicy {
    fn default() -> Self {
        Self {
            max_public_field_chars: 256,
            include_identifiers: false,
        }
    }
}

impl LogPolicy {
    #[must_use]
    pub fn sanitize_public(self, value: &str) -> String {
        let mut output = String::with_capacity(value.len().min(self.max_public_field_chars));
        for character in value.chars().take(self.max_public_field_chars) {
            if character.is_control() {
                output.push(' ');
            } else {
                output.push(character);
            }
        }
        output
    }

    #[must_use]
    pub fn identifier(self, value: &str) -> String {
        if self.include_identifiers {
            self.sanitize_public(value)
        } else {
            REDACTED.to_owned()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LogPolicy, REDACTED, Secret};

    #[test]
    fn secret_debug_and_display_never_expose_value() {
        let secret = Secret::new("correct horse battery staple");
        assert_eq!(format!("{secret:?}"), REDACTED);
        assert_eq!(format!("{secret}"), REDACTED);
        assert_eq!(secret.expose_secret(), &"correct horse battery staple");
    }

    #[test]
    fn default_log_policy_redacts_identifiers() {
        let policy = LogPolicy::default();
        assert_eq!(policy.identifier("device-123"), REDACTED);
    }

    #[test]
    fn public_fields_are_single_line_and_bounded() {
        let policy = LogPolicy {
            max_public_field_chars: 8,
            include_identifiers: true,
        };
        assert_eq!(policy.sanitize_public("abc\ndefghijkl"), "abc defg");
    }
}
