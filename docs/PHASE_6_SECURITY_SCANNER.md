# Phase 6 — Security Scanner

## Status

**Verified Complete**

Phase 6 adds DragonForge Security Scanner as a dedicated read-only local posture assessment application.

## Delivered

- product-owned `dragonforge-security-scanner` Rust crate;
- Tauri desktop application under `apps/security-scanner/`;
- fixed Windows posture probes for:
  - Windows Defender Firewall profile state;
  - BitLocker/system-volume protection;
  - Windows Update service policy;
  - latest installed Windows hotfix metadata;
  - Microsoft Defender antivirus / real-time / behavior-monitoring state;
  - Secure Boot;
  - User Account Control;
  - SMB1 optional feature state;
  - Remote Desktop enablement;
  - listening TCP endpoints and selected remote-management ports;
- structured findings with category, severity, status, evidence, and recommendation;
- explicit `Unknown` results when a probe cannot safely determine posture;
- bounded listener inventory processing;
- no automatic remediation, no elevation, and no arbitrary shell input;
- Security Center integration and strict sibling executable launch;
- Phase 6 verification script and CI coverage.

## Read-only trust boundary

The Security Scanner is intentionally observational in Phase 6.

The webview cannot provide shell fragments, executable paths, registry paths, PowerShell scripts, or remediation commands. Native scanner probes are hard-coded in Rust and use a fixed PowerShell invocation on Windows.

Phase 6 does **not**:
- change firewall rules;
- enable or disable BitLocker;
- install updates;
- alter Defender policy;
- change UAC, SMB1, Secure Boot, or Remote Desktop configuration;
- stop services or close ports;
- request administrator elevation;
- scan Password Manager, File Vault, or Authenticator secret material;
- claim to be a traditional antivirus, exploit scanner, or vulnerability-feed product.

## Findings model

Each finding contains:
- stable finding ID;
- category;
- title;
- severity;
- status;
- concise summary;
- bounded evidence text;
- recommendation.

Statuses are:
- `pass` — the observed setting matches the scanner's safe baseline;
- `attention` — the observed posture deserves user review;
- `unknown` — the scanner could not determine the setting safely;
- `info` — inventory/context only; not a pass/fail assertion.

The scanner does not compute a security score. A single summary count must not be presented as proof that a device is secure.

## Windows probe behavior

Some Windows checks may be unavailable on a particular edition, firmware mode, enterprise policy, or user privilege level. Those cases return `Unknown` rather than fabricating a result.

The latest-hotfix probe is informational only. It does not claim that the system has no pending updates.

Listening-port assessment inventories local TCP listeners and calls attention only to a small selected set of commonly sensitive remote-management/service ports when bound to wildcard addresses. It does not label an open port as a vulnerability by itself.

## Verification

Run on Windows:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase6-security-scanner-tests.ps1
```

The verifier checks formatting, compilation, strict Clippy, Scanner regressions, prior suite regressions, JavaScript syntax, and builds all Security Center sibling applications.

Authoritative Windows verification passed on 2026-09-21 using `dragonforge-phase6-security-scanner-20260921-114027.log`.

SHA-256: `9932DC5308851B74BECEDC510D595752750243E73983EA2AF83E98DE92260D76`.

## Phase 7 handoff

Phase 7 is Integrity Monitor.

Integrity Monitor should remain a separate component. It may consume scanner-style safe reporting primitives later, but Phase 7 baseline storage and continuous change detection must not be folded into the Phase 6 one-shot posture scanner.
