use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use rand_core::{OsRng, RngCore};
use serde::Serialize;

use crate::error::{AgentError, Result};
use crate::paths::AgentPaths;
use crate::protocol::{
    AGENT_PROTOCOL_MAJOR, AGENT_PROTOCOL_MINOR, MAX_WIRE_BYTES, NONCE_BYTES, RequestWire,
    ResponseWire, RuntimeDescriptor, now_ms, request_message, response_message, sign_hex,
    verify_hex,
};

const CONNECT_TIMEOUT: Duration = Duration::from_millis(450);
const IO_TIMEOUT: Duration = Duration::from_millis(750);

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgentHealth {
    pub available: bool,
    pub state: String,
    pub detail: String,
    pub transport: String,
    pub pid: Option<u32>,
    pub uptime_ms: Option<u64>,
    pub capabilities: Vec<String>,
}

impl AgentHealth {
    #[must_use]
    pub fn unavailable(detail: impl Into<String>) -> Self {
        Self {
            available: false,
            state: "unavailable".to_owned(),
            detail: detail.into(),
            transport: "loopback-tcp+hmac-sha256".to_owned(),
            pid: None,
            uptime_ms: None,
            capabilities: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentClient {
    paths: AgentPaths,
}

impl AgentClient {
    #[must_use]
    pub fn from_paths(paths: AgentPaths) -> Self {
        Self { paths }
    }

    pub fn discover() -> Result<Self> {
        Ok(Self::from_paths(AgentPaths::discover()?))
    }

    pub fn health(&self) -> Result<AgentHealth> {
        let descriptor_bytes = fs::read(self.paths.runtime_file())
            .map_err(|_| AgentError::Unavailable("agent runtime descriptor is unavailable"))?;
        if descriptor_bytes.len() > MAX_WIRE_BYTES {
            return Err(AgentError::Protocol("agent runtime descriptor is oversized"));
        }
        let descriptor: RuntimeDescriptor = serde_json::from_slice(&descriptor_bytes)
            .map_err(|_| AgentError::Protocol("agent runtime descriptor is malformed"))?;
        if descriptor.format_version != 1 || descriptor.protocol_major != AGENT_PROTOCOL_MAJOR {
            return Err(AgentError::Protocol("agent runtime descriptor is incompatible"));
        }

        let key = read_session_key(&self.paths)?;
        let address = SocketAddrV4::new(Ipv4Addr::LOCALHOST, descriptor.port);
        let mut stream = TcpStream::connect_timeout(&address.into(), CONNECT_TIMEOUT)
            .map_err(|_| AgentError::Unavailable("DragonForge Agent is not responding"))?;
        stream
            .set_read_timeout(Some(IO_TIMEOUT))
            .map_err(|_| AgentError::Io("agent read timeout could not be configured"))?;
        stream
            .set_write_timeout(Some(IO_TIMEOUT))
            .map_err(|_| AgentError::Io("agent write timeout could not be configured"))?;

        let mut nonce = [0_u8; NONCE_BYTES];
        OsRng.fill_bytes(&mut nonce);
        let mut request = RequestWire {
            protocol_major: AGENT_PROTOCOL_MAJOR,
            protocol_minor: AGENT_PROTOCOL_MINOR,
            request_id: random_request_id(),
            source: "security-center".to_owned(),
            action: "health".to_owned(),
            timestamp_ms: now_ms(),
            nonce_b64: BASE64.encode(nonce),
            auth_tag_hex: String::new(),
        };
        request.auth_tag_hex = sign_hex(&key, &request_message(&request))?;

        let encoded = serde_json::to_vec(&request)
            .map_err(|_| AgentError::Protocol("agent request could not be serialized"))?;
        if encoded.len() + 1 > MAX_WIRE_BYTES {
            return Err(AgentError::Protocol("agent request is oversized"));
        }
        stream
            .write_all(&encoded)
            .and_then(|_| stream.write_all(b"\n"))
            .map_err(|_| AgentError::Io("agent request could not be written"))?;

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        let count = reader
            .by_ref()
            .take(MAX_WIRE_BYTES as u64)
            .read_line(&mut line)
            .map_err(|_| AgentError::Io("agent response could not be read"))?;
        if count == 0 || count >= MAX_WIRE_BYTES {
            return Err(AgentError::Protocol("agent response is missing or oversized"));
        }

        let response: ResponseWire = serde_json::from_str(line.trim_end())
            .map_err(|_| AgentError::Protocol("agent response is malformed"))?;
        if response.request_id != request.request_id {
            return Err(AgentError::Protocol("agent response request ID does not match"));
        }
        verify_hex(&key, &response_message(&response), &response.auth_tag_hex)?;
        if !response.ok {
            return Err(AgentError::Unavailable(
                "agent returned an authenticated failure response",
            ));
        }
        let health = response
            .health
            .ok_or(AgentError::Protocol("agent health response is missing"))?;

        Ok(AgentHealth {
            available: true,
            state: health.state,
            detail: "Authenticated local agent session is healthy.".to_owned(),
            transport: "loopback-tcp+hmac-sha256".to_owned(),
            pid: Some(health.pid),
            uptime_ms: Some(health.uptime_ms),
            capabilities: health.capabilities,
        })
    }
}

fn read_session_key(paths: &AgentPaths) -> Result<Vec<u8>> {
    let value = fs::read_to_string(paths.credential_file())
        .map_err(|_| AgentError::Unavailable("agent session credential is unavailable"))?;
    let key = BASE64
        .decode(value.trim())
        .map_err(|_| AgentError::Authentication("agent session credential is malformed"))?;
    if key.len() != 32 {
        return Err(AgentError::Authentication(
            "agent session credential length is invalid",
        ));
    }
    Ok(key)
}

fn random_request_id() -> u64 {
    let mut bytes = [0_u8; 8];
    OsRng.fill_bytes(&mut bytes);
    u64::from_le_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{AgentClient, AgentHealth};
    use crate::AgentPaths;

    #[test]
    fn unavailable_health_model_is_explicit() {
        let health = AgentHealth::unavailable("not running");
        assert!(!health.available);
        assert_eq!(health.state, "unavailable");
    }

    #[test]
    fn missing_runtime_descriptor_fails_closed() {
        let dir = tempdir().expect("tempdir");
        let client = AgentClient::from_paths(AgentPaths::from_root(dir.path()));
        assert!(client.health().is_err());
    }
}
