# Phase 17 — DragonForge Privileged Service

**Status: Verified Complete**

Phase 17 implements the separately reviewed Phase 16 Windows privilege boundary as a real Windows Service Control Manager (SCM) service. The phase is intentionally limited to the privileged control plane itself; it does not add firewall mutation, quarantine, protected process termination, protected registry remediation, or other product features.

## Delivered

- Windows SCM service executable: `dragonforge-privileged-service.exe`;
- fixed service identity `DragonForgePrivilegedService`;
- virtual service account `NT SERVICE\DragonForgePrivilegedService`;
- restricted service SID configuration;
- protected SCM service DACL;
- explicit local named-pipe DACL and remote-client rejection;
- named-pipe client PID verification through Windows;
- exact installed `dragonforge-agent.exe` path verification;
- WinVerifyTrust Authenticode validation;
- full X.500 signer-subject pinning;
- request protocol versioning, freshness, nonce replay protection, per-process request quotas, bounded message sizes, and I/O timeouts;
- bounded JSONL audit records and size-based rotation;
- protected service configuration under `%ProgramData%\DragonForge\Security\privileged-service`;
- Agent client commands for privileged-service health and policy inspection;
- signed-install requirement for both the Agent and privileged-service executable;
- explicit Administrator/UAC install and remove workflows;
- optional installer task that preserves the suite's normal per-user, non-elevated installation path;
- zero privileged mutation capabilities enabled in Phase 17.

## Service command surface

The service accepts exactly two commands:

- `health`;
- `describe-policy`.

Neither command performs a privileged system mutation.

Generic `exec`, `shell`, `cmd`, PowerShell, arbitrary executable launch, arbitrary registry mutation, arbitrary firewall mutation, and similar generic command paths remain prohibited.

## Caller authentication and authorization

A request is considered for dispatch only after the service:

1. accepts a local named-pipe connection with remote clients rejected;
2. obtains the peer PID from the pipe using `GetNamedPipeClientProcessId`;
3. obtains the peer executable image path with limited process-query access;
4. verifies that path is the exact sibling `dragonforge-agent.exe`;
5. validates the Agent's Authenticode signature using `WinVerifyTrust`;
6. extracts the signing certificate's full X.500 subject;
7. compares that subject with the publisher pinned in protected service configuration;
8. validates protocol version, timestamp freshness, nonce shape and replay state;
9. applies the per-process request quota;
10. authorizes the fixed command through the Phase 16 boundary policy.

Authentication does not grant a privileged capability. `CapabilityPolicy::phase17_allows` remains deny-by-default for every modeled privileged capability.

## Service installation

The main DragonForge installer remains a normal per-user installer. The privileged service is optional and requires a separate explicit UAC approval.

The install flow refuses to register the service unless both:

- `dragonforge-agent.exe`; and
- `dragonforge-privileged-service.exe`

have valid Authenticode signatures from the same publisher.

The UAC installer copies the signed privileged-service executable into a protected `%ProgramFiles%\DragonForge Security Suite\Privileged Service` directory before SCM registration. It stores the exact signed Agent path and exact publisher certificate subject in protected service configuration, then applies restricted ACLs before starting the service.

Unsigned development builds therefore cannot silently become the privileged service.

## Protected configuration

Configuration lives at:

`%ProgramData%\DragonForge\Security\privileged-service\service-config.json`

It contains only bounded operational policy:

- schema version;
- exact expected Agent executable path;
- expected publisher subject;
- maximum requests per minute;
- maximum audit file size.

It contains no user password, vault key, recovery secret, signing private key, or generic command configuration.

## Audit records

Audit records are redaction-safe JSONL and contain only:

- timestamp;
- event class;
- client PID when relevant;
- fixed action name when relevant;
- bounded outcome label.

Request payloads, vault contents, tokens, certificate private material, and arbitrary caller-provided strings are not written to the audit.

The audit file is size-bounded and rotated.

## Abuse resistance

Phase 17 includes:

- 16 KiB maximum wire messages;
- two-second request and response timeouts;
- 60-second freshness window;
- 128-bit request nonces;
- bounded nonce replay cache;
- per-client-PID rate limits;
- bounded quota identity state;
- maximum four pipe instances;
- remote named-pipe clients rejected.

## Unsafe-code exception

The DragonForge workspace normally forbids unsafe Rust. Phase 17 introduces one narrowly reviewed exception because Windows peer-process, DACL, and WinTrust APIs require FFI.

Unsafe code is allowed only in:

`services/dragonforge-privileged-service/src/windows.rs`

The Phase 17 verifier checks that the rest of the privileged-service source and the rest of the suite do not gain a new unsafe-code surface.

## Explicit non-goals

Phase 17 does not implement:

- Windows Firewall changes;
- process termination;
- file quarantine;
- protected registry remediation;
- system-integrity remediation;
- arbitrary elevated command execution;
- a general-purpose administrative shell.

Those capabilities require separately scoped phases and typed authorization rules.

## Verification

Run on Windows:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase17-privileged-service-tests.ps1
```

This verifier does not weaken the signing boundary or install an unsigned development build as a service. Full installed-service acceptance is performed with `scripts\test-privileged-service.ps1` against a signed build after explicit Administrator installation.


## Authoritative local verification

Phase 17 was verified on 2026-09-23 on DRACO (Microsoft Windows NT 10.0.26200.0) using Windows PowerShell 5.1.26100.9444 with `scripts/run-phase17-privileged-service-tests.ps1`.

The verifier passed formatting, locked dependency metadata, full workspace checking, strict Clippy, the complete workspace and doc-test suite, the Phase 17 privileged-service security tests, PowerShell syntax validation, and the full Windows application build including `dragonforge-privileged-service.exe`.

Final verifier marker:

`PHASE 17 DRAGONFORGE PRIVILEGED SERVICE VERIFICATION: PASS`

Authoritative log SHA-256:

`05E32B87809B5BFB34E2A9AF0E7581428528EB180BFD2748894F481C6CCE0433`
