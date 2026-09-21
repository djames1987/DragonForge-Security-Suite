use dragonforge_core::{
    AuthenticationMechanism, Component, IpcEnvelope, LocalIpcPolicy, PeerContext, RequestId,
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentStatus {
    pub available: bool,
    pub state: &'static str,
    pub label: &'static str,
    pub detail: &'static str,
    pub transport: &'static str,
}

pub trait AgentClient: Send + Sync {
    fn status(&self) -> AgentStatus;
}

#[derive(Debug, Default)]
pub struct UnavailableAgentClient;

impl AgentClient for UnavailableAgentClient {
    fn status(&self) -> AgentStatus {
        AgentStatus {
            available: false,
            state: "unavailable",
            label: "Agent not installed",
            detail: "Background agent transport is reserved for a later phase.",
            transport: "none",
        }
    }
}

pub fn validate_future_agent_request(request_id: u128) -> bool {
    let envelope = IpcEnvelope::new(
        RequestId::new(request_id),
        Component::SecurityCenter,
        Component::Agent,
        "health",
    );
    let peer = PeerContext::authenticated(
        Component::SecurityCenter,
        1,
        AuthenticationMechanism::OperatingSystemPeer,
    );

    LocalIpcPolicy::new(Component::Agent, [Component::SecurityCenter])
        .authorize(peer, &envelope)
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::{AgentClient, UnavailableAgentClient, validate_future_agent_request};

    #[test]
    fn unavailable_agent_is_reported_without_false_connection() {
        let status = UnavailableAgentClient.status();
        assert!(!status.available);
        assert_eq!(status.transport, "none");
    }

    #[test]
    fn future_agent_request_matches_phase2_policy() {
        assert!(validate_future_agent_request(7));
    }
}
