use std::{error::Error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthenticatorError {
    InvalidPassword,
    PasswordTooShort,
    InvalidStore,
    UnsupportedFormat,
    StoreExists,
    StoreNotFound,
    SymlinkNotAllowed,
    InvalidAccount,
    InvalidSecret,
    InvalidOtpUri,
    AccountNotFound,
    TooManyAccounts,
    TooManyRecoveryCodes,
    InvalidRecoveryCode,
    Io,
    Crypto,
}

impl AuthenticatorError {
    #[must_use]
    pub const fn safe_message(self) -> &'static str {
        match self {
            Self::InvalidPassword => "the Authenticator password is incorrect or the store is corrupted",
            Self::PasswordTooShort => "Authenticator passwords must be at least 12 characters",
            Self::InvalidStore => "the Authenticator store is invalid or corrupted",
            Self::UnsupportedFormat => "the Authenticator store version is unsupported",
            Self::StoreExists => "an Authenticator store already exists at this location",
            Self::StoreNotFound => "the Authenticator store does not exist",
            Self::SymlinkNotAllowed => "symbolic links are not accepted for the Authenticator store",
            Self::InvalidAccount => "the authenticator account is invalid",
            Self::InvalidSecret => "the authenticator secret is invalid",
            Self::InvalidOtpUri => "the otpauth URI is invalid or unsupported",
            Self::AccountNotFound => "the authenticator account was not found",
            Self::TooManyAccounts => "the Authenticator store contains too many accounts",
            Self::TooManyRecoveryCodes => "the account contains too many recovery codes",
            Self::InvalidRecoveryCode => "a recovery code is invalid",
            Self::Io => "a local Authenticator file operation failed",
            Self::Crypto => "an Authenticator cryptographic operation failed",
        }
    }
}

impl fmt::Display for AuthenticatorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.safe_message())
    }
}

impl Error for AuthenticatorError {}

pub type Result<T> = std::result::Result<T, AuthenticatorError>;
