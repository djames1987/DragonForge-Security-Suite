use dragonforge_agent::{AgentClient as RuntimeAgentClient, AgentHealth};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentStatus {
    pub available: bool,
    pub state: String,
    pub label: String,
    pub detail: String,
    pub transport: String,
    pub pid: Option<u32>,
    pub uptime_ms: Option<u64>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AgentClient {
    runtime: Option<RuntimeAgentClient>,
}

impl AgentClient {
    #[must_use]
    pub fn discover() -> Self {
        Self {
            runtime: RuntimeAgentClient::discover().ok(),
        }
    }

    #[cfg(test)]
    #[must_use]
    pub const fn unavailable() -> Self {
        Self { runtime: None }
    }

    pub fn shutdown(&self) -> Result<(), String> {
        self.runtime
            .as_ref()
            .ok_or_else(|| "DragonForge Agent runtime paths are unavailable.".to_owned())?
            .shutdown()
            .map_err(|error| error.to_string())
    }

    #[must_use]
    pub fn status(&self) -> AgentStatus {
        let health = match &self.runtime {
            Some(client) => client
                .health()
                .unwrap_or_else(|error| AgentHealth::unavailable(error.to_string())),
            None => AgentHealth::unavailable("DragonForge Agent runtime paths are unavailable."),
        };
        status_from_health(health)
    }
}

fn status_from_health(health: AgentHealth) -> AgentStatus {
    if health.available {
        let pid = health.pid.unwrap_or_default();
        let uptime = health.uptime_ms.unwrap_or_default();
        AgentStatus {
            available: true,
            state: health.state,
            label: "Agent connected".to_owned(),
            detail: format!(
                "Authenticated local agent is healthy (PID {pid}, uptime {}s).",
                uptime / 1_000
            ),
            transport: health.transport,
            pid: health.pid,
            uptime_ms: health.uptime_ms,
            capabilities: health.capabilities,
        }
    } else {
        AgentStatus {
            available: false,
            state: "unavailable".to_owned(),
            label: "Agent not running".to_owned(),
            detail: health.detail,
            transport: health.transport,
            pid: None,
            uptime_ms: None,
            capabilities: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AgentClient;

    #[test]
    fn unavailable_agent_is_reported_without_false_connection() {
        let status = AgentClient::unavailable().status();
        assert!(!status.available);
        assert_eq!(status.state, "unavailable");
        assert!(status.pid.is_none());
    }
}
