# DragonForge Security Suite

DragonForge Security Suite is a Windows-first security platform built as a Rust workspace of focused desktop applications, shared security libraries, background services, and supporting integrations. The goal is to keep security-sensitive behavior in narrow native components while giving users one place—**Security Center**—to understand and operate the suite.

> **Status:** 1.0 release readiness has been verified, but the signed stable `v1.0.0` release is still pending. Additional Windows uninstall hardening is being validated before the final release gate is rerun.

## What is in the suite

| Component | Purpose |
| --- | --- |
| **Security Center** | Unified status, events, policy, automation, diagnostics, and application launch surface |
| **Password Manager** | Encrypted credential vault, browser integration, zero-knowledge sync, device enrollment, and recovery |
| **File Vault** | Authenticated encrypted containers for files and folders |
| **Authenticator** | Encrypted TOTP/HOTP account and recovery-code storage |
| **Security Scanner** | Read-only Windows security-posture assessment |
| **Integrity Monitor** | Baseline and change detection for selected system surfaces |
| **Network Guard** | Per-process network and DNS visibility with policy integration |
| **Backup & Recovery** | Encrypted suite backup, verification, migration, and restore workflows |
| **Secure Share** | Offline encrypted packages for recipient-oriented sharing |
| **DragonForge Agent / Privileged Service** | Authenticated background work and narrowly scoped privileged operations |

The active Password Manager implementation also lives here. The separate [DragonForge Password Manager](https://github.com/djames1987/DragonForge-Password-Manager) repository is the historical pre-migration baseline.

## Why this architecture

The suite is deliberately modular rather than one large privileged process. Interactive apps sit above product-owned Rust crates, shared non-cryptographic primitives live in `dragonforge-core`, and background/privileged work crosses authenticated boundaries instead of being performed directly by webview code.

Key trust rules include:

- webview/UI input is not trusted as authority for filesystem, process, policy, or privileged operations;
- security-sensitive formats and cryptographic behavior remain owned by their product crates;
- serialized data is validated before expensive or security-sensitive processing;
- remote Password Manager sync stores opaque encrypted vault data rather than plaintext vault contents;
- diagnostic/event output is designed to be bounded and redaction-safe;
- privileged Windows operations are isolated behind a separate service boundary.

See [Architecture](docs/ARCHITECTURE.md) and [Security Model](docs/SECURITY_MODEL.md) for the detailed design.

## Technology

The repository is centered on **Rust** and **Tauri 2**, with native Windows integrations where the product needs OS security or lifecycle features. It also includes PostgreSQL-backed Password Manager sync, a Chromium Manifest V3 browser extension, release/installer tooling, and Windows validation scripts.

```text
apps/        desktop applications
crates/      product engines and shared Rust libraries
services/    background, sync, and privileged services
extensions/  browser integration
assets/      DragonForge branding/application assets
docs/        architecture, security, release, roadmap, and engineering history
scripts/     validation, packaging, release, and migration tooling
```

## Evaluate the project

For source-level evaluation, start with the workspace checks appropriate to your platform:

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Windows release qualification is more involved than a normal Cargo test run because the suite includes desktop applications, installer behavior, background services, Windows security boundaries, and release-integrity checks. The maintained qualification procedures and evidence expectations are documented under [`docs/`](docs/) and in the [roadmap](docs/ROADMAP.md).

Password Manager sync deployment has its own setup guide at [services/password-manager-sync/README.md](services/password-manager-sync/README.md). Non-loopback deployments require HTTPS/TLS termination.

## Security and maturity

DragonForge is security-focused software, but “security-focused” is not the same as independently audited or risk-free software.

- The stable 1.0 release has **not yet been signed and published**.
- Product-specific security boundaries and residual risks are documented rather than hidden behind a generic “secure” claim.
- Security Scanner findings are posture signals, not malware or vulnerability verdicts.
- Integrity Monitor changes are review signals, not proof of compromise.
- Network Guard visibility does not imply that a listener or connection is malicious.
- Backup & Recovery is not a bare-metal disk-imaging system and does not claim full ACL/reparse-point/live-consistency preservation.
- Secure Share recipient labels are context after decryption, not cryptographic identity authentication.
- Password Manager and its cryptography should not be described as independently audited unless an actual independent audit is completed.

Please report security issues using [SECURITY.md](SECURITY.md). Do not place credentials, recovery material, OTP seeds, vault contents, private user files, or other sensitive data in public issue reports.

## Documentation

- [Architecture](docs/ARCHITECTURE.md) — repository zones, dependency direction, trust boundaries, and component architecture
- [Security Model](docs/SECURITY_MODEL.md) — security assumptions, boundaries, and residual risks
- [Roadmap](docs/ROADMAP.md) — complete implementation history and phase status
- [External Test Checklist](docs/EXTERNAL_TEST_CHECKLIST.md) — external validation guidance
- [Beta Qualification Matrix](docs/BETA_QUALIFICATION_MATRIX.md) — qualification coverage and evidence expectations
- [Password Manager migration record](docs/password-manager/MIGRATION_RECORD.md) — source provenance and migration mapping
- [Third-party notices](THIRD_PARTY_NOTICES.md) — dependency and asset-license obligations
- [`docs/`](docs/) — phase records, release engineering, installer, support, recovery, and product-specific detail

The detailed phase-by-phase build and validation record remains in those documents; the root README is intentionally kept as an entry point rather than a build diary.

## Licensing

Copyright © 2026 David James. All rights reserved.

Original DragonForge material in this repository is **source-visible, not open source**. It is made visible for evaluation, portfolio review, security review, and reference. Except for rights expressly required by GitHub's Terms of Service for public repositories, no general permission is granted to use, copy, modify, redistribute, sublicense, sell, commercially exploit, or incorporate original DragonForge material into another work.

See [LICENSE](LICENSE) for the full DragonForge Proprietary Source Notice. Third-party components retain the rights granted by their own licenses; see [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

Visual branding and documentation assets remain subject to the provenance review recorded in the public-readiness program before public publication. No claim is made here that that publication gate has already been resolved.