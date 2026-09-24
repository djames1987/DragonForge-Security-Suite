#![forbid(unsafe_code)]

//! Authenticated local IPC, background runtime, and Phase 12.2 lifecycle controls for DragonForge Agent.
//!
//! The Agent remains intentionally per-user and non-elevated. It establishes
//! a real authenticated local boundary without claiming a privileged Windows
//! service or durable enforcement capability.

mod automation;
mod client;
mod error;
mod integrity;
mod paths;
#[cfg(windows)]
mod privileged;
mod protocol;
mod server;

pub use automation::{AgentAutomationRuntime, AutomationEvent, AutomationJob, AutomationJobKind, AutomationStatus, AUTOMATION_STATE_VERSION, MAX_AUTOMATION_EVENTS, MAX_INTERVAL_MINUTES, MIN_INTERVAL_MINUTES};
pub use client::{AgentClient, AgentHealth};
pub use error::{AgentError, Result};
pub use integrity::AgentIntegrityRuntime;
pub use paths::AgentPaths;
#[cfg(windows)]
pub use privileged::PrivilegedServiceClient;
pub use protocol::{AGENT_PROTOCOL_MAJOR, AGENT_PROTOCOL_MINOR, MAX_CLOCK_SKEW_MS};
pub use server::AgentServer;

#[cfg(windows)]
pub use dragonforge_windows_boundary::{
    FirewallAction, FirewallApplicationIdentity, FirewallMutationResult, FirewallPolicyState,
};
