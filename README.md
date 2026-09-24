# DragonForge Security Suite

DragonForge Security Suite is a security-focused Rust workspace hosting a family of interoperable applications and shared security components.

> **Current status:** Phase 22 — Recovery, Migration & Disaster Readiness is **Implementation Complete — Local Verification Pending**. DragonForge now adds encrypted suite recovery packages, configuration/full-suite migration, versioned recovery-schema compatibility, conservative corrupted-state repair, and clean-machine no-overwrite restore semantics.

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

### Password Manager sync server

The Password Manager's multi-device sync backend lives under `services/password-manager-sync/`. A complete PostgreSQL + Docker setup guide is available at [services/password-manager-sync/README.md](services/password-manager-sync/README.md), including account provisioning, Docker Compose startup, automatic database migrations, device enrollment, client configuration, backup guidance, and the HTTPS requirement for non-loopback deployments.


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
- Phase 12.0 — External Test Baseline: **Verified Complete**
- Phase 12.1 — Windows Installer: **Verified Complete**
- Phase 12.2 — Agent Lifecycle: **Verified Complete**
- Phase 12.3 — Release Engineering: **Verified Complete**
- Phase 12.4 — Code Signing: **Verified Complete**
- Phase 12.5 — Crash Handling & Diagnostics: **Verified Complete**
- Phase 12.6 — UX Consistency: **Verified Complete**
- Phase 12.7 — Security Hardening Review: **Verified Complete**
- Phase 13 — Beta Readiness & Release Qualification: **Verified Complete**
- Phase 14 — Secure Update System: **Verified Complete**
- Phase 15 — Password Manager Ecosystem Production Hardening: **Verified Complete**
- Phase 16 — Windows Security Boundary Foundation: **Verified Complete**
- Phase 17 — DragonForge Privileged Service: **Verified Complete**
- Phase 18 — Continuous Integrity Monitoring: **Verified Complete**
- Phase 19 — Network Policy & Firewall Integration: **Verified Complete**
- Phase 20 — Security Center Policy & Event Hub: **Verified Complete**
- Phase 21 — Scheduled Protection & Automation: **Verified Complete**
- Phase 22 — Recovery, Migration & Disaster Readiness: **Implementation Complete — Local Verification Pending**
- Phase 23 — Privacy, Accessibility & Product Polish: **Planned**
- Phase 24 — 1.0 Security & Release Audit: **Planned**
- Phase 25 — DragonForge Security Suite 1.0: **Planned**

See [docs/ROADMAP.md](docs/ROADMAP.md) for details.

## Beta readiness & release qualification

Phase 13 converts external testing into a formal beta gate with required Windows scenarios, retained evidence, release-blocking criteria, machine-readable qualification records, and a release-profile packaging verifier. A single Phase 13 PASS proves the qualification machinery on that machine; the suite is not labeled beta until every required matrix row has evidence.

See [docs/PHASE_13_BETA_READINESS.md](docs/PHASE_13_BETA_READINESS.md), [docs/BETA_QUALIFICATION_MATRIX.md](docs/BETA_QUALIFICATION_MATRIX.md), and [docs/BETA_RELEASE_GATE.md](docs/BETA_RELEASE_GATE.md).

## External testing baseline

Phase 12.0 establishes the external-test process for DragonForge. Security Center can now generate a redaction-safe support report from **About → Copy diagnostics**. External testers should follow [docs/EXTERNAL_TEST_CHECKLIST.md](docs/EXTERNAL_TEST_CHECKLIST.md) and the coverage matrix in [docs/EXTERNAL_TEST_MATRIX.md](docs/EXTERNAL_TEST_MATRIX.md).

Future portable releases include a package-integrity verifier and the external test checklist. Bugs should be filed with the external-test issue template and must never include real credentials, recovery material, OTP seeds, vault contents, or sensitive user files.

See [docs/PHASE_12_0_EXTERNAL_TEST_BASELINE.md](docs/PHASE_12_0_EXTERNAL_TEST_BASELINE.md).

Phase 12.0 local verification result: **PASS** on a second Windows development machine (`DRACO`, Windows NT 10.0.26200.0).  
Verified log: `dragonforge-phase12-external-test-baseline-20260922-093452.log`  
Verified log SHA-256: `96A158EA38F10E8A2F46F092E25C55CBE27E0C3F8BDF359EE844141BE18CDE88`


## Windows installer

Phase 12.1 adds a suite-level, per-user Windows installer while preserving the exact-sibling layout required by Security Center. The normal install path is under `%LOCALAPPDATA%\Programs\DragonForge Security Suite`, so the alpha installer does not require administrator elevation.

Build it with:

~~~powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\package-windows-installer.ps1
~~~

Inno Setup 6 is required on the build machine. The installer preserves DragonForge user data on uninstall by design, creates Start Menu integration, offers an optional desktop shortcut, warns when WebView2 is not detected, and is included by default in the Phase 12.3 exact-tag release workflow.

See [docs/PHASE_12_1_WINDOWS_INSTALLER.md](docs/PHASE_12_1_WINDOWS_INSTALLER.md) and [docs/INSTALLER_TEST_CHECKLIST.md](docs/INSTALLER_TEST_CHECKLIST.md).

Phase 12.1 local verification result: **PASS** on `DRACO` (Windows NT 10.0.26200.0).  
Verified log: `dragonforge-phase12.1-installer-20260922-103728.log`  
Verified log SHA-256: `655086683B19789C3B7CAC44BC0E9E80A3E0F3EE79A2CA38A744A36F9753F898`  
Verified installer SHA-256: `2C8FA84D044BF53562EBD494DAC526F8F94462EAF67C6371E9EA96A6498D23B2`

## Security hardening review

Phase 12.7 reviews DragonForge as one deployed security product. It adds hostile-format and concurrency regression coverage, scheduled/local RustSec advisory auditing, Windows data-permission review tooling, and explicit filesystem/trust/residual-risk documentation.

See [docs/PHASE_12_7_SECURITY_HARDENING.md](docs/PHASE_12_7_SECURITY_HARDENING.md).

Phase 12.7 Windows verification: **PASS** on `DRACO` (Windows NT 10.0.26200.0).  
Verified log: `dragonforge-phase12.7-security-hardening-20260922-144042.log`  
Verified log SHA-256: `D70D7A61DAC9976F67303D428A2F1AFC5E06216074D67DEDD828AA0AE964532C`

## UX consistency

Phase 12.6 aligns all nine DragonForge desktop applications around one visible and behavioral suite contract: current package/suite metadata, Security Center as the recommended entry point, visible keyboard focus, consistent disabled controls, and explicit confirmations for sensitive replacement/restore/extract actions.

See [docs/PHASE_12_6_UX_CONSISTENCY.md](docs/PHASE_12_6_UX_CONSISTENCY.md).

Phase 12.6 Windows verification: **PASS** on `DRACO` (Windows NT 10.0.26200.0).  
Verified log: `dragonforge-phase12.6-ux-consistency-20260922-142449.log`  
Verified log SHA-256: `C5AB4E9062CF311920241640525BA0B4EE0570BE9329C7271FBA69A02AE227EE`

## Crash handling & diagnostics

Phase 12.5 adds a shared component-scoped diagnostics layer with bounded log rotation, secret-aware redaction, safe panic/failure records, per-component failure visibility, WebView2/per-component version metadata, and exportable support bundles from Security Center.

See [docs/PHASE_12_5_CRASH_DIAGNOSTICS.md](docs/PHASE_12_5_CRASH_DIAGNOSTICS.md).

Phase 12.5 Windows verification: **PASS** on `DRACO` (Windows NT 10.0.26200.0).  
Verified log: `dragonforge-phase12.5-crash-diagnostics-20260922-121347.log`  
Verified log SHA-256: `6BAD86F67AC0DF2233A0101BB1AD9812D98418C9DF6C813376FAB60D9637B80C`

## Code signing

Phase 12.4 integrates Windows Authenticode into the verified exact-tag release workflow. Signed builds use SHA-256 file digests, RFC 3161 timestamping with SHA-256, post-sign verification, and release-manifest signer metadata. Certificate private keys remain external to the repository. Stable tags fail closed unless signing is enabled.

See [docs/PHASE_12_4_CODE_SIGNING.md](docs/PHASE_12_4_CODE_SIGNING.md).

Phase 12.4 Windows verification: **PASS** on `DRACO` (Windows NT 10.0.26200.0).  
Verified log: `dragonforge-phase12.4-code-signing-20260922-113515.log`  
Verified log SHA-256: `EE98BCF2D6E0AD1780197698E43AE655BAEE7FDED078C1621B2C100E2B903DA4`

## Release engineering

Phase 12.3 makes release source identity and artifact integrity fail-closed. Version stamping updates the workspace, all Tauri applications, Security Center's visible version, and Inno binary metadata. Tagged builds require tracked/locked dependencies, exact tag-to-HEAD identity, verified portable contents, installer/ZIP SHA-256 sidecars, generated release notes, and a machine-readable release manifest before GitHub publication.

See [docs/PHASE_12_3_RELEASE_ENGINEERING.md](docs/PHASE_12_3_RELEASE_ENGINEERING.md).

Phase 12.3 Windows verification: **PASS** on `DRACO` (Windows NT 10.0.26200.0).  
Verified log: `dragonforge-phase12.3-release-engineering-20260922-112111.log`  
Verified log SHA-256: `E896E0343DD622D1116AD3D63A29FBC8E9E10B9A157900EF762BE354B4C9E037`

## Portable test release

The first external-test package is published and frozen as **v0.1.0-alpha.1**, a Windows x64 portable pre-release. Test machines do not need Rust, Cargo, Node.js, Git, or the source checkout; all ten suite executables are packaged together so Security Center can continue to use exact sibling launch paths.

Phase 12.3 replaces ad-hoc publication with a tag-bound release flow.

Prepare a version on a clean working tree:

~~~powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\prepare-release.ps1 -Version <version>
~~~

After reviewing/committing the stamped files and `Cargo.lock`, create and push an annotated `v<version>` tag. Build and verify that exact tag with:

~~~powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-tagged-windows-release.ps1 -Tag v<version>
~~~

Publish only the already verified tagged source with:

~~~powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\publish-windows-release.ps1 -Tag v<version>
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


## Agent lifecycle

Phase 12.2 extends the verified Phase 11 Agent with lifecycle management while keeping the same normal-user security boundary.

Current lifecycle behavior:
- Security Center starts the exact-sibling Agent automatically when health is needed;
- authenticated graceful Stop and Restart operations;
- restart waits for authenticated reconnection;
- stale/crashed runtime state gets a bounded recovery retry after the existing lock-recovery window;
- explicit Stop suppresses same-session auto-restart until Start/Restart is requested;
- installed builds start the per-user Agent at Windows sign-in through the current user's Startup folder;
- portable builds remain persistence-free and start the Agent on demand;
- no Windows service or elevation is introduced.

See [docs/PHASE_12_2_AGENT_LIFECYCLE.md](docs/PHASE_12_2_AGENT_LIFECYCLE.md).

Phase 12.2 Windows verification: **PASS** on `DRACO` (Windows NT 10.0.26200.0).  
Verified log: `dragonforge-phase12.2-agent-lifecycle-20260922-110040.log`  
Verified log SHA-256: `D7D5EF5979C9AAA250D527192457D6DB476274A30F50A2C2253C81EC1D70B2E5`

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
