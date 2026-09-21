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
**Status: Next**

Extract only proven cross-product capabilities such as:
- configuration primitives;
- error/event types;
- platform abstraction;
- logging/redaction policies;
- authenticated IPC foundations.

Crypto is not automatically moved into a generic core crate. Any reuse of Password Manager cryptography must retain clear ownership and receive a dedicated security review.

## Phase 3 — Security Center

Build the unified dashboard and orchestration layer.

## Phase 4 — File Vault

Encrypted files/folders and secure containers, reusing vetted cryptographic foundations where appropriate.

## Phase 5 — Authenticator

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
