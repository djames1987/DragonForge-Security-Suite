#![forbid(unsafe_code)]

//! Authenticated local IPC, background runtime, lifecycle, integrity, automation,
//! and narrow privileged-service coordination for DragonForge Agent.
//!
//! The Agent remains intentionally per-user and non-elevated. Optional privileged
//! firewall policy mutation is delegated to the separately authenticated,
//! publisher-pinned DragonForge Privileged Service; generic elevated execution
//! remains prohibited.

mod automation;
mod client;
mod error;
mod integrity;
mod paths;
#[cfg(windows)]
mod privileged;
mod protocol;
mod server;

pub use automation::{
    AUTOMATION_STATE_VERSION, AgentAutomationRuntime, AutomationEvent, AutomationJob,
    AutomationJobKind, AutomationStatus, MAX_AUTOMATION_EVENTS, MAX_INTERVAL_MINUTES,
    MIN_INTERVAL_MINUTES,
};
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
