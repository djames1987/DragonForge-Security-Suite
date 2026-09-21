#![forbid(unsafe_code)]

//! Offline encrypted secure-share packages for DragonForge Security Suite.
//!
//! Phase 10 packages are password-protected and recipient-labelled. Expiration is
//! enforced by the native open/extract operations. Offline packages do not claim
//! revocation or reliable open-count enforcement.

mod error;
mod format;
mod package;

pub use error::{Result, ShareError};
pub use format::{FORMAT_VERSION, SHARE_EXTENSION};
pub use package::{
    CreateShareOptions, ShareSummary, create_share, extract_attachments, reveal_secret,
    verify_share,
};
