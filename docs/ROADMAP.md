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
**Status: Verified Complete**

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
- dedicated Phase 12.4 Windows verification tooling and documentation;
- authoritative Windows verification passed on `DRACO` (Windows NT 10.0.26200.0), including formatting, targeted compile checks, strict Clippy, all 9 Agent tests, all 22 Security Center tests, JavaScript validation, PowerShell syntax validation across signing/release scripts, signing-order checks, stable-release enforcement checks, signing metadata checks, and private-key ignore-policy checks;
- verified log: `dragonforge-phase12.4-code-signing-20260922-113515.log`;
- verified log SHA-256: `EE98BCF2D6E0AD1780197698E43AE655BAEE7FDED078C1621B2C100E2B903DA4`.

Authenticode improves publisher identity and integrity but does not guarantee immediate SmartScreen reputation.

See [PHASE_12_4_CODE_SIGNING.md](PHASE_12_4_CODE_SIGNING.md).


## Phase 12.5 — Crash Handling & Diagnostics
**Status: Verified Complete**

Delivered:
- shared component-scoped rotating logger with 1 MiB default active-log threshold and three bounded backups;
- secret-aware diagnostic sanitization for credentials, vault references, sync/admin tokens, OTP seeds, recovery material, private keys, sensitive paths/content, and authorization headers;
- panic hooks that persist fixed public failure metadata without serializing panic payloads;
- Security Center and DragonForge Agent panic-hook adoption;
- per-component namespaced last-failure records and Security Center visibility across all ten suite components;
- redaction-safe support bundle generation with component log metadata and bounded sanitized Security Center log tail;
- diagnostic schema v2 with per-component application version metadata;
- WebView2 version detection on Windows with explicit null when unavailable;
- existing diagnostic identifier opt-in preserved for Agent PID;
- automated redaction, rotation, and failure-record tests;
- dedicated Phase 12.5 Windows verification tooling and documentation;
- authoritative Windows verification passed on `DRACO` (Windows NT 10.0.26200.0), including rustfmt, targeted compile checks, strict Clippy with `-D warnings`, all 9 Agent tests, all 21 Core tests, all 24 Security Center tests, JavaScript syntax validation, and required Phase 12.5 artifact checks;
- verified log: `dragonforge-phase12.5-crash-diagnostics-20260922-121347.log`;
- verified log SHA-256: `6BAD86F67AC0DF2233A0101BB1AD9812D98418C9DF6C813376FAB60D9637B80C`.

See [PHASE_12_5_CRASH_DIAGNOSTICS.md](PHASE_12_5_CRASH_DIAGNOSTICS.md).


## Phase 12.6 — UX Consistency
**Status: Verified Complete**

Delivered:
- current Version 0.1.0 and Suite Phase 12.6 terminology across all nine DragonForge desktop application surfaces;
- Security Center identified as the recommended suite entry point while direct component launch remains supported;
- historical component phases retained only where they explain product/security scope rather than current suite status;
- shared Phase 12.6 CSS baseline across all nine applications for visible keyboard focus, disabled-control feedback, and suite-version metadata treatment;
- Password Manager sidebar identity normalized from generic "Vault" to "Password Manager";
- stale pre-Phase-11 "future Agent" wording removed from current Integrity Monitor and Network Guard surfaces;
- existing explicit security limitations/non-claims preserved instead of being hidden by generic suite wording;
- sensitive-action confirmation alignment for File Vault extraction, Backup & Recovery restore, Secure Share attachment extraction, and Authenticator recovery-code replacement;
- existing Password Manager deletion, Authenticator account deletion, and Integrity Monitor baseline-replacement confirmations preserved;
- Security Center visible suite milestone updated to Phase 12.6;
- dedicated Phase 12.6 UX contract, Windows verifier, and CMD launcher;
- authoritative Windows verification passed on `DRACO` (Windows NT 10.0.26200.0), including rustfmt, targeted Rust compile checks, JavaScript syntax validation for all nine desktop applications, suite/version metadata checks, shared CSS focus/disabled-state checks, sensitive-action confirmation checks, stale Agent wording checks, and required documentation checks;
- verified log: `dragonforge-phase12.6-ux-consistency-20260922-142449.log`;
- verified log SHA-256: `C5AB4E9062CF311920241640525BA0B4EE0570BE9329C7271FBA69A02AE227EE`.

See [PHASE_12_6_UX_CONSISTENCY.md](PHASE_12_6_UX_CONSISTENCY.md).


## Phase 12.7 — Security Hardening Review
**Status: Verified Complete**

Delivered:
- deployment-wide threat/trust-boundary review across UI/native, Security Center/Agent, sync/network, browser-extension, serialized-input, OS/API, and shared/product crate boundaries;
- Agent IPC review retaining loopback-only transport, HMAC-SHA256 authentication, freshness/nonces, replay rejection, caller/destination authorization, bounded wire messages/timeouts, and no arbitrary command execution;
- active Agent runtime-lock contention regression test;
- File Vault hostile parser tests for unsupported versions, truncation, duplicate entries, malformed traversal-like paths, and truncated internal payloads;
- Backup & Recovery truncated-archive and unsupported-version tests;
- Secure Share truncated-package and unsupported-version tests;
- repeatable local RustSec dependency advisory audit wrapper plus scheduled/on-dependency-change GitHub audit workflow;
- Windows DragonForge data/Agent ACL review tooling that reports ownership and flags broad write-capable ACEs;
- root security policy updated for current authenticated Agent IPC and future authenticated-channel requirements;
- architecture review updated for current Agent capabilities, filesystem expectations, concurrency expectations, and stale pre-Agent wording;
- explicit residual-risk/non-claim register, including inherited Windows Agent runtime ACLs and the non-privileged Agent boundary;
- dedicated Phase 12.7 Windows verification tooling and documentation;
- authoritative Windows verification passed on `DRACO` (Windows NT 10.0.26200.0), including rustfmt, targeted compile checks, strict Clippy with `-D warnings`, all 10 Agent tests, all 6 Backup & Recovery tests, all 21 Core tests, all 11 File Vault tests, all 7 Secure Share tests, all 24 Security Center tests, locked Cargo metadata, live RustSec advisory audit, PowerShell 5.1 syntax validation, and hardening-invariant source checks;
- verified log: `dragonforge-phase12.7-security-hardening-20260922-144042.log`;
- verified log SHA-256: `D70D7A61DAC9976F67303D428A2F1AFC5E06216074D67DEDD828AA0AE964532C`.

Phase 12.7 intentionally does not add antivirus, EDR, packet filtering, or privileged remediation. Dedicated Windows ACL application and any privileged service boundary remain separately reviewed future work.

See [PHASE_12_7_SECURITY_HARDENING.md](PHASE_12_7_SECURITY_HARDENING.md).


## Phase 13 — Beta Readiness & Release Qualification
**Status: Verified Complete**

Delivered:
- formal six-scenario beta qualification matrix covering Windows 10/11, physical/VM, standard/admin-non-elevated, fresh/established profile, WebView2 present/missing, installer/portable, and reboot cases;
- explicit release-blocking versus advisory defect criteria and beta release gate;
- machine-readable qualification-record generator that verifies the log SHA-256 and exact Phase 13 PASS marker before recording evidence;
- scenario IDs constrained to BQ-01 through BQ-06;
- fail-closed beta evidence evaluator requiring all six scenarios to pass for the same candidate commit;
- authoritative Windows verifier covering the complete workspace with all features, strict Clippy, full tests, JavaScript/browser-extension tests, locked Cargo metadata, and RustSec advisory audit;
- real release-profile construction of all ten expected executables;
- qualification-only portable ZIP and Inno Setup installer builds with external/internal SHA-256 verification;
- automatic cleanup of qualification-only artifacts while retaining reusable Cargo build cache;
- Suite Phase 13 metadata across all nine desktop UIs and Security Center diagnostics;
- external-test and installer checklists connected to retained Phase 13 evidence;
- suite CI awareness for Phase 13 scripts, artifacts, PowerShell syntax, and current-phase metadata.

The Phase 13 implementation can be verified on one authoritative machine, but the suite may be labeled beta only after all required matrix rows have evidence and the release-blocking defect gate is clear.

See [PHASE_13_BETA_READINESS.md](PHASE_13_BETA_READINESS.md).

## Phase 14 — Secure Update System
**Status: Verified Complete**

Delivered:
- ML-DSA-65 signed update manifests with a pinned release-key identity;
- explicit alpha, beta, and stable channel policy with cross-channel rejection;
- SemVer downgrade protection and same-version current-state handling;
- Security Center Check / Download & verify / Install verified update workflow;
- bounded HTTPS manifest/artifact retrieval with signed byte length and SHA-256 enforcement;
- mandatory Windows Authenticode verification after download and immediately before installer launch;
- user-controlled installer execution with no silent or unsigned fallback;
- release tooling that binds exact version/tag/commit and verified installer metadata into the signed update manifest;
- update-capable publication requiring Authenticode signing plus out-of-repository ML-DSA signing credentials;
- Phase 14 authoritative verification tooling, documentation, and trust-boundary ADR.

See [PHASE_14_SECURE_UPDATE.md](PHASE_14_SECURE_UPDATE.md).

## Phase 15 — Password Manager Ecosystem Production Hardening
**Status: Verified Complete**

Delivered:
- fail-closed production sync-server mode requiring PostgreSQL persistence, a 256-bit hexadecimal admin token, explicit TLS reverse-proxy declaration, and an HTTPS public base URL;
- bounded request-rate windows keyed by hashed credential/device identity, bounded limiter state, existing body-size limits, and request timeouts;
- no-store, nosniff, and no-referrer HTTP response hardening;
- explicit sync protocol compatibility range in the health endpoint;
- corrected browser-extension packaging from the migrated extension tree with broad-permission rejection;
- stricter Chrome/Edge native-host origin validation;
- hardened Docker runtime defaults with a read-only filesystem, dropped Linux capabilities, no-new-privileges, and a restricted tmpfs;
- PostgreSQL backup/restore tooling with SHA-256 evidence and explicit destructive-restore acknowledgement;
- protocol/version compatibility and database migration guidance;
- dedicated Phase 15 verification tooling and CI coverage.

See [PHASE_15_PASSWORD_MANAGER_HARDENING.md](PHASE_15_PASSWORD_MANAGER_HARDENING.md).

## Phase 16 — Windows Security Boundary Foundation
**Status: Verified Complete**

Delivered:
- dedicated `dragonforge-windows-boundary` policy crate with no privileged runtime behavior;
- fixed future Windows service identity, virtual service account, local named-pipe endpoint, protocol and message bounds;
- explicit OS peer-process verification requirement for Agent/service IPC;
- exact installed Agent executable-path verification requirement;
- valid Authenticode and exact publisher-subject verification requirements;
- deny-by-default privileged capability model with zero Phase 16 privileged capabilities enabled;
- fixed non-privileged command allow-list containing only `health` and `describe-policy`;
- explicit rejection tests for generic `exec`, `shell`, `PowerShell`, `cmd`, and `run` command surfaces;
- documented explicit named-pipe DACL/service-SID requirements for Phase 17;
- ADR defining the future privileged-service trust boundary;
- dedicated Phase 16 verification tooling and CI coverage.

See [PHASE_16_WINDOWS_SECURITY_BOUNDARY.md](PHASE_16_WINDOWS_SECURITY_BOUNDARY.md).

## Phase 17 — DragonForge Privileged Service
**Status: Verified Complete**

Delivered:
- real Windows SCM service executable for the separately reviewed Phase 16 boundary;
- virtual service account and restricted service SID lifecycle;
- explicit local named-pipe DACL with remote-client rejection;
- Windows-reported client PID, exact Agent executable path, Authenticode, and full publisher-subject verification;
- versioned fixed-command protocol with freshness, nonce replay protection, bounded per-process quotas, message limits, and I/O timeouts;
- protected ProgramData service configuration;
- bounded redaction-safe JSONL audit events with rotation;
- Agent health/policy client for the privileged service;
- guarded signed-only Administrator/UAC service installation and removal;
- optional installer integration while preserving the normal per-user non-elevated suite install;
- zero firewall, quarantine, process-control, registry-remediation, or other privileged mutation capabilities enabled in Phase 17;
- dedicated Phase 17 verifier and signed-service acceptance tooling.

See [PHASE_17_DRAGONFORGE_PRIVILEGED_SERVICE.md](PHASE_17_DRAGONFORGE_PRIVILEGED_SERVICE.md).

## Phase 18 — Continuous Integrity Monitoring
**Status: Verified Complete**

Delivered:
- Agent-driven restart-persistent scheduled integrity comparisons;
- versioned continuous-monitor state with bounded event history and persisted due time;
- SHA-256 baseline seal with explicit resealing through the approved baseline create/replace workflow;
- fail-safe alert when the saved baseline changes unexpectedly;
- bounded per-surface key-prefix suppression rules that retain suppressed events for audit;
- Security Center warning/security alerts for new unsuppressed integrity events;
- Integrity Monitor controls for schedule, enable/disable state, suppression rules, seal status, and retained events;
- Agent health capability plus `--integrity-status` and `--integrity-events` diagnostics;
- five-second bounded scheduler polling with 60-second minimum monitoring interval;
- Phase 17 privileged service command surface left unchanged with zero privileged mutation capabilities enabled;
- dedicated Phase 18 Windows verifier, documentation, ADR, and CI coverage.

Phase 18 remains a change detector. It does not claim malware classification, same-user tamper resistance, quarantine, process termination, registry remediation, or firewall enforcement.

See [PHASE_18_CONTINUOUS_INTEGRITY_MONITORING.md](PHASE_18_CONTINUOUS_INTEGRITY_MONITORING.md).

## Phase 19 — Network Policy & Firewall Integration
**Status: Verified Complete**

Delivered:
- controlled native Windows Firewall integration behind the authenticated privileged service;
- typed firewall status/apply/remove/rollback commands with protocol minor-version evolution;
- Phase 19 capability policy enabling only `FirewallPolicyMutation`;
- outbound-only all-profile per-application allow/block rules;
- application PID/name/path/SHA-256 identity context with privileged-service re-hashing before mutation;
- deterministic DragonForge-owned rule namespace/group with collision and ownership checks;
- bounded protected service state for managed policies and rollback records;
- rollback tokens tied to replay-protected privileged requests;
- Network Guard policy UI and exact-sibling Agent routing;
- no direct UI privileged-pipe access and no raw firewall/shell command surface;
- no packet-payload capture, connection termination, inbound custom rules, or unrelated privileged capabilities;
- Phase 19 Windows verifier, documentation, ADR, and CI coverage.

See [PHASE_19_NETWORK_POLICY_FIREWALL.md](PHASE_19_NETWORK_POLICY_FIREWALL.md).

## Phase 20 — Security Center Policy & Event Hub
**Status: Verified Complete**

Delivered:
- versioned persistent bounded Security Center event hub with monotonic IDs;
- normalized component, kind, severity, and open/acknowledged status presentation;
- persistent acknowledgement timestamps and notification filtering by suite-policy severity threshold;
- notification center with acknowledge-one and acknowledge-all workflows;
- persistent bounded component-health history with unchanged-state deduplication;
- coordinated Security Center suite policy for event retention, notification threshold, health-history retention, and signed update channel;
- crash-recoverable event/health state replacement and invalid-state quarantine;
- existing Agent, update, component-launch, support-bundle, and Phase 18 integrity events routed into the durable hub;
- Phase 20 Security Center UI, documentation, ADR, CI coverage, and Windows verifier;
- authoritative Windows verification passed on `DRACO` (Windows NT 10.0.26200.0), including rustfmt, locked Cargo metadata, full workspace/all-target/all-feature compile checks, strict Clippy with `-D warnings`, the complete workspace test suite, Security Center JavaScript validation, Phase 20 invariant checks, PowerShell syntax validation, and all expected Windows test application builds;
- verified log: `dragonforge-phase20-security-center-policy-event-hub-20260923-211911.log`;
- verified log SHA-256: `CACA365D94B6DC9D273183BA7787152AAA47C6420F5839B82F2BCD8FD7E7568A`.

Phase 20 remains local-first and normal-user. It does not add generic automation, automatic remediation, or privileged-service commands.

See [PHASE_20_SECURITY_CENTER_POLICY_EVENT_HUB.md](PHASE_20_SECURITY_CENTER_POLICY_EVENT_HUB.md).

## Phase 21 — Scheduled Protection & Automation
**Status: Verified Complete**

Delivered:
- restart-persistent normal-user Agent scheduler with three fixed capabilities: Security Scanner, explicit integrity check, and encrypted-backup reminder;
- opt-in schedules bounded from 15 minutes through 7 days;
- persisted last/next run state, bounded 250-event history, atomic replacement, backup recovery, invalid-state quarantine, and stale-lock recovery;
- overdue-job recovery plus bounded one-minute retries before returning to the configured cadence;
- Security Scanner jobs isolated from authenticated Agent IPC on a dedicated worker thread;
- explicit Phase 21 integrity-check entry point that preserves the existing Phase 18 sealed-baseline and suppression policy;
- backup scheduling that raises an actionable notification without persisting a backup password or weakening the encrypted backup boundary;
- automation results integrated into the Phase 20 durable event/notification hub with a persistent import cursor;
- Security Center Automation page with schedule controls, run-now actions, last/next run state, failure count, and recent history;
- Agent CLI automation status/configuration/manual-run diagnostics;
- Phase 21 documentation, ADR, CI coverage, and authoritative Windows verifier;
- authoritative Windows verification passed on `DRACO` (Windows NT 10.0.26200.0), including rustfmt, locked Cargo metadata, full workspace/all-target/all-feature compile checks, strict Clippy with `-D warnings`, the complete workspace test suite, Security Center JavaScript/invariant checks, PowerShell syntax validation, and all expected Windows suite application builds;
- verified log: `dragonforge-phase21-scheduled-protection-automation-20260923-215411.log`;
- verified log SHA-256: `5A2477940D368CB8996FDCB3038E17F19D62DB42A5711ADC342463A0FF6FF4E9`.

Phase 21 remains capability-scoped and normal-user. It does not add generic command execution, scripts, arbitrary executable paths/arguments, standing authorization for privileged actions, or silent encrypted-backup credentials.

See [PHASE_21_SCHEDULED_PROTECTION_AUTOMATION.md](PHASE_21_SCHEDULED_PROTECTION_AUTOMATION.md).

## Phase 22 — Recovery, Migration & Disaster Readiness
**Status: Verified Complete**

Delivered:
- encrypted `.dfrecovery` suite recovery package with independent outer-format and logical-schema versioning;
- configuration-only export/import plus full-suite configuration + persistent-data migration;
- Argon2id + AES-256-GCM authenticated encryption with per-entry SHA-256 verification;
- logical config/data namespaces without persisting original absolute source roots;
- clean-machine restore to missing or empty roots only, with staged finalization and rollback if full-suite data finalization fails;
- explicit exclusion of Agent session/runtime credentials, locks, temporary files, backup/quarantine copies, logs, cache data, and support bundles;
- schema v1 -> v2 compatibility migration plus future-schema fail-closed behavior;
- bounded conservative `.json.bak` recovery for missing/corrupt state, including `.invalid` quarantine;
- Backup & Recovery Phase 22 UI for package creation, inspection, verification/migration, clean restore, and state repair;
- Phase 22 documentation, ADR, CI coverage, and authoritative Windows verifier;
- authoritative Windows verification passed on `DRACO` (Windows NT 10.0.26200.0), covering rustfmt, locked Cargo metadata, full workspace/all-target/all-feature compile checks, strict Clippy with `-D warnings`, the complete workspace test and doc-test suite, Security Center and Backup & Recovery JavaScript checks, Phase 22 recovery invariants, PowerShell syntax validation, and all expected Windows suite application builds;
- verified log: `dragonforge-phase22-recovery-migration-disaster-readiness-20260923-222646.log`;
- verified log SHA-256: `E51144C9ED85DB6F577D06AF49CD4AE167CFDD31BB5857E29445E6AF58655837`.

Phase 22 deliberately does not clone OS-bound credentials, privileged-service installation state, Authenticode private keys, operating-system settings, ACLs/ownership, or installed binaries. Destination-machine re-authentication/re-enrollment remains required where a feature uses OS-bound secret storage.

See [PHASE_22_RECOVERY_MIGRATION_DISASTER_READINESS.md](PHASE_22_RECOVERY_MIGRATION_DISASTER_READINESS.md).

## Phase 23 — Privacy, Accessibility & Product Polish
**Status: Verified Complete**

Delivered:
- shared accessibility runtime and stylesheet across all nine desktop applications;
- skip-to-content, visible keyboard focus, navigation arrow/Home/End behavior, active-page ARIA state, and screen-reader live-region support;
- Windows forced-colors/high-contrast, reduced-motion, minimum-control sizing, and DPI/text-scaling resilience;
- reusable `DragonForgeUX` helpers for accessible loading, empty, error, status, busy, and announcement states;
- Security Center one-time first-run privacy/accessibility guidance plus permanent guidance in Settings;
- explicit local-first privacy wording without introducing telemetry, analytics, remote UI resources, or automatic diagnostic uploads;
- localization-ready Phase 23 shell string catalog, locale metadata, and key-based lookup boundary;
- approved DragonForge branding wired into Tauri application bundle icon metadata, while preserving the migrated Password Manager icon;
- accessible installer privacy/data-retention notice added without changing privilege or signing behavior;
- all desktop apps advanced to visible Suite Phase 23 metadata;
- Phase 23 documentation, ADR, CI coverage, and authoritative Windows verifier;
- authoritative Windows verification passed on `DRACO` (Microsoft Windows NT 10.0.26200.0), including formatting, locked metadata, full-workspace/all-target/all-feature compile checks, strict Clippy, complete workspace tests/doc-tests, all Phase 23 UI/accessibility/privacy/branding invariants, PowerShell syntax validation, and all expected Windows suite application builds;
- verified log: `dragonforge-phase23-privacy-accessibility-product-polish-20260923-232110.log`;
- verifier-reported log SHA-256: `C2EA7882638E66075F7D5398A59FEB16757820A94FCEA77DBB4ED277112841C2`.

Phase 23 does not claim formal WCAG certification. Hands-on keyboard, Narrator/NVDA, Windows high-contrast, zoom/DPI, and installer accessibility acceptance remain part of the final Phase 24 release audit.

See [PHASE_23_PRIVACY_ACCESSIBILITY_PRODUCT_POLISH.md](PHASE_23_PRIVACY_ACCESSIBILITY_PRODUCT_POLISH.md).

## Phase 24 — 1.0 Security & Release Audit
**Status: Verified Complete**

Delivered:
- active 1.0 feature-freeze policy covering formats, protocols, privileged capability scope, component scope, and update trust semantics;
- consolidated deployment-wide threat-model review and explicit residual-risk register;
- machine-readable release audit gate with release-blocking conditions and required evidence;
- live RustSec dependency advisory audit;
- locked dependency-license inventory with reviewed SPDX identifier allowlist;
- deterministic same-commit Git source-archive reproducibility verification;
- tracked-source private-key/update-signing-secret scan;
- deterministic fuzz-style mutation corpus for File Vault header parsing and signed-update verification;
- full existing hostile/truncated/tampered Backup & Recovery and Secure Share regressions retained in the workspace gate;
- fail-closed privilege-boundary review for OS peer identity, exact Agent path, Authenticode, publisher pinning, replay/rate abuse, and fixed typed commands;
- secure-update downgrade/channel/signature/AuthentiCode invariants;
- stable-release signing, clean/tagged-source, checksum, timestamp, installer, and immutable-publication invariants;
- installer privilege/optional-service review;
- Phase 24 ADR, CI/scheduled license coverage, and authoritative Windows verifier.

Phase 24 qualifies the repository to enter Phase 25; it does not publish 1.0. Bit-for-bit Windows binary reproducibility is not claimed. Stable release identity is bound through exact source/tag identity, tracked lockfiles, artifact hashes, timestamped Authenticode, and signed update metadata.

Authoritative Windows verification passed on `DRACO` (Windows NT 10.0.26200.0) on 2026-09-24, including full workspace formatting/check/strict-Clippy/tests/doc-tests, RustSec, 525-package license review, exact-commit source reproducibility, JavaScript syntax checks, tracked-source secret scanning, Phase 24 release/security invariants, PowerShell syntax validation, and all expected Windows suite application builds.

Verified log: `dragonforge-phase24-security-release-audit-20260924-084326.log`  
Verified log SHA-256: `F09EC68A5C63C37B8B60C5B38DB62513F6ECA0FE27D33C9CD2738B8B549FE18E`

See [PHASE_24_1_0_SECURITY_RELEASE_AUDIT.md](PHASE_24_1_0_SECURITY_RELEASE_AUDIT.md).

## Phase 25 — DragonForge Security Suite 1.0
**Status: Beta Installer/Uninstaller Qualified — Final Stable Readiness Rerun Pending**

Delivered:
- DragonForge suite/workspace/Tauri/installer source baseline stamped to `1.0.0`;
- all nine desktop application release markers updated to DragonForge 1.0;
- stable 1.x encrypted-format, IPC, privileged-service, update, sync, recovery, and browser/native compatibility commitments;
- 1.0 migration/rollback guide;
- stable support policy and coordinated vulnerability-response policy;
- Phase 24 machine-readable audit gate finalized as verified;
- public Security Center/Agent/release-note privilege-boundary language aligned with the Phase 19 architecture;
- machine-readable Phase 25 release gate and final stable release checklist;
- authoritative Phase 25 Windows release-readiness verifier;
- fail-closed production `v1.0.0` publisher that requires Authenticode + RFC 3161 timestamping + ML-DSA update signing and post-validates the published GitHub release/assets;
- CI coverage for the 1.0 baseline and release documentation.

The repository-side implementation and authoritative Windows release-readiness verification are complete. The DRACO run passed on 2026-09-24 with log `dragonforge-phase25-1.0-release-readiness-20260924-092829.log` and independently verified SHA-256 `7071C7D9EA1FC4CF17CF2F33F786AEA279559C14568344370BA5A1F4E56643C5`.

Before the protected stable-release step, a dedicated Windows testing prerelease now validates the generated installer/uninstaller lifecycle. Because this hardening changes the installer after the recorded readiness PASS, the authoritative Phase 25 readiness verifier must be rerun on the final post-test `main` commit before `v1.0.0` is tagged, signed, and published.

See [PHASE_25_DRAGONFORGE_1_0.md](PHASE_25_DRAGONFORGE_1_0.md).

## Later research

Potential post-1.0 research:
- ransomware behavior protection;
- reputation services;
- YARA-compatible detection;
- sandbox integration;
- EDR-style telemetry.

A traditional antivirus signature engine remains intentionally outside the near-term roadmap.
