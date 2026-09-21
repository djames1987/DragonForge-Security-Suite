# Phase 7 — Integrity Monitor

## Status

**Verified Complete**

Phase 7 adds a Windows-first, local-only integrity baseline and on-demand comparison application.

## Delivered

- dedicated `dragonforge-integrity-monitor` engine crate;
- dedicated Tauri desktop application under `apps/integrity-monitor/`;
- versioned local baseline format;
- SHA-256 fingerprints for monitored values and selected files;
- monitoring of user/common Startup folders;
- monitoring of Windows `Run` / `RunOnce` registry persistence;
- service configuration fingerprinting;
- scheduled-task configuration fingerprinting;
- Windows hosts-file fingerprinting;
- selected UAC, Remote Desktop, SMB1, and firewall configuration fingerprinting;
- Added / Removed / Changed comparison results;
- bounded collection and baseline-size limits;
- symlink rejection for baseline storage and monitored startup files;
- explicit baseline creation and replacement;
- Security Center sibling-app integration;
- CI and Windows verification tooling.

## Trust boundary

Phase 7 is an on-demand integrity monitor. It does not claim continuous protection while its UI is closed.

The native engine uses fixed Windows probes only. The webview cannot submit PowerShell, registry paths, service names, scheduled-task names, arbitrary filesystem roots, or remediation commands.

The baseline stores monitored identifiers and fingerprints. It does not persist registry command contents, service executable command lines, scheduled-task action contents, or monitored file contents.

A change is evidence for review, not proof of compromise. Legitimate software installation, updates, administration, and user configuration can all create integrity changes.

## Baseline lifecycle

The first baseline should be created when the user considers the system state trustworthy. Replacing the baseline is an explicit action because it resets the comparison reference point.

The baseline is stored in the Integrity Monitor component data directory as `baseline-v1.json`. Writes are staged through a temporary file and replacement keeps a temporary backup until the new baseline is in place.

## Monitored surfaces

Phase 7 covers:

- Startup folder regular files, recursively and with bounded depth/count;
- current-user and local-machine `Run` / `RunOnce` values;
- Windows service start mode, account, and image command fingerprint;
- scheduled-task state, principal, run level, and action fingerprint;
- Windows hosts file;
- selected security configuration values.

Collection failures become warnings and do not silently turn into an unchanged result.

## Not included in Phase 7

- continuous background monitoring;
- kernel callbacks or drivers;
- automatic quarantine or remediation;
- arbitrary user-selected folders;
- tamper-resistant privileged baseline storage;
- event-log or ETW streaming;
- remote reporting.

Those areas require later Agent/trust-boundary work.

## Verification

Run on Windows:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase7-integrity-monitor-tests.ps1
```

Authoritative Windows verification passed on 2026-09-21 using `dragonforge-phase7-integrity-monitor-20260921-123231.log`.

SHA-256: `DBE01791CEB551EC6CFD880F89858768F5402C8A154B3DC382B97A8ED60C0063`.

## Phase 8 handoff

Phase 8 is Network Guard, focused on network visibility and eventually carefully scoped enforcement.
