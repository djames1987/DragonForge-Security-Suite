# DragonForge Security Suite

DragonForge Security Suite is a security-focused Rust workspace intended to host a family of interoperable applications and shared security components.

> **Current status:** Password Manager migration is in progress on `migration/password-manager-1ee25e7`, using tested source commit `1ee25e751da094577a9c6bbe10af5bdca92e96ba`.

## Design goals

- Security-first, auditable Rust components.
- Independent applications backed by shared, narrowly-scoped libraries.
- A central Security Center for visibility and orchestration.
- A background agent for protections that must continue when the UI is closed.
- Strict separation between UI/orchestration code and security-sensitive primitives.
- Incremental migration: working products are moved first, then common code is extracted only after behavior is verified.
- No traditional antivirus/signature engine is planned for the initial suite.

## Planned repository layout

```text
DragonForge-Security-Suite/
├── apps/
│   ├── security-center/               # Unified desktop dashboard
│   └── password-manager/              # DragonForge Password Manager desktop app
├── crates/
│   └── dragonforge-core/              # Suite-wide non-cryptographic foundation
├── services/
│   └── dragonforge-agent/             # Future background service
├── extensions/
│   └── password-manager-browser/      # Password Manager browser extension
├── docs/
│   ├── ARCHITECTURE.md
│   ├── MIGRATION_PLAN.md
│   ├── ROADMAP.md
│   └── SECURITY_MODEL.md
├── scripts/
├── .github/workflows/
├── Cargo.toml
├── SECURITY.md
└── rust-toolchain.toml
```

## Password Manager migration

The current password manager remains in its existing repository while active testing and development continue:

- Source repository: https://github.com/djames1987/DragonForge-Password-Manager
- Suite repository: https://github.com/djames1987/DragonForge-Security-Suite

The Password Manager is being migrated from the tested standalone baseline. The original repository remains available as the historical source until post-migration verification is complete.

See [docs/MIGRATION_PLAN.md](docs/MIGRATION_PLAN.md) for the staged migration procedure.

## Initial product roadmap

1. DragonForge Password Manager
2. DragonForge Security Center
3. DragonForge File Vault
4. DragonForge Authenticator
5. DragonForge Security Scanner
6. DragonForge Integrity Monitor
7. DragonForge Network Guard
8. DragonForge Backup & Recovery
9. DragonForge Secure Share
10. DragonForge Agent / unified monitoring

See [docs/ROADMAP.md](docs/ROADMAP.md) for details.

## Development status

This repository contains the suite architecture plus the in-progress Password Manager migration. The migration must pass its original regression suite before the suite becomes the canonical development location.

