# ADR-0002: Security Center Native Boundaries

Status: Accepted  
Date: 2026-09-21

## Context

Phase 3 introduces a webview-based desktop Security Center that will eventually orchestrate multiple security products and a privileged background Agent.

The dashboard must not become a general-purpose privileged execution surface.

## Decision

Use Tauri as the Security Center desktop shell and keep native capabilities behind a narrow Rust command surface.

Phase 3 rules:

1. UI JavaScript cannot directly access arbitrary filesystem/process APIs.
2. Settings are persisted by native Rust code under a component-specific SuitePaths directory.
3. Activity and logging use safe/redaction-oriented shared primitives.
4. Password Manager orchestration may launch only the exact sibling executable beside Security Center.
5. The dashboard must explicitly report the Agent as unavailable until a real authenticated service/transport exists.
6. Agent authentication will not be inferred from request payload identity.
7. General arbitrary command execution is not part of Security Center.

## Security implications

The sibling-only launcher reduces path-hijacking and user-input execution risk compared with searching PATH or accepting arbitrary executable paths.

Tauri commands remain privileged native boundaries and must validate inputs.

The current settings file is not a secret store and must remain limited to preferences.

The current activity store is operational UI state, not a tamper-resistant audit log.

## Alternatives considered

### Browser-only local dashboard

Rejected. Future suite orchestration requires controlled native integration.

### Electron

Not selected. The suite is already Rust-centric and Tauri keeps the native boundary in Rust with a smaller architecture fit.

### Implement Agent transport now

Deferred. Phase 3 should not create a privileged service prematurely. The Phase 2 IPC authorization model exists so transport design can be reviewed separately.

### Generic executable launcher

Rejected. It would create unnecessary local execution risk.

## Consequences

- Security Center can grow into the main suite dashboard without becoming the Agent itself.
- Future app-launch actions should use explicit allowlisted products and trusted install metadata.
- Privileged enforcement belongs in DragonForge Agent, not the webview.
- File Vault and other future products can integrate through registry/health/event contracts established here.
