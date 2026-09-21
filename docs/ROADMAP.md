# DragonForge Security Suite Roadmap

This roadmap is directional. Security and verification take priority over phase numbers.

## Phase 0 — Suite architecture and migration readiness
**Status: Complete**

Delivered:
- repository/workspace structure;
- documented trust boundaries;
- baseline CI and security policy;
- Password Manager migration design;
- suite branding asset structure.

## Phase 1 — Password Manager migration
**Status: Complete**

Delivered:
- tested Password Manager desktop application migrated to `apps/password-manager/`;
- sync server migrated to `services/password-manager-sync/`;
- browser extension migrated to `extensions/password-manager-browser/`;
- crypto and vault crates migrated without intentional redesign;
- original regression suite retained;
- Windows credential-store test isolation stabilized;
- full Phase 11 post-migration verification passed;
- PR #2 merged into `main`.

## Phase 1.1 — Post-Migration Baseline Cleanup
**Status: Complete**

Goals:
- make repository documentation match the migrated state;
- record the frozen source and verified suite baseline;
- establish this repository as the canonical Password Manager development home;
- separate suite-foundation CI concerns from Password Manager regression CI;
- leave a clean baseline for Phase 2.

See [PHASE_1_1_BASELINE.md](PHASE_1_1_BASELINE.md).

## Phase 2 — Shared foundation
**Status: Complete**

Delivered:
- validated component-scoped configuration keys and source metadata;
- stable shared error codes and redaction-safe error messaging contract;
- shared event severity/kind/record primitives;
- cross-platform suite path discovery without implicit filesystem mutation;
- secret-formatting redaction wrapper and conservative log policy;
- versioned local IPC envelopes, authenticated peer context, and fail-closed caller policy;
- Security Center adoption of shared platform/event primitives;
- unit coverage for validation, redaction, event metadata, platform namespacing, and IPC authorization failures;
- ADR and Phase 2 foundation documentation.

No Password Manager cryptography, vault format, sync protocol, recovery flow, or credential-storage implementation was moved into the shared core.

See [PHASE_2_SHARED_FOUNDATION.md](PHASE_2_SHARED_FOUNDATION.md).

## Phase 3 — Security Center
**Status: Complete**

Delivered:
- Tauri desktop application shell with DragonForge suite styling;
- overview, components, activity, settings, and about navigation;
- suite component registry with accurate current/future states;
- aggregate suite health model without treating planned components as failures;
- bounded redaction-safe in-memory activity history;
- persistent versioned local settings;
- safe local logging bootstrap;
- explicit future-agent status and Phase 2 IPC-policy boundary;
- strict co-located Password Manager launch orchestration;
- Security Center-specific CI and local Phase 3 verification script;
- Phase 3 architecture/security documentation and ADR.

The Phase 3 UI does not claim that the future privileged DragonForge Agent is installed or connected.

See [PHASE_3_SECURITY_CENTER.md](PHASE_3_SECURITY_CENTER.md).

## Phase 4 — File Vault
**Status: Complete**

Delivered:
- dedicated File Vault encrypted-container crate and Tauri desktop app;
- AES-256-GCM authenticated encryption;
- Argon2id password derivation with versioned KDF parameters;
- encrypted filenames, directory structure, and file contents;
- tamper/wrong-password rejection;
- path traversal and symbolic-link rejection;
- bounded entries, path lengths, and aggregate file size;
- no-overwrite container creation and extraction;
- temporary-directory extraction with cleanup on failure;
- Security Center integration and strict sibling-app launch;
- Phase 4 verification script, documentation, and CI coverage.

See [PHASE_4_FILE_VAULT.md](PHASE_4_FILE_VAULT.md).

## Phase 5 — Authenticator
**Status: Verified Complete**

Delivered:
- dedicated Authenticator engine crate and Tauri desktop app;
- RFC-compatible TOTP and HOTP generation;
- SHA-1/SHA-256/SHA-512 and 6/8 digit support;
- otpauth URI import and manual account entry;
- encrypted local account/secret store with Argon2id + AES-256-GCM;
- encrypted recovery-code management with explicit reveal;
- persistent HOTP counter advancement;
- encrypted store master-password rotation;
- Security Center integration and strict sibling-app launch;
- Phase 5 verification script, documentation, and CI coverage.

See [PHASE_5_AUTHENTICATOR.md](PHASE_5_AUTHENTICATOR.md).

## Phase 6 — Security Scanner
**Status: Verified Complete**

Delivered:
- dedicated read-only Security Scanner engine crate and Tauri desktop app;
- Windows Firewall profile assessment;
- BitLocker/system-volume protection assessment;
- Windows Update service and latest-hotfix visibility;
- Microsoft Defender protection-state assessment;
- Secure Boot and User Account Control checks;
- SMB1 and Remote Desktop configuration checks;
- bounded listening TCP endpoint inventory with selected remote-management exposure findings;
- explicit Pass / Attention / Unknown / Info result semantics;
- no elevation, remediation, arbitrary shell input, or secret-store access;
- Security Center integration with strict sibling-app launch;
- Phase 6 verification script, documentation, and CI coverage.

See [PHASE_6_SECURITY_SCANNER.md](PHASE_6_SECURITY_SCANNER.md).

## Phase 7 — Integrity Monitor
**Status: Verified Complete**

Delivered:
- dedicated Integrity Monitor engine crate and Tauri desktop application;
- versioned local baseline with explicit creation/replacement;
- SHA-256 fingerprinting for monitored values and selected files;
- Startup folder, Run/RunOnce, service, scheduled-task, hosts-file, and selected system-configuration coverage;
- Added / Removed / Changed comparison reporting;
- bounded collection and baseline validation;
- fixed Windows probes with no arbitrary command input;
- Security Center integration with strict sibling-app launch;
- Phase 7 verification script, documentation, and CI coverage.

See [PHASE_7_INTEGRITY_MONITOR.md](PHASE_7_INTEGRITY_MONITOR.md).

## Phase 8 — Network Guard
**Status: Verified Complete**

Delivered:
- dedicated Network Guard engine crate and Tauri desktop application;
- Windows-first per-process TCP connection visibility;
- Windows-first UDP endpoint visibility;
- Windows DNS client cache visibility;
- process, listener, and wildcard-exposure summaries;
- bounded probe output and row counts;
- fixed native probes with no arbitrary shell input;
- no packet payload capture, traffic blocking, connection termination, or firewall mutation;
- Security Center integration with strict sibling-app launch;
- Phase 8 verification tooling and CI coverage;
- suite-wide build-all-apps-for-testing script;
- full local Windows verification passed, including formatting, compile checks, strict Clippy, regression tests, JavaScript checks, and all seven desktop application builds;
- verified log SHA-256: `B9734604E9E3550B9742C6440FB21B5E6B418CB7E810317B6CB63AC153FA1549`.

Persistent enforcement remains deferred to the future DragonForge Agent so it can be implemented behind an authenticated, narrow privilege boundary.

See [PHASE_8_NETWORK_GUARD.md](PHASE_8_NETWORK_GUARD.md).

## Phase 9 — Backup & Recovery
**Status: Implementation Complete — Local Verification Pending**

Delivered:
- dedicated encrypted Backup & Recovery engine crate and Tauri desktop application;
- versioned `.dfbackup` format using AES-256-GCM and Argon2id;
- encrypted source metadata, relative paths, SHA-256 manifests, and file contents;
- suite config/data discovery plus selected user file/folder backup;
- bounded collection with symbolic-link and traversal rejection;
- inspect, full integrity verification, and wrong-password/tamper rejection;
- staged no-overwrite restore into a new destination;
- Security Center integration with strict sibling-app launch;
- Phase 9 verification tooling and CI coverage;
- all-app build coverage extended to Backup & Recovery.

See [PHASE_9_BACKUP_RECOVERY.md](PHASE_9_BACKUP_RECOVERY.md).

## Phase 10 — Secure Share
**Status: Next**


Encrypted packages/secrets with expiration and recipient-oriented controls.

## Phase 11 — DragonForge Agent

Unify background monitoring/protection behind authenticated local IPC.

## Later research

Potential areas after the suite is mature:
- ransomware behavior protection;
- reputation services;
- YARA-compatible detection;
- sandbox integration;
- EDR-style telemetry.

A traditional antivirus signature engine is intentionally not an early roadmap goal.
