//! Shared, redaction-safe error primitives.

use std::{error::Error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    InvalidConfiguration,
    UnsupportedPlatform,
    AuthenticationRequired,
    AuthorizationDenied,
    ProtocolMismatch,
    InvalidRequest,
    Internal,
}

impl ErrorCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidConfiguration => "invalid-configuration",
            Self::UnsupportedPlatform => "unsupported-platform",
            Self::AuthenticationRequired => "authentication-required",
            Self::AuthorizationDenied => "authorization-denied",
            Self::ProtocolMismatch => "protocol-mismatch",
            Self::InvalidRequest => "invalid-request",
            Self::Internal => "internal",
        }
    }
}

/// Error type intended for cross-component foundation failures.
///
/// The safe message must be suitable for logs and UI surfaces. Callers must not
/// place passwords, keys, tokens, vault contents, or other secrets in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreError {
    code: ErrorCode,
    safe_message: String,
}

impl CoreError {
    #[must_use]
    pub fn new_safe(code: ErrorCode, safe_message: impl Into<String>) -> Self {
        Self {
            code,
            safe_message: safe_message.into(),
        }
    }

    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        self.code
    }

    #[must_use]
    pub fn safe_message(&self) -> &str {
        &self.safe_message
    }
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code.as_str(), self.safe_message)
    }
}

impl Error for CoreError {}

pub type CoreResult<T> = Result<T, CoreError>;

#[cfg(test)]
mod tests {
    use super::{CoreError, ErrorCode};

    #[test]
    fn error_code_ids_are_stable() {
        assert_eq!(
            ErrorCode::AuthenticationRequired.as_str(),
            "authentication-required"
        );
        assert_eq!(
            ErrorCode::ProtocolMismatch.as_str(),
            "protocol-mismatch"
        );
    }

    #[test]
    fn display_contains_code_and_safe_message() {
        let error = CoreError::new_safe(ErrorCode::InvalidRequest, "request was rejected");
        assert_eq!(error.to_string(), "invalid-request: request was rejected");
    }
}
