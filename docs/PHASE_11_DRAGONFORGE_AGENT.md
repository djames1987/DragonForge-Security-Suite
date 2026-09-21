# Phase 11 — DragonForge Agent

## Status

**Implementation Complete — Local Verification Pending**

Phase 11 introduces the first real DragonForge Agent runtime: a per-user background process with authenticated local IPC and Security Center integration.

## Delivered

- dedicated `dragonforge-agent` runtime/client crate;
- dedicated `dragonforge-agent` background executable under `services/dragonforge-agent/`;
- Security Center live Agent health/status integration;
- exact-sibling Agent launch from Security Center;
- per-user loopback TCP transport bound only to `127.0.0.1`;
- random 256-bit session credential generated at Agent startup;
- HMAC-SHA256 authentication for both requests and responses;
- request timestamps with a 60-second freshness window;
- random request nonces and bounded replay rejection;
- request source/destination validation through the Phase 2 `LocalIpcPolicy`;
- bounded wire-message size and read/write timeouts;
- single-instance startup lock and stale runtime cleanup;
- authenticated health endpoint exposing PID, uptime, protocol state, and narrow capability names;
- malformed/unauthenticated client failures isolated to the connection rather than terminating the Agent;
- end-to-end test using an isolated runtime directory and a real authenticated client/server health exchange;
- Phase 11 CI and Windows local verification tooling;
- all-suite build extended to include the Agent executable.

## Runtime files

The per-user Agent runtime directory stores:

- `agent-runtime.json` — versioned local descriptor containing protocol, loopback port, PID, and startup time;
- `agent-session.key` — random session credential used to authenticate the current Agent session;
- `agent.lock` — single-instance startup/lifetime guard.

On Windows these files live below the current user's DragonForge local application-data directory and inherit that user's filesystem access controls.

The session credential is regenerated each time the Agent starts and is not embedded in the runtime descriptor.

## IPC authentication

Security Center health requests include:

- protocol major/minor version;
- random request ID;
- fixed authenticated source identity;
- fixed action name;
- current timestamp;
- random nonce;
- HMAC-SHA256 authentication tag.

The Agent:

1. validates protocol compatibility;
2. validates the fixed Security Center source and Agent destination through Phase 2 policy;
3. validates nonce encoding/length;
4. rejects stale timestamps;
5. verifies the request HMAC in constant-time through the HMAC library;
6. rejects replayed nonces;
7. dispatches only recognized fixed actions;
8. signs the response with the same session credential.

The Phase 11 command surface contains only the `health` action.

## Capability boundary

The Agent currently reports:

- `health`;
- `authenticated-ipc`;
- `background-lifetime`.

Phase 11 deliberately does **not** add firewall mutation, process termination, file quarantine, packet filtering, registry remediation, privileged persistence monitoring, or arbitrary command execution.

Those capabilities require separate product requirements, authorization policy, tests, and—where needed—a narrow elevated Windows service boundary.

## Privilege model

The Phase 11 Agent is a **normal-user, per-user background process**. Launching it from Security Center allows it to continue after the UI closes, but it is not installed as a Windows service and does not request elevation.

The session credential protects the Agent protocol from unauthenticated local network clients and accidental/spoofed protocol traffic. It is not a defense against a process that has already fully compromised the same user account and can read that user's Agent runtime files.

A future elevated service design must use stronger OS-backed service identity/ACL controls and keep privileged operations separate from this per-user command surface.

## Verification

Run on Windows:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase11-agent-tests.ps1
```

The verifier covers formatting, compile checks, strict Clippy, Agent protocol/runtime tests including the authenticated client/server round trip, all prior suite regression tests, JavaScript checks, and all current suite executable builds.

A passing local verifier log and SHA-256 sidecar are required before Phase 11 is marked **Verified Complete**.
