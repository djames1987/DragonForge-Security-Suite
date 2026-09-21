# Phase 3 — Security Center

## Status

**Complete**

Phase 3 turns the Security Center from a command-line foundation into the first unified DragonForge Security Suite desktop dashboard.

## Delivered application

The Security Center now runs as a Tauri desktop application from:

- `apps/security-center/`

It uses the suite's established dark charcoal / DragonForge orange visual language and exposes five primary views:

1. Overview
2. Components
3. Activity
4. Settings
5. About

## Overview

The Overview page provides:

- aggregate suite health;
- counts for active, integrated, and planned/future components;
- a compact component registry;
- recent local activity;
- explicit DragonForge Agent status.

Planned components do not create a false warning condition.

## Component registry

Phase 3 distinguishes component lifecycle state rather than pretending every roadmap product is installed.

Current states include:

- Active
- Integrated
- Planned
- Unavailable
- Attention

At the Phase 3 baseline:

- Security Center is Active.
- Password Manager is Integrated.
- DragonForge Agent is Unavailable / not installed.
- File Vault, Authenticator, Security Scanner, Integrity Monitor, Network Guard, Backup & Recovery, and Secure Share are Planned.

## Password Manager orchestration

Security Center can request launch of the Password Manager.

For safety, the launcher:

- resolves the current Security Center executable;
- constructs the exact expected Password Manager executable path in the same directory;
- does not search PATH;
- does not accept a user-supplied executable path;
- refuses the request if the sibling binary does not exist.

This is intentionally narrow orchestration, not a general process launcher.

## Activity model

Security Center maintains a bounded in-memory activity store.

Properties:

- newest events are displayed first;
- capacity is configurable from 50 to 2,000 events;
- event records use the shared Phase 2 event metadata;
- summaries are intended to be safe for UI/log display;
- clearing activity only clears the current in-memory dashboard history.

Persistent forensic/event storage is not implemented in Phase 3.

## Settings

Settings are versioned JSON stored beneath the current user's Security Center configuration directory resolved by `SuitePaths`.

Phase 3 settings include:

- start-on-overview preference;
- in-memory event retention count;
- diagnostic identifier preference.

Settings must never contain:

- master passwords;
- account secrets;
- encryption keys;
- sync tokens;
- recovery material;
- vault content.

Invalid settings fall back to safe defaults and generate a local warning event rather than preventing the dashboard from opening.

## Safe logging

Security Center initializes a local append-only diagnostic log beneath its component data directory.

The logger:

- uses the Phase 2 `LogPolicy`;
- sanitizes control characters;
- bounds public log fields;
- accepts only explicitly public/safe messages from the Security Center code path.

The Phase 2 `Secret<T>` formatting guard remains available to future components but is not a replacement for zeroized secure memory.

## DragonForge Agent boundary

The privileged DragonForge Agent does not exist yet.

Phase 3 deliberately reports:

- available: false
- transport: none
- state: unavailable

The dashboard validates its future Security Center → Agent request shape against the Phase 2 fail-closed IPC policy, but it does not create a socket, pipe, credential, or fake authenticated connection.

A future Agent phase must independently verify peer identity at the operating-system transport boundary before constructing an authenticated `PeerContext`.

## Backend modules

`apps/security-center/src/` contains:

- `agent.rs` — Agent status boundary.
- `events.rs` — bounded dashboard event store.
- `logging.rs` — safe local diagnostic logging.
- `model.rs` — component registry and health summary.
- `orchestration.rs` — strict Password Manager launch.
- `settings.rs` — versioned persistent settings.
- `state.rs` — synchronized application state.
- `lib.rs` — Tauri command surface.
- `main.rs` — desktop entry point.

## Tauri command surface

Phase 3 exposes a narrow command surface:

- `dashboard_snapshot`
- `refresh_health`
- `recent_events`
- `clear_events`
- `get_settings`
- `save_settings`
- `agent_status`
- `launch_password_manager`

The webview does not directly perform filesystem or process operations.

## Test coverage

Rust tests cover:

- registry population;
- planned-component health behavior;
- bounded event retention;
- future Agent policy compatibility;
- explicit Agent unavailable state;
- settings defaults and round-trip persistence;
- settings validation;
- log newline sanitization;
- application snapshot behavior;
- persisted settings updates;
- strict Password Manager sibling-path resolution.

The dashboard JavaScript is syntax-checked with Node.

## Local verification

On Windows:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase3-security-center-tests.ps1
```

The script runs:

- `cargo fmt --all --check`
- Security Center/core `cargo check`
- Clippy with warnings denied
- Rust tests
- dashboard JavaScript syntax validation
- Security Center build

It creates a timestamped log and SHA-256 checksum in `test-logs/`.

## Phase 4 handoff

Phase 4 can add File Vault as the next real suite application.

Security Center already has a planned File Vault registry entry and can later consume File Vault health/events without changing the Phase 3 dashboard architecture.
