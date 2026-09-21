use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum AgentError {
    Io(&'static str),
    InvalidState(&'static str),
    Protocol(&'static str),
    Authentication(&'static str),
    Authorization(&'static str),
    Unavailable(&'static str),
}

impl Display for AgentError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Io(message)
            | Self::InvalidState(message)
            | Self::Protocol(message)
            | Self::Authentication(message)
            | Self::Authorization(message)
            | Self::Unavailable(message) => message,
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for AgentError {}

pub type Result<T> = std::result::Result<T, AgentError>;
