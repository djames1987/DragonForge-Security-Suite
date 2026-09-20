# DragonForge Security Suite Roadmap

This roadmap is directional. Security and verification take priority over phase numbers.

## Phase 0 — Suite architecture and migration readiness
Current phase.

Goals:
- establish repository/workspace structure;
- document trust boundaries;
- add baseline CI and security policy;
- prepare a history-preserving Password Manager migration plan;
- do not migrate the Password Manager yet.

## Phase 1 — Password Manager migration
Move the tested Password Manager into the suite without functional redesign.

## Phase 2 — Shared foundation
Extract only proven cross-product capabilities such as:
- configuration primitives;
- error/event types;
- platform abstraction;
- logging/redaction policies;
- authenticated IPC foundations.

Crypto is not automatically moved into a generic core crate.

## Phase 3 — Security Center
Build the unified dashboard and orchestration layer.

## Phase 4 — File Vault
Encrypted files/folders and secure containers, reusing vetted cryptographic foundations where appropriate.

## Phase 5 — Authenticator
TOTP/HOTP, recovery material and later hardware-backed authentication integrations.

## Phase 6 — Security Scanner
Assess system security posture, updates, firewall, disk encryption, exposed services and common configuration weaknesses.

## Phase 7 — Integrity Monitor
Baseline and monitor important files, startup locations, services, tasks and system configuration.

## Phase 8 — Network Guard
Per-process network visibility, DNS monitoring and eventually application-level firewall controls.

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
