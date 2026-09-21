use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use dragonforge_core::{
    AuthenticationMechanism, Component, IpcEnvelope, LocalIpcPolicy, PeerContext, ProtocolVersion,
    RequestId,
};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::error::{AgentError, Result};

pub const AGENT_PROTOCOL_MAJOR: u16 = 1;
pub const AGENT_PROTOCOL_MINOR: u16 = 0;
pub const MAX_CLOCK_SKEW_MS: u64 = 60_000;
pub const MAX_WIRE_BYTES: usize = 16 * 1024;
pub const SESSION_KEY_BYTES: usize = 32;
pub const NONCE_BYTES: usize = 16;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RuntimeDescriptor {
    pub format_version: u16,
    pub protocol_major: u16,
    pub protocol_minor: u16,
    pub port: u16,
    pub pid: u32,
    pub started_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RequestWire {
    pub protocol_major: u16,
    pub protocol_minor: u16,
    pub request_id: u64,
    pub source: String,
    pub action: String,
    pub timestamp_ms: u64,
    pub nonce_b64: String,
    pub auth_tag_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HealthWire {
    pub state: String,
    pub pid: u32,
    pub uptime_ms: u64,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ResponseWire {
    pub request_id: u64,
    pub ok: bool,
    pub health: Option<HealthWire>,
    pub error: Option<String>,
    pub auth_tag_hex: String,
}

pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            duration.as_millis().min(u128::from(u64::MAX)) as u64
        })
}

pub(crate) fn request_message(request: &RequestWire) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}|{}",
        request.protocol_major,
        request.protocol_minor,
        request.request_id,
        request.source,
        request.action,
        request.timestamp_ms,
        request.nonce_b64
    )
}

pub(crate) fn response_message(response: &ResponseWire) -> String {
    let health = response.health.as_ref().map_or_else(
        || "-".to_owned(),
        |health| {
            format!(
                "{}:{}:{}:{}",
                health.state,
                health.pid,
                health.uptime_ms,
                health.capabilities.join(",")
            )
        },
    );
    format!(
        "{}|{}|{}|{}",
        response.request_id,
        response.ok,
        health,
        response.error.as_deref().unwrap_or("-")
    )
}

pub(crate) fn sign_hex(key: &[u8], message: &str) -> Result<String> {
    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|_| AgentError::Authentication("agent session credential is invalid"))?;
    mac.update(message.as_bytes());
    Ok(mac
        .finalize()
        .into_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

pub(crate) fn verify_hex(key: &[u8], message: &str, tag_hex: &str) -> Result<()> {
    if tag_hex.len() != 64 {
        return Err(AgentError::Authentication(
            "agent authentication tag is malformed",
        ));
    }
    let mut tag = [0_u8; 32];
    for (index, chunk) in tag_hex.as_bytes().chunks_exact(2).enumerate() {
        let pair = std::str::from_utf8(chunk)
            .map_err(|_| AgentError::Authentication("agent authentication tag is malformed"))?;
        tag[index] = u8::from_str_radix(pair, 16)
            .map_err(|_| AgentError::Authentication("agent authentication tag is malformed"))?;
    }
    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|_| AgentError::Authentication("agent session credential is invalid"))?;
    mac.update(message.as_bytes());
    mac.verify_slice(&tag)
        .map_err(|_| AgentError::Authentication("agent request authentication failed"))
}

pub(crate) fn decode_nonce(value: &str) -> Result<Vec<u8>> {
    let nonce = BASE64
        .decode(value)
        .map_err(|_| AgentError::Protocol("agent request nonce is malformed"))?;
    if nonce.len() != NONCE_BYTES {
        return Err(AgentError::Protocol("agent request nonce length is invalid"));
    }
    Ok(nonce)
}

pub(crate) fn authorize_request(request: &RequestWire) -> Result<()> {
    if request.protocol_major != AGENT_PROTOCOL_MAJOR {
        return Err(AgentError::Protocol(
            "agent protocol major version is incompatible",
        ));
    }
    if request.source != Component::SecurityCenter.as_str() {
        return Err(AgentError::Authorization(
            "agent request source is not authorized",
        ));
    }

    let envelope = IpcEnvelope {
        protocol: ProtocolVersion::new(request.protocol_major, request.protocol_minor),
        request_id: RequestId::new(u128::from(request.request_id)),
        source: Component::SecurityCenter,
        destination: Component::Agent,
        payload: request.action.as_str(),
    };
    let peer = PeerContext::authenticated(
        Component::SecurityCenter,
        1,
        AuthenticationMechanism::SessionCredential,
    );
    LocalIpcPolicy::new(Component::Agent, [Component::SecurityCenter])
        .authorize(peer, &envelope)
        .map_err(|_| AgentError::Authorization("agent IPC policy denied the request"))
}
