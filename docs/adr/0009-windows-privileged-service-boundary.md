# ADR 0009 — Windows Privileged Service Boundary

## Status

Accepted for Phase 16.

## Context

DragonForge currently has a normal-user per-user Agent. Future features may require narrowly privileged Windows operations. Adding those operations directly to the Agent would collapse the privilege boundary and turn a user-process compromise into privileged command execution.

## Decision

Privileged operations will live behind a separate Windows service boundary.

The service contract is:

- virtual service account identity: `NT SERVICE\DragonForgePrivilegedService`;
- restricted service SID and explicit named-pipe DACL;
- local named-pipe IPC only;
- only the installed DragonForge Agent is an application caller;
- OS peer-process identity verification is mandatory;
- exact Agent executable path verification is mandatory;
- valid Authenticode and exact DragonForge publisher subject are mandatory;
- bounded protocol messages, freshness, and replay protection are mandatory;
- commands are typed and explicitly allow-listed;
- capabilities are deny-by-default;
- generic shell/exec/run behavior is prohibited.

Phase 16 authorizes no privileged capability.

## Consequences

Phase 17 must implement the service against this contract rather than inventing a broader interface. Every new privileged capability requires its own schema, authorization rule, validation, test coverage, and audit semantics.

The design accepts some implementation complexity in exchange for isolating privileged code and preventing the Agent from becoming a general-purpose elevated execution surface.
