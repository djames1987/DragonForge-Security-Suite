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
**Status: Verified Complete**

Delivered:
- dedicated Authenticator engine crate and Tauri desktop app;
- RFC-compatible TOTP and HOTP generation;
- SHA-1/SHA-256/SHA-512 and 6/8 digit support;
- otpauth URI import and manual account entry;
- encrypted local account/secret store with Argon2id + AES-256-GCM;
- encrypted recovery-code management with explicit reveal;
- persistent HOTP counter advancement;
- encrypted store master-password rotation;
- Security Center integration and strict sibling-app launch;
- Phase 5 verification script, documentation, and CI coverage.

See [PHASE_5_AUTHENTICATOR.md](PHASE_5_AUTHENTICATOR.md).

## Phase 6 — Security Scanner
**Status: Verified Complete**

Delivered:
- dedicated read-only Security Scanner engine crate and Tauri desktop app;
- Windows Firewall profile assessment;
- BitLocker/system-volume protection assessment;
- Windows Update service and latest-hotfix visibility;
- Microsoft Defender protection-state assessment;
- Secure Boot and User Account Control checks;
- SMB1 and Remote Desktop configuration checks;
- bounded listening TCP endpoint inventory with selected remote-management exposure findings;
- explicit Pass / Attention / Unknown / Info result semantics;
- no elevation, remediation, arbitrary shell input, or secret-store access;
- Security Center integration with strict sibling-app launch;
- Phase 6 verification script, documentation, and CI coverage.

See [PHASE_6_SECURITY_SCANNER.md](PHASE_6_SECURITY_SCANNER.md).

## Phase 7 — Integrity Monitor
**Status: Verified Complete**

Delivered:
- dedicated Integrity Monitor engine crate and Tauri desktop application;
- versioned local baseline with explicit creation/replacement;
- SHA-256 fingerprinting for monitored values and selected files;
- Startup folder, Run/RunOnce, service, scheduled-task, hosts-file, and selected system-configuration coverage;
- Added / Removed / Changed comparison reporting;
- bounded collection and baseline validation;
- fixed Windows probes with no arbitrary command input;
- Security Center integration with strict sibling-app launch;
- Phase 7 verification script, documentation, and CI coverage.

See [PHASE_7_INTEGRITY_MONITOR.md](PHASE_7_INTEGRITY_MONITOR.md).

## Phase 8 — Network Guard
**Status: Verified Complete**

Delivered:
- dedicated Network Guard engine crate and Tauri desktop application;
- Windows-first per-process TCP connection visibility;
- Windows-first UDP endpoint visibility;
- Windows DNS client cache visibility;
- process, listener, and wildcard-exposure summaries;
- bounded probe output and row counts;
- fixed native probes with no arbitrary shell input;
- no packet payload capture, traffic blocking, connection termination, or firewall mutation;
- Security Center integration with strict sibling-app launch;
- Phase 8 verification tooling and CI coverage;
- suite-wide build-all-apps-for-testing script;
- full local Windows verification passed, including formatting, compile checks, strict Clippy, regression tests, JavaScript checks, and all seven desktop application builds;
- verified log SHA-256: `B9734604E9E3550B9742C6440FB21B5E6B418CB7E810317B6CB63AC153FA1549`.

Persistent enforcement remains deferred to the future DragonForge Agent so it can be implemented behind an authenticated, narrow privilege boundary.

See [PHASE_8_NETWORK_GUARD.md](PHASE_8_NETWORK_GUARD.md).

## Phase 9 — Backup & Recovery
**Status: Verified Complete**

Delivered:
- dedicated encrypted Backup & Recovery engine crate and Tauri desktop application;
- versioned `.dfbackup` format using AES-256-GCM and Argon2id;
- encrypted source metadata, relative paths, SHA-256 manifests, and file contents;
- suite config/data discovery plus selected user file/folder backup;
- bounded collection with symbolic-link and traversal rejection;
- inspect, full integrity verification, and wrong-password/tamper rejection;
- staged no-overwrite restore into a new destination;
- Security Center integration with strict sibling-app launch;
- Phase 9 verification tooling and CI coverage;
- all-app build coverage extended to Backup & Recovery;
- full local Windows verification passed, including formatting, compile checks, strict Clippy, Phase 9 and regression tests, JavaScript validation, and all eight desktop application builds;
- verified log SHA-256: `1633B93A38D828757E82FC9C677BEB8A0CA1AB91CC269E9782130B3CBD3BD82B`.

See [PHASE_9_BACKUP_RECOVERY.md](PHASE_9_BACKUP_RECOVERY.md).

## Phase 10 — Secure Share
**Status: Verified Complete**

Delivered:
- dedicated Secure Share engine crate and Tauri desktop application;
- versioned `.dfshare` offline encrypted package format using AES-256-GCM and Argon2id;
- encrypted sender/recipient labels, expiration, secret text, attachment paths, SHA-256 hashes, and attachment bytes;
- recipient-oriented package metadata with required recipient label;
- optional protected secret text and file/folder attachments;
- expiration enforcement for secret reveal and attachment extraction while retaining post-expiration integrity verification;
- bounded collection with symbolic-link, traversal, duplicate-path, wrong-password, and tamper rejection;
- staged no-overwrite attachment extraction;
- explicit non-claims for offline revocation and open-count enforcement;
- Security Center integration with strict sibling-app launch;
- Phase 10 verification tooling and CI coverage;
- all-app build coverage extended to nine desktop applications;
- full local Windows verification passed, including formatting, compile checks, strict Clippy, Secure Share and regression tests, JavaScript validation, and all nine desktop application builds;
- verified log SHA-256: `71F0900206E214B464CBE82F912EC043FF668BDC9F5CBDCBD65B20A1D8434913`.

See [PHASE_10_SECURE_SHARE.md](PHASE_10_SECURE_SHARE.md).

## Phase 11 — DragonForge Agent
**Status: Verified Complete**

Delivered:
- dedicated Agent runtime/client crate and background service executable;
- per-user loopback-only IPC transport;
- random per-session 256-bit credential;
- HMAC-SHA256 authenticated requests and responses;
- timestamp freshness and nonce replay protection;
- Phase 2 caller/destination authorization policy enforcement;
- bounded wire messages and socket timeouts;
- single-instance lock and stale-runtime cleanup;
- authenticated health/status endpoint with PID, uptime, and capability reporting;
- Security Center live Agent status and exact sibling launch;
- isolated end-to-end authenticated client/server round-trip test;
- Phase 11 verification tooling and CI coverage;
- all-suite build extended to include `dragonforge-agent`;
- full local Windows verification passed, including formatting, compile checks, strict Clippy, Agent and regression tests, JavaScript validation, and all ten expected suite executables;
- verified log SHA-256: `171A70C3496BA8FFADAD42597E23D44E39F66078457982EB0933C0BC069E38BE`.

Phase 11 is intentionally per-user and non-elevated. Privileged enforcement remains future work behind a separately reviewed Windows service boundary.

See [PHASE_11_DRAGONFORGE_AGENT.md](PHASE_11_DRAGONFORGE_AGENT.md).


## Phase 12.0 — External Test Baseline
**Status: Verified Complete**

Delivered:
- redaction-safe Security Center diagnostic report with one-click copy action;
- package version, exact build commit, Phase 12.0, release channel, platform, architecture, component states, and Agent capability metadata;
- diagnostic-identifier gating for Agent PID;
- explicit exclusion of vault paths, credentials, event/log contents, sync tokens, recovery material, OTP secrets, and user file contents from diagnostics;
- external Windows test matrix and tester checklist;
- structured external-test GitHub bug report template;
- future portable-release package integrity verifier and bundled external tester checklist;
- Phase 12.0 local verification script with log + SHA-256 sidecar;
- cross-machine Windows verification passed on `DRACO` (Windows NT 10.0.26200.0), including formatting, targeted compile checks, strict Clippy with `-D warnings`, Agent and Security Center tests, JavaScript syntax validation, and required Phase 12.0 artifact checks;
- verified log: `dragonforge-phase12-external-test-baseline-20260922-093452.log`;
- verified log SHA-256: `96A158EA38F10E8A2F46F092E25C55CBE27E0C3F8BDF359EE844141BE18CDE88`.

The existing v0.1.0-alpha.1 release remains frozen. Fixes discovered through external testing should ship in a new pre-release rather than replacing that release in place.

Phase 12.0 scope alignment note:
- the current diagnostic report already records platform/OS family, architecture, suite package version, exact build commit, release channel, component states, and Agent capabilities;
- WebView2 **version** and per-component application **version** metadata are not yet recorded in the diagnostic JSON and remain explicit follow-up work for the diagnostics/release-hardening cycle;
- diagnostics must continue to exclude passwords, vault contents, sync/admin tokens, OTP seeds, recovery material, private keys, sensitive file contents, and similar secrets.

See [PHASE_12_0_EXTERNAL_TEST_BASELINE.md](PHASE_12_0_EXTERNAL_TEST_BASELINE.md).


## Phase 12.1 — Windows Installer
**Status: Verified Complete**

Delivered:
- suite-level Inno Setup installer that keeps all ten DragonForge executables together;
- per-user default installation under LocalAppData with no elevation required for the normal path;
- stable installer AppId for in-place alpha upgrades;
- Start Menu integration, optional desktop shortcut, and Security Center post-install launch;
- explicit WebView2 detection/warning without silently downloading third-party executables, with clear direction to install the Microsoft Edge WebView2 Evergreen Runtime when missing;
- upgrade-time exact-path Agent shutdown plus Windows application-closing integration;
- uninstall behavior that removes installed program files while preserving DragonForge user data and externally stored vault/backup/share files;
- installer-specific staged BUILD-INFO and SHA256SUMS metadata;
- SHA-256 sidecar for the compiled installer;
- optional GitHub pre-release publication of installer + checksum assets;
- manual installer acceptance checklist;
- Phase 12.1 Windows verification harness with real installer compilation;
- authoritative Windows verification passed on `DRACO` (Windows NT 10.0.26200.0), including formatting, targeted compile checks, strict Clippy, Agent/Security Center tests, JavaScript validation, all ten release builds, portable packaging, real Inno Setup installer compilation, installer checksum validation, and staging cleanup checks;
- verified log: `dragonforge-phase12.1-installer-20260922-103728.log`;
- verified log SHA-256: `655086683B19789C3B7CAC44BC0E9E80A3E0F3EE79A2CA38A744A36F9753F898`;
- verified installer SHA-256: `2C8FA84D044BF53562EBD494DAC526F8F94462EAF67C6371E9EA96A6498D23B2`.

See [PHASE_12_1_WINDOWS_INSTALLER.md](PHASE_12_1_WINDOWS_INSTALLER.md).


## Phase 12.2 — Agent Lifecycle
**Status: Verified Complete**

Delivered:
- automatic exact-sibling Agent startup when Security Center requests dashboard/health state;
- authenticated graceful `shutdown` IPC action and `dragonforge-agent --stop` CLI;
- Agent protocol minor version 1 for lifecycle-capability evolution;
- graceful runtime cleanup through the existing descriptor/session-key/lock guard;
- Security Center Start / Restart / Stop controls;
- authenticated restart with health reconnection confirmation;
- bounded stale/crashed runtime recovery retry using the existing Phase 11 stale-lock window;
- explicit manual-stop suppression so Security Center does not immediately relaunch an Agent the user intentionally stopped;
- current-user Windows Startup entry for installed builds, giving normal-user login/reboot recovery without a Windows service;
- portable-build on-demand recovery with no persistence changes;
- health capability reporting for `graceful-shutdown` and `restartable-session`;
- tests for authenticated shutdown cleanup and manual-stop suppression;
- dedicated Phase 12.2 Windows verification tooling and documentation;
- authoritative Windows verification passed on `DRACO` (Windows NT 10.0.26200.0): formatting, targeted compile checks, strict Clippy, 9 Agent tests, 22 Security Center tests, JavaScript validation, and required Phase 12.2 artifact checks;
- verified log: `dragonforge-phase12.2-agent-lifecycle-20260922-110040.log`;
- verified log SHA-256: `D7D5EF5979C9AAA250D527192457D6DB476274A30F50A2C2253C81EC1D70B2E5`.

Phase 12.2 remains normal-user and per-user. It does not introduce a privileged Windows service, elevation, firewall mutation, process termination, quarantine, or arbitrary command execution.

See [PHASE_12_2_AGENT_LIFECYCLE.md](PHASE_12_2_AGENT_LIFECYCLE.md).


## Phase 12.3 — Release Engineering
**Status: Verified Complete**

Delivered:
- automated suite-wide version stamping across the Rust workspace, all nine Tauri apps, Security Center's visible version, and Inno binary metadata;
- release-preparation workflow that refreshes and validates `Cargo.lock`;
- exact-tag Windows release orchestrator that refuses dirty or non-matching source revisions;
- tracked `Cargo.lock` enforcement plus `cargo metadata --locked`;
- tag/commit identity injected into portable and installer BUILD-INFO metadata;
- SHA-256 sidecars for ZIP, installer, and release manifest;
- portable ZIP verification of all ten expected executables and every internal `SHA256SUMS.txt` entry;
- machine-readable release manifest containing tag, commit, platform, executable set, artifact hashes/sizes, and signing state;
- repository-controlled release-notes template with Git-history-driven note generation;
- immutable GitHub publication flow that verifies an existing tag and refuses to overwrite an existing release;
- prerelease/stable channel selection derived from the tag version;
- explicit unsigned Phase 12.3 signing state ready for Phase 12.4 insertion;
- dedicated Phase 12.3 Windows verification tooling and documentation;
- authoritative Windows verification passed on `DRACO` (Windows NT 10.0.26200.0), including formatting, targeted compile checks, strict Clippy, Agent/Security Center tests, JavaScript syntax validation, tracked/locked Cargo metadata, suite-wide release-version consistency checks, PowerShell syntax validation for every release script, and repository-controlled release invariant checks;
- verified log: `dragonforge-phase12.3-release-engineering-20260922-112111.log`;
- verified log SHA-256: `E896E0343DD622D1116AD3D63A29FBC8E9E10B9A157900EF762BE354B4C9E037`.

See [PHASE_12_3_RELEASE_ENGINEERING.md](PHASE_12_3_RELEASE_ENGINEERING.md).


## Phase 12.4 — Code Signing
**Status: Implementation Complete — Local Verification Pending**

Delivered:
- Windows Authenticode signing helper using Microsoft SignTool;
- SHA-256 file digests and RFC 3161 timestamping with SHA-256;
- support for Windows certificate-store thumbprints or externally supplied PFX credentials;
- signing-secret isolation from the repository plus PFX/private-key ignore rules;
- portable-release executable signing before internal SHA-256 manifests and ZIP creation;
- installer signing after Inno compilation but before installer sidecar hashing;
- immediate post-sign SignTool verification with timestamp requirement;
- final release-artifact signature verification before publication;
- signer subject/thumbprint and signing-state metadata in the machine-readable release manifest;
- stable release tags fail closed unless signing is enabled;
- prerelease/development builds may remain explicitly unsigned;
- documented certificate rotation, revocation, timestamp outage, and compromised-key response procedures;
- dedicated Phase 12.4 Windows verification tooling and documentation.

Authenticode improves publisher identity and integrity but does not guarantee immediate SmartScreen reputation.

See [PHASE_12_4_CODE_SIGNING.md](PHASE_12_4_CODE_SIGNING.md).


## Phase 12.5 — Crash Handling & Diagnostics
**Status: Planned**

Goals:
- add panic/crash logging that never records secrets;
- provide per-component diagnostic logs with consistent metadata and redaction policy;
- implement bounded log rotation and retention;
- create an exportable support bundle suitable for bug reports;
- add automated redaction tests covering credentials, vault paths/content, sync/admin tokens, OTP seeds, recovery material, private keys, and sensitive user data;
- add Security Center visibility for the most recent failure/status of each installed application;
- extend diagnostic metadata to include WebView2 version and per-component application versions;
- keep diagnostic collection metadata-focused and opt-in where identifiers could increase privacy risk.


## Phase 12.6 — UX Consistency
**Status: Planned**

Goals:
- make every DragonForge application visibly and behaviorally part of one suite;
- standardize navigation patterns, typography, spacing, buttons, dialogs, status terminology, error presentation, version display, and About pages;
- standardize safe confirmation patterns for destructive/security-sensitive actions;
- make Security Center the primary suite entry point while preserving direct component launching for troubleshooting and advanced use;
- align component health/status presentation between Security Center and individual applications;
- ensure installer, portable package, diagnostics, and application UI use consistent product/version terminology.


## Phase 12.7 — Security Hardening Review
**Status: Planned**

Goals:
- threat-model DragonForge as one deployed product rather than as isolated applications;
- review filesystem permissions and ownership expectations for installed binaries, runtime files, configuration, logs, and security-sensitive data;
- review Agent and cross-component IPC authentication/authorization boundaries;
- review all suite data directories and persistence locations;
- audit logs and diagnostic surfaces for secret leakage;
- perform dependency/security advisory review and establish repeatable dependency-audit tooling;
- fuzz or property-test package/container parsers, including `.dfvault`, `.dfbackup`, `.dfshare`, and other versioned formats;
- add corrupted, truncated, oversized, malformed, duplicate-entry, and unsupported-version tests;
- expand malicious path, traversal, symlink/reparse-point, and destination-overwrite tests;
- add concurrent-access/race-condition tests around vaults, stores, backups, Agent runtime files, and other shared state;
- document residual risks and explicit non-claims before moving toward a broader beta/stable channel.

Phase 12.7 is a hardening/review phase, not a mandate to add antivirus, EDR, packet filtering, or privileged remediation features.


## Later research

Potential areas after the suite is mature:
- ransomware behavior protection;
- reputation services;
- YARA-compatible detection;
- sandbox integration;
- EDR-style telemetry.

A traditional antivirus signature engine is intentionally not an early roadmap goal.
