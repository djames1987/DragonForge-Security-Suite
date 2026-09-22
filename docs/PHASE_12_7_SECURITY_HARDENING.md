# Phase 12.7 — Security Hardening Review

## Status

**Verified Complete**

Phase 12.7 reviews DragonForge as one deployed security product and converts review findings into regression tests, repeatable audit tooling, and explicit residual-risk documentation.

## Hardening delivered

- deployment-wide trust-boundary review across UI/native, Security Center/Agent, sync/network, browser-extension, serialized-input, OS/API, and product/shared-crate boundaries;
- Agent IPC review confirming loopback-only transport, HMAC-SHA256 request/response authentication, timestamp freshness, nonce replay rejection, caller/destination policy, bounded messages/timeouts, and no arbitrary command execution;
- active Agent runtime-lock contention regression test;
- hostile File Vault parser tests for unsupported versions, truncated headers/payloads, duplicate entries, malformed traversal-like paths, and truncated decoded entry streams;
- Backup & Recovery regression tests for truncated archives and unsupported format versions;
- Secure Share regression tests for truncated packages and unsupported format versions;
- repeatable local RustSec advisory audit wrapper using cargo-audit;
- scheduled/on-dependency-change GitHub RustSec audit workflow;
- Windows DragonForge data/Agent ACL review script that reports ownership and flags broad write-capable ACEs;
- security policy updated to reflect authenticated Agent IPC and mandatory authentication for future security-sensitive channels;
- architecture updated to remove stale pre-Agent claims and record Phase 12.7 deployment-wide trust conclusions;
- explicit residual-risk and non-claim register.

## Filesystem and permission review

DragonForge remains a per-user application suite. Installed executables and user-profile data are not represented as tamper-resistant against the owning user or a local administrator.

Current suite paths are user-scoped. Agent runtime files are component-scoped under the DragonForge local data root. Unix runtime files are explicitly restricted to mode 0600 by the Agent writer. Windows currently relies on inherited profile ACLs rather than a dedicated Rust-applied ACL. The Phase 12.7 Windows ACL review script flags broad write-capable entries for review.

That Windows ACL limitation is a residual risk, not a blocker for the current non-elevated Agent model. It is a blocker for any future claim of privileged or tamper-resistant enforcement.

## Dependency security

The repository now has two repeatable RustSec paths:

- local: scripts/run-dependency-audit.ps1;
- CI/scheduled: .github/workflows/security-audit.yml.

The local wrapper runs cargo audit and can explicitly install cargo-audit when requested. Advisory ignores must be documented with applicability analysis.

## Parser hardening

Encrypted container/package inputs are treated as untrusted until their magic/version, length constraints, authenticated encryption, internal metadata, path rules, duplicate rules, and content hashes have passed validation.

Phase 12.7 adds regression coverage specifically for malformed/truncated/unsupported/duplicate cases that are easy to accidentally weaken during later format evolution.

## Concurrency and race review

The Agent single-instance lock is now explicitly regression-tested against a second live owner. File Vault, Backup & Recovery, and Secure Share continue to use create-new/no-overwrite behavior and randomized sibling staging for extraction/restore rather than merging into an existing target.

Broader multi-process transaction coordination for every product store is not claimed in this phase and remains a residual area for future targeted work if multi-writer use cases are introduced.

## Residual risks and non-claims

- The current Agent is normal-user/per-user, not a privileged Windows service or tamper-resistant EDR boundary.
- Windows Agent runtime files depend on inherited profile ACLs; the suite reviews but does not yet apply a dedicated Windows DACL in Rust.
- Authenticode establishes publisher/integrity evidence but does not guarantee immediate SmartScreen reputation.
- Network Guard remains visibility-only and does not block traffic or mutate Windows Firewall.
- Integrity Monitor remains on-demand and is not a continuous kernel/filesystem monitor.
- Security Scanner is a bounded posture assessment, not an antivirus/vulnerability-feed/exploit engine.
- Secure Share offline expiration relies on the local clock and cannot provide remote revocation or guaranteed deletion.
- Backup & Recovery is not a bare-metal/disk-image backup and does not preserve every filesystem metadata/ACL/reparse-point semantic.
- User-chosen vault/archive/share locations can be deleted, corrupted, rolled back, or denied by storage outside DragonForge's control.
- A compromised same-user desktop session or local administrator can attack process memory, UI automation, files, and installed binaries; the current suite does not claim defense against that threat class.

## Verification

Run:

    powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase12.7-security-hardening-tests.ps1

The first run may install cargo-audit if it is missing. The verifier creates a timestamped log and SHA-256 sidecar under test-logs\.

## Verified Windows result

Phase 12.7 completed authoritative Windows verification on `DRACO`.

- Machine: `DRACO`
- Windows: `Microsoft Windows NT 10.0.26200.0`
- Result: **PASS**
- Verified log: `dragonforge-phase12.7-security-hardening-20260922-144042.log`
- Log SHA-256: `D70D7A61DAC9976F67303D428A2F1AFC5E06216074D67DEDD828AA0AE964532C`

The passing run covered rustfmt, targeted compile checks, strict Clippy with `-D warnings`, all 10 Agent tests, all 6 Backup & Recovery tests, all 21 Core tests, all 11 File Vault tests, all 7 Secure Share tests, all 24 Security Center tests, locked Cargo metadata, a live RustSec dependency advisory audit, PowerShell 5.1 syntax validation for the Phase 12.7 security scripts, and required hardening-invariant checks.
