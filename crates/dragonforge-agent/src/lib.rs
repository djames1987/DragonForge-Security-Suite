#![forbid(unsafe_code)]

//! Authenticated local IPC, background runtime, and Phase 12.2 lifecycle controls for DragonForge Agent.
//!
//! The Agent remains intentionally per-user and non-elevated. It establishes
//! a real authenticated local boundary without claiming a privileged Windows
//! service or durable enforcement capability.

mod client;
mod error;
mod paths;
mod protocol;
mod server;

pub use client::{AgentClient, AgentHealth};
pub use error::{AgentError, Result};
pub use paths::AgentPaths;
pub use protocol::{AGENT_PROTOCOL_MAJOR, AGENT_PROTOCOL_MINOR, MAX_CLOCK_SKEW_MS};
pub use server::AgentServer;
