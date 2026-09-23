use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Read, Write};

use dragonforge_windows_boundary::{
    FirewallAction, FirewallApplicationIdentity, FirewallMutationRequest, FirewallMutationResult,
    MAX_MESSAGE_BYTES, PIPE_NAME, PROTOCOL_MAJOR, PROTOCOL_MINOR, PrivilegedPolicyDescription,
    PrivilegedRequest, PrivilegedResponse, PrivilegedServiceHealth,
};
use rand_core::{OsRng, RngCore};

use crate::error::{AgentError, Result};

#[derive(Debug, Clone, Copy, Default)]
pub struct PrivilegedServiceClient;

impl PrivilegedServiceClient {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    pub fn health(&self) -> Result<PrivilegedServiceHealth> {
        self.request("health", None)?
            .health
            .ok_or(AgentError::Protocol(
                "privileged service health response is missing",
            ))
    }

    pub fn describe_policy(&self) -> Result<PrivilegedPolicyDescription> {
        self.request("describe-policy", None)?
            .policy
            .ok_or(AgentError::Protocol(
                "privileged service policy response is missing",
            ))
    }

    pub fn firewall_status(
        &self,
        identity: FirewallApplicationIdentity,
    ) -> Result<FirewallMutationResult> {
        self.firewall_request("firewall-status", identity, FirewallAction::Block, None)
    }

    pub fn firewall_apply(
        &self,
        identity: FirewallApplicationIdentity,
        action: FirewallAction,
    ) -> Result<FirewallMutationResult> {
        self.firewall_request("firewall-apply", identity, action, None)
    }

    pub fn firewall_remove(
        &self,
        identity: FirewallApplicationIdentity,
    ) -> Result<FirewallMutationResult> {
        self.firewall_request("firewall-remove", identity, FirewallAction::Block, None)
    }

    pub fn firewall_rollback(
        &self,
        identity: FirewallApplicationIdentity,
        rollback_token: String,
    ) -> Result<FirewallMutationResult> {
        self.firewall_request(
            "firewall-rollback",
            identity,
            FirewallAction::Block,
            Some(rollback_token),
        )
    }

    fn firewall_request(
        &self,
        action_name: &str,
        identity: FirewallApplicationIdentity,
        action: FirewallAction,
        rollback_token: Option<String>,
    ) -> Result<FirewallMutationResult> {
        self.request(
            action_name,
            Some(FirewallMutationRequest {
                identity,
                action,
                rollback_token,
            }),
        )?
        .firewall
        .ok_or(AgentError::Protocol(
            "privileged firewall response is missing",
        ))
    }

    fn request(
        &self,
        action: &str,
        firewall: Option<FirewallMutationRequest>,
    ) -> Result<PrivilegedResponse> {
        let mut pipe = OpenOptions::new()
            .read(true)
            .write(true)
            .open(PIPE_NAME)
            .map_err(|_| AgentError::Unavailable("privileged service is unavailable"))?;

        let request = PrivilegedRequest {
            protocol_major: PROTOCOL_MAJOR,
            protocol_minor: PROTOCOL_MINOR,
            request_id: random_request_id(),
            action: action.to_owned(),
            timestamp_ms: now_ms(),
            nonce_hex: random_nonce_hex(),
            firewall,
        };

        let mut encoded = serde_json::to_vec(&request)
            .map_err(|_| AgentError::Protocol("privileged service request serialization failed"))?;
        if encoded.len() + 1 > MAX_MESSAGE_BYTES {
            return Err(AgentError::Protocol(
                "privileged service request is oversized",
            ));
        }
        encoded.push(b'\n');
        pipe.write_all(&encoded)
            .and_then(|_| pipe.flush())
            .map_err(|_| AgentError::Io("privileged service request could not be written"))?;

        let mut reader = BufReader::new(pipe);
        let mut line = String::new();
        let count = reader
            .by_ref()
            .take(MAX_MESSAGE_BYTES as u64)
            .read_line(&mut line)
            .map_err(|_| AgentError::Io("privileged service response could not be read"))?;
        if count == 0 || count >= MAX_MESSAGE_BYTES {
            return Err(AgentError::Protocol(
                "privileged service response is missing or oversized",
            ));
        }

        let response: PrivilegedResponse = serde_json::from_str(line.trim_end())
            .map_err(|_| AgentError::Protocol("privileged service response is malformed"))?;
        if response.request_id != request.request_id {
            return Err(AgentError::Protocol(
                "privileged service response request ID does not match",
            ));
        }
        if !response.ok {
            return Err(AgentError::Unavailable(
                "privileged service rejected the request",
            ));
        }
        Ok(response)
    }
}

fn random_request_id() -> u64 {
    let mut bytes = [0_u8; 8];
    OsRng.fill_bytes(&mut bytes);
    u64::from_le_bytes(bytes)
}

fn random_nonce_hex() -> String {
    let mut bytes = [0_u8; 16];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| {
            duration.as_millis().min(u128::from(u64::MAX)) as u64
        })
}

#[cfg(test)]
mod tests {
    use super::random_nonce_hex;

    #[test]
    fn privileged_nonce_is_128_bit_hex() {
        let nonce = random_nonce_hex();
        assert_eq!(nonce.len(), 32);
        assert!(nonce.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }
}
