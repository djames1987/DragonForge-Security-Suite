# DragonForge Security Suite

DragonForge Security Suite is a security-focused Rust workspace hosting a family of interoperable applications and shared security components.

> **Current status:** Phase 1.1 baseline established. DragonForge Password Manager has been migrated into this repository and passed the full post-migration Phase 11 verification suite.

## Canonical repository

This repository is now the canonical development home for DragonForge Password Manager and all future DragonForge Security Suite components.

- Security Suite / active Password Manager development: https://github.com/djames1987/DragonForge-Security-Suite
- Historical standalone Password Manager baseline: https://github.com/djames1987/DragonForge-Password-Manager

The standalone Password Manager repository remains useful as the pre-migration historical source and verified baseline, but new Password Manager development should occur here.

## Current repository layout

```text
DragonForge-Security-Suite/
├── apps/
│   ├── security-center/               # Unified suite dashboard foundation
│   └── password-manager/              # Migrated Password Manager desktop application
├── crates/
│   ├── dragonforge-core/              # Suite-wide non-cryptographic foundation
│   ├── dragonforge-crypto/            # Migrated Password Manager cryptography
│   └── dragonforge-vault/             # Migrated encrypted vault implementation
├── services/
│   └── password-manager-sync/         # Migrated Password Manager sync service
├── extensions/
│   └── password-manager-browser/      # Migrated browser extension
├── docs/
│   ├── password-manager/              # Migrated Password Manager documentation
│   ├── ARCHITECTURE.md
│   ├── MIGRATION_PLAN.md
│   ├── ROADMAP.md
│   ├── SECURITY_MODEL.md
│   └── PHASE_1_1_BASELINE.md
├── scripts/
│   └── password-manager/              # Migrated validation and packaging scripts
├── assets/
│   └── branding/                      # Suite and application branding assets
├── .github/workflows/
├── Cargo.toml
├── SECURITY.md
└── rust-toolchain.toml
```

## Verified Password Manager migration

The migration used the tested standalone baseline:

- Source commit: `1ee25e751da094577a9c6bbe10af5bdca92e96ba`
- Final migration branch commit tested locally: `fb48621503561255b2b0ae21248124a05515f985`
- Merge commit into `main`: `e4d2c6debbb28c1cfb58816f1f97ccca51767b41`
- Phase 11 result: **PASS**
- Verified log SHA-256: `DDCEF374F6DD632146CFEFB6F20780B01FAD918ABF2DC5B2089481A91B281ADB`

The migration preserved the rule **move first, refactor second**. No intentional redesign of cryptography, vault formats, key derivation, sync protocol, account recovery, or public Password Manager behavior was part of Phase 1.

See [docs/PHASE_1_1_BASELINE.md](docs/PHASE_1_1_BASELINE.md) for the post-migration baseline record.

## Design goals

- Security-first, auditable Rust components.
- Independent applications backed by shared, narrowly scoped libraries.
- A central Security Center for visibility and orchestration.
- A future background agent for protections that must continue when the UI is closed.
- Strict separation between UI/orchestration code and security-sensitive primitives.
- Incremental refactoring only after tested behavior is preserved.
- No traditional antivirus/signature engine in the initial suite roadmap.

## Current roadmap

- Phase 0 — Suite architecture and migration readiness: **Complete**
- Phase 1 — Password Manager migration: **Complete**
- Phase 1.1 — Post-Migration Baseline Cleanup: **Complete**
- Phase 2 — Shared foundation: **Next**
- Phase 3 — Security Center
- Phase 4 — File Vault
- Phase 5 — Authenticator
- Phase 6 — Security Scanner
- Phase 7 — Integrity Monitor
- Phase 8 — Network Guard
- Phase 9 — Backup & Recovery
- Phase 10 — Secure Share
- Phase 11 — DragonForge Agent

See [docs/ROADMAP.md](docs/ROADMAP.md) for details.
