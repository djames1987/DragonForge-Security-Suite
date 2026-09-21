use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum ShareError {
    Io(&'static str),
    InvalidInput(&'static str),
    Format(&'static str),
    Crypto(&'static str),
    Integrity(&'static str),
    Conflict(&'static str),
    Expired(&'static str),
}

impl Display for ShareError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Io(message)
            | Self::InvalidInput(message)
            | Self::Format(message)
            | Self::Crypto(message)
            | Self::Integrity(message)
            | Self::Conflict(message)
            | Self::Expired(message) => message,
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ShareError {}

pub type Result<T> = std::result::Result<T, ShareError>;
