use std::{error::Error, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileVaultError {
    InvalidPassword,
    PasswordTooShort,
    InvalidContainer,
    UnsupportedFormat,
    InvalidPath,
    SourceNotFound,
    SymlinkNotAllowed,
    DestinationExists,
    TooManyEntries,
    ContainerTooLarge,
    Io,
    Crypto,
}

impl FileVaultError {
    #[must_use]
    pub const fn safe_message(self) -> &'static str {
        match self {
            Self::InvalidPassword => "the password is incorrect or the container is corrupted",
            Self::PasswordTooShort => "File Vault passwords must be at least 12 characters",
            Self::InvalidContainer => "the File Vault container is invalid or corrupted",
            Self::UnsupportedFormat => "the File Vault container version is unsupported",
            Self::InvalidPath => "a file path is invalid for a secure container",
            Self::SourceNotFound => "a selected source file or folder does not exist",
            Self::SymlinkNotAllowed => "symbolic links are not accepted by File Vault",
            Self::DestinationExists => "the destination already contains one of these files",
            Self::TooManyEntries => "the container contains too many entries",
            Self::ContainerTooLarge => "the container exceeds the Phase 4 size limit",
            Self::Io => "a local file operation failed",
            Self::Crypto => "a File Vault cryptographic operation failed",
        }
    }
}

impl fmt::Display for FileVaultError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.safe_message())
    }
}

impl Error for FileVaultError {}

pub type Result<T> = std::result::Result<T, FileVaultError>;
