# Phase 16 — Windows Security Boundary Foundation

**Status: Implementation Complete — Local Verification Pending**

Phase 16 defines and regression-tests the security contract for the future privileged DragonForge Windows service. It intentionally does **not** introduce elevation or privileged mutation.

## Boundary identity

The future service identity is fixed as:

- service name: `DragonForgePrivilegedService`;
- display name: `DragonForge Privileged Service`;
- service account: `NT SERVICE\DragonForgePrivilegedService`;
- local named pipe: `\\.\pipe\DragonForgePrivilegedService-v1`.

The service is intended to use its Windows virtual service account and service SID rather than LocalSystem for ordinary operation. Phase 17 must request only the specific Windows rights required for implemented capabilities.

## IPC trust model

The per-user DragonForge Agent is the only intended privileged-service caller.

Before dispatch, the future service must verify:

1. local named-pipe transport;
2. operating-system peer identity;
3. exact installed `dragonforge-agent.exe` path;
4. valid Authenticode signature;
5. exact expected DragonForge publisher subject;
6. protocol major compatibility;
7. bounded message size;
8. request freshness and replay protection;
9. a fixed allow-listed command;
10. capability authorization for the requested operation.

Authentication is not authorization. Passing caller identity checks does not grant a capability.

## Named-pipe DACL

Phase 17 must create the named pipe with an explicit DACL. The intended policy is:

- LocalSystem: administrative/service-control access as required by Windows;
- DragonForge service SID: full pipe ownership/service access;
- Administrators: service administration only, not an implicit application caller identity;
- interactive/authenticated users: connect only if required for Agent IPC, with the service still required to verify the client process identity, exact executable path, Authenticode signature, and publisher before dispatch;
- network/anonymous identities: denied.

The service must not rely on an inherited permissive DACL.

## Command model

Phase 16 defines only:

- `health`;
- `describe-policy`.

Neither command is privileged.

The boundary explicitly prohibits generic commands such as:

- shell;
- exec;
- run;
- cmd;
- PowerShell;
- arbitrary executable launch;
- arbitrary registry command;
- arbitrary firewall command.

Future privileged operations must be represented as typed, narrow commands with dedicated request schemas, validation, authorization policy, tests, and audit events.

## Capability policy

Phase 16 models future capability families but authorizes none:

- firewall policy mutation;
- protected process control;
- protected file quarantine;
- protected registry remediation;
- system integrity remediation.

Phase 17 or later phases must enable capabilities one at a time behind explicit policy. There is no wildcard capability and no "administrator" bypass.

## Executable and publisher verification

A future privileged service must resolve the expected Agent path from the trusted installation location rather than from caller input or PATH lookup.

Caller trust requires:

- exact expected path;
- valid Authenticode;
- expected publisher subject;
- verified OS peer process identity.

A correctly signed copy launched from an unexpected location is rejected by the Phase 16 contract.

## What Phase 16 does not do

Phase 16 does not:

- install or register a Windows service;
- request elevation;
- run as LocalSystem;
- mutate firewall policy;
- terminate processes;
- quarantine files;
- alter protected registry state;
- expose an arbitrary privileged command runner.

Those remain Phase 17+ work.

## Verification

Run:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase16-windows-security-boundary-tests.ps1
```

The verifier checks formatting, locked workspace compile, strict Clippy, full tests, Phase 16 policy invariants, documentation artifacts, CI wiring, and PowerShell syntax.
