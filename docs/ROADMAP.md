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
**Status: Next**

TOTP/HOTP, recovery material, and later hardware-backed authentication integrations.

## Phase 6 — Security Scanner

Assess system security posture, updates, firewall, disk encryption, exposed services, and common configuration weaknesses.

## Phase 7 — Integrity Monitor

Baseline and monitor important files, startup locations, services, tasks, and system configuration.

## Phase 8 — Network Guard

Per-process network visibility, DNS monitoring, and eventually application-level firewall controls.

## Phase 9 — Backup & Recovery

Encrypted, verifiable backup/recovery for suite data and selected user data.

## Phase 10 — Secure Share

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
