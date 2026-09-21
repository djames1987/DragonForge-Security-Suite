# ADR-0001: Shared Foundation Boundaries

Status: Accepted  
Date: 2026-09-20

## Context

After Password Manager migration, DragonForge Security Suite needs common types for configuration, diagnostics, platform paths, events, and future local IPC.

Moving Password Manager cryptography, vault, sync, or recovery behavior into a generic crate during this phase would increase regression and ownership risk.

## Decision

Keep dragonforge-core as a small, dependency-light, non-cryptographic foundation.

Phase 2 places the following transport/domain-neutral primitives in dragonforge-core:
- component identifiers;
- configuration keys and resolution source metadata;
- safe error categories;
- event metadata;
- platform/path discovery;
- redaction/log policy;
- IPC envelope, authentication metadata, and authorization policy.

Password Manager cryptography, encrypted formats, key derivation, sync, recovery, and credential storage remain in their existing product-owned modules/crates.

The IPC layer remains transport-neutral. A transport must independently authenticate the peer before constructing an authenticated PeerContext.

If one of these foundation modules becomes a substantial subsystem or acquires security-sensitive dependencies, split it into a dedicated crate through a new ADR rather than allowing dragonforge-core to become a monolith.

## Security implications

Benefits:
- avoids duplicate component IDs and protocol metadata;
- provides fail-closed IPC authorization rules before privileged IPC exists;
- makes redaction defaults reusable;
- keeps product-proven crypto ownership clear;
- avoids hidden filesystem mutation during path discovery.

Risks:
- authenticated PeerContext is only trustworthy if a transport constructs it after real verification;
- Secret<T> prevents formatting disclosure but does not zeroize memory;
- safe error/event strings can still leak secrets if callers violate their contract.

These limitations are documented in code and the security model.

## Alternatives considered

### Move Password Manager crypto into dragonforge-core

Rejected. It would mix non-cryptographic orchestration primitives with security-sensitive product code and broaden the regression surface.

### Create five new shared crates immediately

Deferred. The current primitives are small and dependency-light. Separate crates can be introduced when real subsystem boundaries justify the maintenance and API cost.

### Implement Windows named-pipe IPC during Phase 2

Deferred to Security Center/Agent development. Phase 2 establishes protocol and authorization invariants first.

## Consequences

- Phase 3 Security Center can depend on stable shared types.
- Future apps can use the same config/event/path/redaction contracts.
- Agent IPC transport work must document how peer authentication maps into PeerContext.
- Security-sensitive Password Manager behavior remains independently reviewable.
