# Phase 2 — Shared Foundation

## Status

**Complete**

Phase 2 establishes the first suite-wide, non-cryptographic foundation that future DragonForge applications can depend on without pulling Password Manager-specific security behavior into generic code.

## Goals completed

The shared foundation now provides:

- validated configuration-key primitives;
- configuration source metadata;
- stable cross-component error codes;
- event severity, category, and record types;
- operating-system and standard suite-path discovery;
- explicit log redaction policy;
- formatting-safe secret wrappers;
- versioned IPC request envelopes;
- transport-authentication metadata;
- fail-closed local IPC caller authorization.

## dragonforge-core modules

### config

Provides:
- ConfigKey validation;
- stable component-scoped environment-variable names;
- ConfigSource metadata;
- ResolvedConfig<T> values that retain provenance.

Configuration keys are intentionally machine-readable and constrained to a small character set.

### error

Provides:
- ErrorCode for stable machine-readable categories;
- CoreError;
- CoreResult<T>.

CoreError messages are explicitly designated safe for logs/UI. Callers must not place secret material in those messages.

### event

Provides:
- Severity;
- EventKind;
- EventRecord.

Event summaries are explicitly named safe_summary to reinforce the logging contract.

### platform

Provides:
- Platform identification;
- SuitePaths;
- conventional per-user config/data/cache roots for Windows, Linux, and macOS.

Path discovery does not create directories or change permissions.

### redaction

Provides:
- Secret<T>, whose Debug and Display implementations emit only [REDACTED];
- LogPolicy, which defaults to hiding identifiers and sanitizes/bounds public fields.

Secret<T> is a formatting guard, not a secure-memory or zeroization primitive.

### ipc

Provides:
- ProtocolVersion and CURRENT_PROTOCOL;
- RequestId;
- IpcEnvelope<T>;
- AuthenticationMechanism;
- PeerContext;
- LocalIpcPolicy.

The policy rejects unauthenticated peers, source-identity mismatches, wrong destinations, unauthorized callers, and incompatible protocol major versions.

The IPC foundation deliberately does not:
- create named pipes, Unix sockets, TCP sockets, or other transports;
- generate or persist authentication credentials;
- perform cryptography;
- infer peer identity from untrusted payload fields.

A future transport must independently verify the peer and only then construct an authenticated PeerContext.

## Security Center adoption

Security Center now consumes shared Component, EventRecord, Platform, and SuitePaths primitives. This proves the foundation can be used by an application without coupling it to Password Manager internals.

## Password Manager boundary

Phase 2 does not move or redesign:
- dragonforge-crypto;
- dragonforge-vault;
- Password Manager key derivation;
- vault formats;
- sync protocol;
- recovery flow;
- Windows Credential Manager storage.

Those remain product-proven security-sensitive components and require dedicated review before any wider reuse.

## Test coverage

dragonforge-core includes unit coverage for:
- stable component and error identifiers;
- configuration key acceptance/rejection;
- environment-variable mapping;
- configuration-source retention;
- secret Debug/Display redaction;
- conservative log-identifier handling;
- public-field sanitization and truncation;
- event metadata;
- platform namespacing;
- unauthenticated IPC rejection;
- authenticated allowed-call acceptance;
- spoofed source rejection;
- incompatible IPC protocol rejection.

Suite Foundation CI runs formatting, cargo check, Clippy with warnings denied, and tests for dragonforge-core and Security Center when the relevant paths change.

## Phase 3 handoff

Phase 3 can build the Security Center on top of these primitives.

Recommended first Phase 3 slices:
1. application shell and navigation;
2. component registry/status model;
3. event/health aggregation;
4. authenticated agent-IPC transport design;
5. settings backed by ConfigKey and SuitePaths;
6. shared logging bootstrap using LogPolicy.

Phase 3 should not add privileged agent behavior until the IPC transport and peer-authentication mechanism have their own review and tests.
