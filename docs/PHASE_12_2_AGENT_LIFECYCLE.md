# Phase 12.2 — Agent Lifecycle

## Status

**Implementation Complete — Local Verification Pending**

Phase 12.2 hardens the existing normal-user DragonForge Agent lifecycle without changing its privilege boundary.

## Delivered

- Security Center automatically ensures the Agent is running when dashboard health is requested;
- exact-sibling Agent launch remains enforced;
- authenticated `shutdown` IPC action added;
- Agent protocol minor version advanced to 1;
- graceful Agent shutdown removes runtime descriptor, session key, and lock through the existing runtime guard;
- Security Center Stop Agent and Restart Agent commands;
- restart waits for authenticated reconnection before reporting success;
- crashed/stale runtime recovery retries after the existing Phase 11 stale-lock window;
- explicit manual Stop Agent suppresses same-session automatic restart until Start/Restart is requested;
- installed builds create a current-user Startup shortcut so the non-elevated Agent starts again at Windows sign-in;
- portable builds recover on demand when Security Center starts after reboot/login;
- Agent CLI adds `--stop` for authenticated graceful shutdown;
- health capabilities now advertise `graceful-shutdown` and `restartable-session`;
- Security Center lifecycle UI updated for Start, Restart, Stop, and Phase 12.2 status;
- Agent remains loopback-only, HMAC-authenticated, per-user, and non-elevated.

## Reboot and login behavior

Installed builds start `dragonforge-agent.exe --serve` from the current user's Startup folder. No Windows service, scheduled task, machine-wide startup entry, or elevation is introduced.

If the Startup launch is missed, delayed, or the Agent crashes, opening Security Center triggers on-demand recovery.

Portable builds do not modify login persistence. Security Center starts the Agent on demand from the portable sibling directory.

## Crash and stale-runtime recovery

Phase 11 already used a single-instance lock and a five-second stale-lock recovery window. Phase 12.2 integrates that behavior into Security Center:

1. health is checked;
2. Security Center starts the exact sibling Agent if unavailable;
3. it waits for authenticated health;
4. if startup still fails, it waits through the stale-lock recovery interval and retries once;
5. a recovered session must answer authenticated health before it is treated as connected.

## Manual stop semantics

An explicit Stop Agent request wins over automatic lifecycle behavior for the current Security Center process. Refreshing the dashboard after a manual stop does not immediately relaunch it. Start Agent or Restart Agent clears that suppression.

A new Security Center session returns to normal automatic-start behavior.

## Security boundary

Phase 12.2 does not add:
- administrator privileges;
- a Windows service;
- privileged firewall/process/file-remediation capabilities;
- unauthenticated shutdown;
- arbitrary command execution.

The shutdown action uses the existing authenticated IPC envelope, timestamp, nonce/replay, HMAC, and caller-authorization checks.

## Verification

Run on Windows:

~~~powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase12.2-agent-lifecycle-tests.ps1
~~~

The verifier writes a timestamped log and SHA-256 sidecar under `test-logs\`.
