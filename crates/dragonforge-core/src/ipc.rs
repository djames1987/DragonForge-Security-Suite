//! Fail-closed local IPC protocol and authorization foundations.
//!
//! This module intentionally does not implement a transport or credential
//! exchange. Platform transports must verify the peer before constructing an
//! authenticated PeerContext.

use crate::Component;
use crate::error::{CoreError, CoreResult, ErrorCode};

pub const CURRENT_PROTOCOL: ProtocolVersion = ProtocolVersion::new(1, 0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProtocolVersion {
    major: u16,
    minor: u16,
}

impl ProtocolVersion {
    #[must_use]
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    #[must_use]
    pub const fn major(self) -> u16 {
        self.major
    }

    #[must_use]
    pub const fn minor(self) -> u16 {
        self.minor
    }

    #[must_use]
    pub const fn is_compatible_with(self, peer: Self) -> bool {
        self.major == peer.major
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(u128);

impl RequestId {
    #[must_use]
    pub const fn new(value: u128) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u128 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuthenticationMechanism {
    OperatingSystemPeer,
    SessionCredential,
    MutualChallenge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PeerContext {
    component: Component,
    instance_id: u128,
    mechanism: Option<AuthenticationMechanism>,
}

impl PeerContext {
    #[must_use]
    pub const fn unauthenticated(component: Component, instance_id: u128) -> Self {
        Self {
            component,
            instance_id,
            mechanism: None,
        }
    }

    /// Construct only after the selected transport has verified the peer.
    #[must_use]
    pub const fn authenticated(
        component: Component,
        instance_id: u128,
        mechanism: AuthenticationMechanism,
    ) -> Self {
        Self {
            component,
            instance_id,
            mechanism: Some(mechanism),
        }
    }

    #[must_use]
    pub const fn component(self) -> Component {
        self.component
    }

    #[must_use]
    pub const fn instance_id(self) -> u128 {
        self.instance_id
    }

    #[must_use]
    pub const fn authentication_mechanism(self) -> Option<AuthenticationMechanism> {
        self.mechanism
    }

    #[must_use]
    pub const fn is_authenticated(self) -> bool {
        self.mechanism.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpcEnvelope<T> {
    pub protocol: ProtocolVersion,
    pub request_id: RequestId,
    pub source: Component,
    pub destination: Component,
    pub payload: T,
}

impl<T> IpcEnvelope<T> {
    #[must_use]
    pub const fn new(
        request_id: RequestId,
        source: Component,
        destination: Component,
        payload: T,
    ) -> Self {
        Self {
            protocol: CURRENT_PROTOCOL,
            request_id,
            source,
            destination,
            payload,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalIpcPolicy {
    destination: Component,
    allowed_callers: Vec<Component>,
}

impl LocalIpcPolicy {
    #[must_use]
    pub fn new(
        destination: Component,
        allowed_callers: impl IntoIterator<Item = Component>,
    ) -> Self {
        Self {
            destination,
            allowed_callers: allowed_callers.into_iter().collect(),
        }
    }

    pub fn authorize<T>(&self, peer: PeerContext, envelope: &IpcEnvelope<T>) -> CoreResult<()> {
        if !CURRENT_PROTOCOL.is_compatible_with(envelope.protocol) {
            return Err(CoreError::new_safe(
                ErrorCode::ProtocolMismatch,
                "IPC protocol major version is incompatible",
            ));
        }

        if envelope.destination != self.destination {
            return Err(CoreError::new_safe(
                ErrorCode::InvalidRequest,
                "IPC request was sent to the wrong destination",
            ));
        }

        if !peer.is_authenticated() {
            return Err(CoreError::new_safe(
                ErrorCode::AuthenticationRequired,
                "IPC peer authentication is required",
            ));
        }

        if peer.component() != envelope.source {
            return Err(CoreError::new_safe(
                ErrorCode::AuthorizationDenied,
                "IPC envelope source does not match the authenticated peer",
            ));
        }

        if !self.allowed_callers.contains(&peer.component()) {
            return Err(CoreError::new_safe(
                ErrorCode::AuthorizationDenied,
                "IPC caller is not authorized for this destination",
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AuthenticationMechanism, IpcEnvelope, LocalIpcPolicy, PeerContext, ProtocolVersion,
        RequestId,
    };
    use crate::Component;

    fn envelope() -> IpcEnvelope<&'static str> {
        IpcEnvelope::new(
            RequestId::new(42),
            Component::SecurityCenter,
            Component::Agent,
            "health",
        )
    }

    #[test]
    fn unauthenticated_peers_fail_closed() {
        let policy = LocalIpcPolicy::new(Component::Agent, [Component::SecurityCenter]);
        let peer = PeerContext::unauthenticated(Component::SecurityCenter, 1);
        assert!(policy.authorize(peer, &envelope()).is_err());
    }

    #[test]
    fn authenticated_allowed_peer_is_authorized() {
        let policy = LocalIpcPolicy::new(Component::Agent, [Component::SecurityCenter]);
        let peer = PeerContext::authenticated(
            Component::SecurityCenter,
            1,
            AuthenticationMechanism::OperatingSystemPeer,
        );
        assert!(policy.authorize(peer, &envelope()).is_ok());
    }

    #[test]
    fn spoofed_envelope_source_is_rejected() {
        let policy = LocalIpcPolicy::new(Component::Agent, [Component::SecurityCenter]);
        let peer = PeerContext::authenticated(
            Component::PasswordManager,
            2,
            AuthenticationMechanism::SessionCredential,
        );
        assert!(policy.authorize(peer, &envelope()).is_err());
    }

    #[test]
    fn authenticated_but_unlisted_caller_is_rejected() {
        let policy = LocalIpcPolicy::new(Component::Agent, [Component::SecurityCenter]);
        let peer = PeerContext::authenticated(
            Component::PasswordManager,
            2,
            AuthenticationMechanism::SessionCredential,
        );
        let request = IpcEnvelope::new(
            RequestId::new(43),
            Component::PasswordManager,
            Component::Agent,
            "health",
        );
        assert!(policy.authorize(peer, &request).is_err());
    }

    #[test]
    fn wrong_destination_is_rejected() {
        let policy = LocalIpcPolicy::new(Component::Agent, [Component::SecurityCenter]);
        let peer = PeerContext::authenticated(
            Component::SecurityCenter,
            1,
            AuthenticationMechanism::OperatingSystemPeer,
        );
        let request = IpcEnvelope::new(
            RequestId::new(44),
            Component::SecurityCenter,
            Component::PasswordManager,
            "health",
        );
        assert!(policy.authorize(peer, &request).is_err());
    }

    #[test]
    fn incompatible_protocol_major_is_rejected() {
        let policy = LocalIpcPolicy::new(Component::Agent, [Component::SecurityCenter]);
        let peer = PeerContext::authenticated(
            Component::SecurityCenter,
            1,
            AuthenticationMechanism::OperatingSystemPeer,
        );
        let mut request = envelope();
        request.protocol = ProtocolVersion::new(2, 0);
        assert!(policy.authorize(peer, &request).is_err());
    }
}
