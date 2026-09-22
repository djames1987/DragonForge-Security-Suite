# Phase 12.5 — Crash Handling & Diagnostics

## Status

**Verified Complete**

Phase 12.5 adds a shared, metadata-focused diagnostics foundation for DragonForge.

## Delivered

- shared component-scoped rotating logs under each component's DragonForge data directory;
- default 1 MiB active log size with three bounded rotated backups;
- secret-aware diagnostic sanitization for passwords, tokens, credentials, vault references, OTP seeds, recovery material, private keys, sensitive paths/content, and authorization headers;
- crash-safe panic hooks that intentionally do not persist panic payloads;
- Security Center panic-hook adoption;
- DragonForge Agent panic-hook adoption;
- per-component last-failure.txt records using public bounded summaries only;
- Security Center visibility for recorded failure state across all ten suite components;
- diagnostics schema v2;
- per-component application version metadata;
- WebView2 runtime version detection on Windows, represented as a version or null;
- redaction-safe support bundle generation from Security Center;
- support bundle component-log metadata plus a sanitized Security Center log tail;
- automated redaction, rotation, and failure-record tests.

## Crash records

The panic hook never serializes the Rust panic payload, user input, stack-local values, or arbitrary filesystem paths. It records only schema version, timestamp, a fixed failure code, and a fixed component-specific public summary.

## Log layout and retention

Each component can use the shared ComponentLogger. The default active log threshold is 1 MiB with three backups. Logs are namespaced under the component's DragonForge data directory.

## Support bundle

Security Center can create a redaction-safe JSON support bundle from About. It contains diagnostics schema v2, suite/build/platform metadata, WebView2 version when detected, per-component suite versions and states, Agent status/capabilities, per-component failure status, per-component log byte counts, and a bounded sanitized Security Center log tail.

The bundle does not recursively copy component data directories or user files.

## Privacy

Diagnostic identifiers remain controlled by the existing opt-in setting. Agent PID remains omitted unless the user enables diagnostic identifiers. Support bundles and diagnostic reports exclude intentional collection of passwords, sync/admin tokens, OTP seeds, recovery material, private keys, vault contents, sensitive user-file contents, and arbitrary user paths.

## Verification

Run:

    powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase12.5-crash-diagnostics-tests.ps1

The verifier writes a timestamped log and .sha256 sidecar under test-logs\.

## Verified Windows result

Phase 12.5 completed authoritative Windows verification on `DRACO`.

- Machine: `DRACO`
- Windows: `Microsoft Windows NT 10.0.26200.0`
- Result: **PASS**
- Verified log: `dragonforge-phase12.5-crash-diagnostics-20260922-121347.log`
- Log SHA-256: `6BAD86F67AC0DF2233A0101BB1AD9812D98418C9DF6C813376FAB60D9637B80C`

The passing run covered rustfmt, targeted compile checks, strict Clippy with `-D warnings`, all 9 Agent tests, all 21 Core tests, all 24 Security Center tests, JavaScript syntax validation, and required Phase 12.5 diagnostics artifacts.
