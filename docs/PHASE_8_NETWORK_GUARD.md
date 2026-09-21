# Phase 8 — Network Guard

## Status

**Verified Complete**

Phase 8 adds a Windows-first, visibility-only Network Guard application.

## Delivered

- dedicated `dragonforge-network-guard` engine crate;
- dedicated Tauri desktop application under `apps/network-guard/`;
- per-process TCP connection and listener inventory;
- per-process UDP endpoint inventory;
- Windows DNS client cache visibility;
- TCP/UDP/process/listener/wildcard-listener summary metrics;
- bounded probe output and result counts;
- fixed native PowerShell probes with no caller-supplied command fragments;
- safe UI rendering through DOM text nodes for probe-derived values;
- Security Center sibling-app integration;
- Phase 8 CI and Windows verification tooling;
- suite-wide `build-all-apps-for-testing.ps1` script.

## Trust boundary

Phase 8 is an observation component. It does not:

- capture packet payloads;
- inspect application secrets;
- block or redirect traffic;
- terminate connections;
- kill processes;
- create, edit, or delete Windows Firewall rules;
- request administrator elevation;
- claim persistent protection while the UI is closed.

Those enforcement responsibilities require the later DragonForge Agent so privileged operations can be narrow, authenticated, reviewable, and long-running.

## Network visibility

The Windows implementation uses fixed native probes to collect:

- TCP local/remote endpoints, state, owning PID, and process name;
- UDP local endpoints, owning PID, and process name;
- DNS client cache name, record type, data, and TTL.

Wildcard listeners such as `0.0.0.0` and `::` are highlighted as exposure context. Their presence is not treated as proof of a vulnerability.

Probe failure is surfaced as a warning rather than represented as an empty or healthy network.

## Limits

Phase 8 bounds:

- TCP/UDP rows;
- DNS cache rows;
- native probe stdout size;
- process names, addresses, DNS names, and related display fields.

The UI receives only normalized snapshot data. It cannot choose arbitrary PowerShell commands or probe parameters.

## Build all apps for testing

To compile every DragonForge desktop application into the shared Cargo target directory and verify the expected executables exist:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-all-apps-for-testing.ps1
```

For release-profile test builds:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-all-apps-for-testing.ps1 -Profile release
```

The script builds Password Manager, Security Center, File Vault, Authenticator, Security Scanner, Integrity Monitor, and Network Guard.

## Phase 8 verification

Run on Windows:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase8-network-guard-tests.ps1
```

The verifier covers formatting, compile checks, strict Clippy, tests, JavaScript syntax checks, prior-suite regression gates, and the all-apps build script.

Local Windows verification passed on 2026-09-21.

Verified log:
`dragonforge-phase8-network-guard-20260921-133952.log`

SHA-256:
`B9734604E9E3550B9742C6440FB21B5E6B418CB7E810317B6CB63AC153FA1549`

The passing run completed formatting checks, workspace compile checks, strict Clippy, Network Guard tests, prior-suite regression tests, JavaScript syntax checks, and the full seven-application desktop build.

## Phase 9 handoff

Phase 9 is Backup & Recovery, focused on encrypted and verifiable recovery for suite data and selected user data.
