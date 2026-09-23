# ADR 0010 — Privileged Service Runtime

## Status

Accepted for Phase 17.

## Context

Phase 16 established the policy contract for a future Windows privileged service. Phase 17 must implement that contract without expanding the product's privileged feature set or introducing a generic elevated execution surface.

## Decision

DragonForge will use a separate SCM-managed Windows service named `DragonForgePrivilegedService`.

The service:

- runs as the virtual account `NT SERVICE\DragonForgePrivilegedService`;
- uses a restricted service SID;
- exposes only a local named pipe with an explicit DACL;
- rejects remote pipe clients;
- treats the normal-user DragonForge Agent as the sole application caller;
- identifies the pipe client using Windows-reported process identity;
- requires the exact installed Agent executable path;
- requires valid Authenticode;
- pins the full X.500 publisher subject;
- applies freshness, replay, rate, message-size, and timeout controls;
- writes bounded redaction-safe audit records;
- dispatches only typed allow-listed commands.

The normal DragonForge installer remains per-user and non-elevated. Privileged-service installation is a separate explicit UAC-approved action and refuses unsigned binaries.

Phase 17 exposes only `health` and `describe-policy`. Every modeled privileged mutation capability remains disabled.

## Unsafe Windows FFI

The suite-wide unsafe-code prohibition remains the default. The service needs a small FFI boundary for APIs not adequately expressible through the safe runtime layers used here:

- named-pipe client PID;
- process image path;
- explicit security descriptor construction;
- Authenticode trust and signer certificate identity.

That exception is isolated to `services/dragonforge-privileged-service/src/windows.rs` and is verifier-enforced.

## Consequences

Future phases can add privileged capabilities only by extending the typed protocol and authorization policy. They may not add shell strings, arbitrary executable paths, arbitrary PowerShell/cmd invocation, or generic administrator commands.

A compromised normal-user process cannot become an accepted service caller merely by speaking the protocol: the service independently validates Windows peer identity, exact Agent path, Authenticode trust, and publisher identity.
