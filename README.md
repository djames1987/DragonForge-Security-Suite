# DragonForge Security Suite

DragonForge Security Suite is a security-focused Rust workspace hosting a family of interoperable applications and shared security components.

> **Current status:** Phase 5 Authenticator implemented. DragonForge now includes the Security Center, Password Manager, File Vault, and a local encrypted TOTP/HOTP Authenticator.

## Canonical repository

This repository is now the canonical development home for DragonForge Password Manager and all future DragonForge Security Suite components.

- Security Suite / active Password Manager development: https://github.com/djames1987/DragonForge-Security-Suite
- Historical standalone Password Manager baseline: https://github.com/djames1987/DragonForge-Password-Manager

The standalone Password Manager repository remains useful as the pre-migration historical source and verified baseline, but new Password Manager development should occur here.

## Current repository layout

```text
DragonForge-Security-Suite/
├── apps/
│   ├── security-center/               # Unified Tauri desktop dashboard
│   ├── password-manager/              # Migrated Password Manager desktop application
│   ├── file-vault/                    # Encrypted file/folder container application
│   └── authenticator/                 # Encrypted TOTP/HOTP desktop application
├── crates/
│   ├── dragonforge-core/              # Suite-wide non-cryptographic foundation
│   ├── dragonforge-file-vault/        # File Vault container engine
│   ├── dragonforge-authenticator/     # Authenticator OTP + encrypted store engine
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
│   ├── PHASE_1_1_BASELINE.md
│   ├── PHASE_2_SHARED_FOUNDATION.md
│   ├── PHASE_3_SECURITY_CENTER.md
│   ├── PHASE_4_FILE_VAULT.md
│   └── PHASE_5_AUTHENTICATOR.md
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
- Phase 2 — Shared foundation: **Complete**
- Phase 3 — Security Center: **Complete**
- Phase 4 — File Vault: **Complete**
- Phase 5 — Authenticator: **Complete**
- Phase 6 — Security Scanner: **Next**
- Phase 7 — Integrity Monitor
- Phase 8 — Network Guard
- Phase 9 — Backup & Recovery
- Phase 10 — Secure Share
- Phase 11 — DragonForge Agent

See [docs/ROADMAP.md](docs/ROADMAP.md) for details.


## Security Center

The Phase 3 Security Center is a Tauri desktop application under `apps/security-center/`.

Current capabilities:
- suite health summary;
- component registry with accurate Active / Integrated / Planned / Unavailable states;
- local redaction-safe activity history;
- persistent local Security Center settings;
- safe local diagnostic logging;
- explicit DragonForge Agent unavailable state until the background service is implemented;
- launch of a co-located Password Manager executable by exact sibling path;
- shared DragonForge dark/orange UI language.

Run the local verification script on Windows with:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase3-security-center-tests.ps1
```

See [docs/PHASE_3_SECURITY_CENTER.md](docs/PHASE_3_SECURITY_CENTER.md).


## File Vault

Phase 4 adds a local-only Tauri application for encrypted `.dfvault` containers.

Current capabilities:
- encrypt one or more files/folders into a new authenticated container;
- encrypt filenames, relative paths, directory structure, and file bytes;
- inspect and verify a container with its password;
- extract into a new destination without overwriting existing data;
- reject symlink sources and traversal-like archive paths;
- enforce bounded entry/path/data limits;
- integrate with Security Center as an installed suite component.

Run Phase 4 verification on Windows with:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase4-file-vault-tests.ps1
```

See [docs/PHASE_4_FILE_VAULT.md](docs/PHASE_4_FILE_VAULT.md).


## Authenticator

Phase 5 adds a local encrypted TOTP/HOTP authenticator under `apps/authenticator/`.

Current capabilities:
- import standard `otpauth://` TOTP/HOTP URIs;
- manual TOTP/HOTP account setup;
- RFC-compatible code generation;
- encrypted OTP secrets and account metadata;
- encrypted recovery-code storage with explicit reveal;
- persistent HOTP counters;
- Security Center launch integration.

Run Phase 5 verification on Windows with:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase5-authenticator-tests.ps1
```

See [docs/PHASE_5_AUTHENTICATOR.md](docs/PHASE_5_AUTHENTICATOR.md).
