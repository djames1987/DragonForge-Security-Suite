#![forbid(unsafe_code)]

//! Product-owned TOTP/HOTP engine and encrypted local store for DragonForge Authenticator.

mod error;
mod otp;
mod store;

pub use error::{AuthenticatorError, Result};
pub use otp::{OtpAlgorithm, OtpKind};
pub use store::{
    AccountView, CodeView, NewAccount, add_account, consume_hotp, create_store, generate_code,
    import_otpauth_uri, list_accounts, parse_otpauth_uri, remove_account, reveal_recovery_codes,
    set_recovery_codes,
};
