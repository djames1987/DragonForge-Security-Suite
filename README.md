# DragonForge Security Suite

DragonForge Security Suite is a security-focused Rust workspace intended to host a family of interoperable applications and shared security components.

> **Current status:** Architecture foundation only. The existing DragonForge Password Manager has **not** been migrated into this repository.

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
│   └── password-manager/              # Reserved migration destination (future)
├── crates/
│   └── dragonforge-core/              # Suite-wide non-cryptographic foundation
├── services/
│   └── dragonforge-agent/             # Future background service
├── extensions/
│   └── password-manager-browser/      # Future browser extension destination
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

No password-manager files are copied by this architecture phase.

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

This repository currently provides the suite architecture and migration foundation. Product functionality will be added in later phases.

