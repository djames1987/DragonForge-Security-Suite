# DragonForge Security Suite

DragonForge Security Suite is a security-focused Rust workspace hosting a family of interoperable applications and shared security components.

> **Current status:** Phase 11 DragonForge Agent is verified complete. DragonForge now includes Security Center, Password Manager, File Vault, Authenticator, Security Scanner, Integrity Monitor, Network Guard, Backup & Recovery, Secure Share, and an authenticated per-user background Agent.

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
│   ├── authenticator/                 # Encrypted TOTP/HOTP desktop application
│   ├── security-scanner/              # Read-only system posture scanner
│   ├── integrity-monitor/             # Integrity baseline/change monitor
│   ├── network-guard/                 # Per-process network visibility
│   ├── backup-recovery/               # Encrypted backup and restore application
│   └── secure-share/                   # Offline encrypted recipient-oriented sharing
├── crates/
│   ├── dragonforge-core/              # Suite-wide non-cryptographic foundation
│   ├── dragonforge-agent/             # Agent authenticated IPC/runtime engine
│   ├── dragonforge-file-vault/        # File Vault container engine
│   ├── dragonforge-authenticator/     # Authenticator OTP + encrypted store engine
│   ├── dragonforge-security-scanner/  # Security posture assessment engine
│   ├── dragonforge-integrity-monitor/ # Integrity baseline/change engine
│   ├── dragonforge-network-guard/     # Network visibility engine
│   ├── dragonforge-backup-recovery/   # Encrypted backup/recovery engine
│   ├── dragonforge-secure-share/       # Secure Share encrypted package engine
│   ├── dragonforge-crypto/            # Migrated Password Manager cryptography
│   └── dragonforge-vault/             # Migrated encrypted vault implementation
├── services/
│   ├── password-manager-sync/         # Migrated Password Manager sync service
│   └── dragonforge-agent/             # Per-user authenticated background Agent
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
│   ├── PHASE_5_AUTHENTICATOR.md
│   ├── PHASE_6_SECURITY_SCANNER.md
│   ├── PHASE_7_INTEGRITY_MONITOR.md
│   ├── PHASE_8_NETWORK_GUARD.md
│   ├── PHASE_9_BACKUP_RECOVERY.md
│   ├── PHASE_10_SECURE_SHARE.md
│   └── PHASE_11_DRAGONFORGE_AGENT.md
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
- A background Agent for authenticated work that must continue when the UI is closed.
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
- Phase 5 — Authenticator: **Verified Complete**
- Phase 6 — Security Scanner: **Verified Complete**
- Phase 7 — Integrity Monitor: **Verified Complete**
- Phase 8 — Network Guard: **Verified Complete**
- Phase 9 — Backup & Recovery: **Verified Complete**
- Phase 10 — Secure Share: **Verified Complete**
- Phase 11 — DragonForge Agent: **Verified Complete**

See [docs/ROADMAP.md](docs/ROADMAP.md) for details.

## Portable test release

The first external-test package is prepared as **v0.1.0-alpha.1**, a Windows x64 portable pre-release. Test machines do not need Rust, Cargo, Node.js, Git, or the source checkout; all ten suite executables are packaged together so Security Center can continue to use exact sibling launch paths.

Release builders can create the ZIP with:

~~~powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\package-windows-release.ps1
~~~

Publishing the pre-release from a clean, up-to-date main checkout uses:

~~~powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\publish-windows-release.ps1
~~~

See [docs/PORTABLE_RELEASES.md](docs/PORTABLE_RELEASES.md) and [docs/releases/v0.1.0-alpha.1.md](docs/releases/v0.1.0-alpha.1.md).



## Security Center

The Phase 3 Security Center is a Tauri desktop application under `apps/security-center/`.

Current capabilities:
- suite health summary;
- component registry with accurate Active / Integrated states;
- local redaction-safe activity history;
- persistent local Security Center settings;
- safe local diagnostic logging;
- live authenticated DragonForge Agent health/status reporting and exact-sibling startup;
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


## Security Scanner

Phase 6 adds a read-only Windows-first posture scanner under `apps/security-scanner/`.

Current capabilities:
- assess Windows Firewall profiles and BitLocker/system-volume protection;
- review Windows Update service posture and latest installed hotfix metadata;
- inspect Microsoft Defender protection signals, Secure Boot, and UAC;
- identify SMB1 and Remote Desktop configuration posture;
- inventory listening TCP endpoints with selected remote-management exposure checks;
- report Pass / Attention / Unknown / Info findings without automatically changing system settings;
- launch from Security Center by exact sibling executable path.

Run Phase 6 verification on Windows with:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase6-security-scanner-tests.ps1
```

See [docs/PHASE_6_SECURITY_SCANNER.md](docs/PHASE_6_SECURITY_SCANNER.md).


## Integrity Monitor

Phase 7 adds an on-demand Windows-first integrity baseline monitor under `apps/integrity-monitor/`.

Current capabilities:
- create and explicitly replace a versioned local integrity baseline;
- fingerprint user/common Startup folders and the Windows hosts file;
- fingerprint Run/RunOnce persistence, services, scheduled tasks, and selected security configuration;
- compare current state with the saved baseline and report Added / Removed / Changed entries;
- store hashes and identifiers rather than monitored command/file contents;
- integrate with Security Center using an exact sibling executable path.

Run Phase 7 verification on Windows with:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase7-integrity-monitor-tests.ps1
```

See [docs/PHASE_7_INTEGRITY_MONITOR.md](docs/PHASE_7_INTEGRITY_MONITOR.md).


## Network Guard

Phase 8 adds a Windows-first, visibility-only network application under `apps/network-guard/`.

Current capabilities:
- map TCP connections and listeners to owning process IDs and process names;
- inventory UDP endpoints by owning process;
- show the Windows DNS client cache;
- summarize process count, TCP/UDP endpoints, listeners, and wildcard listeners;
- refresh on demand using fixed native probes;
- integrate with Security Center using an exact sibling executable path.

Phase 8 does not capture packet payloads, block traffic, terminate connections, or modify Windows Firewall. Persistent enforcement is reserved for the future DragonForge Agent.

Run Phase 8 verification on Windows with:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase8-network-guard-tests.ps1
```

Build every desktop application for hands-on testing with:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-all-apps-for-testing.ps1
```

Use `-Profile release` for release-profile test builds.

Phase 8 local verification result: **PASS**  
Verified log SHA-256: `B9734604E9E3550B9742C6440FB21B5E6B418CB7E810317B6CB63AC153FA1549`

See [docs/PHASE_8_NETWORK_GUARD.md](docs/PHASE_8_NETWORK_GUARD.md).


## Backup & Recovery

Phase 9 adds an encrypted local backup application under `apps/backup-recovery/`.

Current capabilities:
- create versioned `.dfbackup` packages from DragonForge suite data and user-selected files/folders;
- encrypt source metadata, paths, integrity hashes, and file contents with AES-256-GCM;
- derive backup keys with Argon2id without persisting the password;
- inspect backup metadata after authentication;
- fully verify every decrypted file against its SHA-256 manifest;
- reject symbolic links, traversal-like paths, duplicate paths, malformed/tampered archives, and bounded-limit violations;
- restore only into a new destination using staged temporary-directory recovery;
- integrate with Security Center through an exact sibling executable path.

Run Phase 9 verification on Windows with:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase9-backup-recovery-tests.ps1
```

Phase 9 local verification result: **PASS**  
Verified log SHA-256: `1633B93A38D828757E82FC9C677BEB8A0CA1AB91CC269E9782130B3CBD3BD82B`

See [docs/PHASE_9_BACKUP_RECOVERY.md](docs/PHASE_9_BACKUP_RECOVERY.md).


## Secure Share

Phase 10 adds an offline encrypted sharing application under `apps/secure-share/`.

Current capabilities:
- create versioned `.dfshare` packages for a named recipient;
- include optional sender label, protected secret text, and file/folder attachments;
- encrypt recipient/sender labels, expiration, paths, hashes, secret text, and attachment bytes with AES-256-GCM;
- derive package keys with Argon2id without persisting the password;
- verify package integrity even after expiration;
- refuse secret reveal or attachment extraction after expiration;
- reject symbolic links, traversal-like paths, duplicate paths, malformed/tampered packages, and bounded-limit violations;
- extract attachments only into a new destination using staged temporary-directory finalization;
- integrate with Security Center through an exact sibling executable path.

Secure Share is intentionally offline in Phase 10. It does not claim remote revocation, guaranteed deletion after expiration, open-count enforcement, or delivery tracking.

Run Phase 10 verification on Windows with:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase10-secure-share-tests.ps1
```

Phase 10 local verification result: **PASS**  
Verified log SHA-256: `71F0900206E214B464CBE82F912EC043FF668BDC9F5CBDCBD65B20A1D8434913`

See [docs/PHASE_10_SECURE_SHARE.md](docs/PHASE_10_SECURE_SHARE.md).


## DragonForge Agent

Phase 11 adds the first real DragonForge background Agent under `services/dragonforge-agent/`.

Current capabilities:
- run as a per-user background process independent of the Security Center UI lifetime;
- bind only to IPv4 loopback;
- generate a fresh 256-bit session credential at each startup;
- authenticate requests and responses with HMAC-SHA256;
- enforce timestamp freshness and bounded nonce replay rejection;
- validate Security Center callers through the Phase 2 IPC authorization policy;
- expose only a narrow authenticated health operation in Phase 11;
- report PID, uptime, transport state, and current capability names;
- launch from Security Center using the exact sibling `dragonforge-agent` executable;
- reject malformed/unauthenticated connections without terminating the Agent.

Phase 11 is intentionally non-elevated and per-user. It does not claim privileged firewall enforcement, process termination, quarantine, or Windows-service protection yet.

Run Phase 11 verification on Windows with:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase11-agent-tests.ps1
```

Phase 11 local verification result: **PASS**  
Verified log SHA-256: `171A70C3496BA8FFADAD42597E23D44E39F66078457982EB0933C0BC069E38BE`

See [docs/PHASE_11_DRAGONFORGE_AGENT.md](docs/PHASE_11_DRAGONFORGE_AGENT.md).
