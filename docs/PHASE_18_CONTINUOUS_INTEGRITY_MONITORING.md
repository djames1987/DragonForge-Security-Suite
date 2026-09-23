# Phase 18 — Continuous Integrity Monitoring

**Status: Implementation Complete — Local Verification Pending**

Phase 18 turns the Phase 7 integrity baseline engine into a restart-persistent, Agent-driven change detector. It remains intentionally detection-only: a reported change is evidence for review, not a malware verdict, and Phase 18 does not add quarantine, process termination, firewall mutation, registry remediation, or arbitrary privileged execution.

## Delivered

- restart-persistent continuous monitoring state under the DragonForge Agent data directory;
- Agent scheduler that evaluates the saved integrity baseline on a bounded interval even when Security Center is not actively polling;
- default 5-minute schedule with an allowed range of 60 seconds through 24 hours;
- baseline SHA-256 seal recorded when continuous monitoring is configured;
- automatic resealing only through the explicit baseline create/replace workflow;
- fail-safe baseline-change event when the saved baseline no longer matches its seal;
- bounded retained event history with monotonic event IDs;
- explicit per-surface key-prefix suppression rules with a fixed maximum count;
- suppressed changes retained for audit while excluded from Security Center alerts;
- Security Center ingestion of new unsuppressed integrity events as warning/security events;
- Agent capability reporting for `continuous-integrity-monitoring`;
- Agent operator commands `--integrity-status` and `--integrity-events`;
- Integrity Monitor controls for enabling/disabling continuous monitoring, selecting the interval, managing suppression rules, viewing seal state, and reviewing retained events;
- dedicated Phase 18 Windows verification tooling and CI coverage.

## Monitoring flow

1. A user creates or explicitly replaces the Phase 7 integrity baseline.
2. Continuous monitoring is configured from Integrity Monitor.
3. The monitor state stores the schedule, suppression rules, current baseline SHA-256 seal, next due time, and bounded event history.
4. DragonForge Agent wakes the monitor on a bounded polling cadence and runs a comparison only when the persisted due time has arrived.
5. If the baseline seal does not match, the Agent records a baseline-change event and does not compare against the unexpectedly modified reference.
6. If the seal is valid, the Phase 7 engine gathers the fixed Windows surfaces and compares them with the baseline.
7. Added, removed, and changed entries become bounded continuous-integrity events.
8. Suppression rules mark matching events suppressed; the events remain retained but do not create Security Center alerts.
9. Security Center reads new retained events and surfaces unsuppressed changes as warning/security events.

## Persistence and bounds

The continuous state is versioned and bounded:

- state schema version: 1;
- event history: at most 500 entries;
- suppression rules: at most 256;
- minimum interval: 60 seconds;
- maximum interval: 24 hours;
- state file maximum: 2 MiB;
- existing Phase 7 baseline limits remain unchanged.

The Agent polls the due state no more frequently than every five seconds; it does not continuously re-run the expensive Windows probes.

## Baseline protection

Phase 18 records a SHA-256 seal of the approved baseline. A baseline replacement performed through Integrity Monitor explicitly reseals the new baseline. An unexpected baseline change generates a dedicated unsuppressed event and blocks that scheduled comparison from treating the modified file as trusted.

This is tamper detection, not a claim of same-user tamper resistance. A process that already has the user's permissions and can rewrite both the baseline and Agent state could potentially replace both. Phase 18 does not introduce a new privileged-service storage or write capability merely to overstate this boundary.

## Privileged-service boundary

The verified Phase 17 privileged service remains the narrow Windows privilege boundary. Phase 18 does not expand its command surface and does not enable any privileged mutation capability. Current integrity probes are read-only and continue to run through the existing fixed Phase 7 collection paths.

If a future integrity surface genuinely requires protected read access or service-owned baseline custody, it must be introduced as a separately reviewed typed service capability rather than by adding generic shell or arbitrary file access.

## Suppression rules

A suppression rule consists of:

- one fixed `SurfaceKind`;
- a non-empty bounded key prefix.

Rules are explicit and local. They suppress notification for matching events but do not delete those events from retained history. There is no wildcard command language, regular-expression execution, script input, or arbitrary PowerShell supplied by the user.

## Security Center alerting

Security Center consumes new retained events by monotonically increasing event ID. Unsuppressed changes are surfaced as:

- component: Integrity Monitor;
- kind: Security;
- severity: Warning;
- code: `integrity-monitor.change-detected`.

The alert summary contains only the already bounded integrity surface/key metadata and change type. It does not include monitored command contents or file contents.

## Non-claims

Phase 18 does not claim:

- that an integrity change is malware;
- anti-malware detection or classification;
- kernel tamper resistance;
- same-user protection of every local state file;
- real-time filesystem interception;
- packet/network enforcement;
- process termination;
- quarantine;
- registry remediation;
- firewall policy mutation.

Phase 19 remains the planned network/firewall enforcement phase.

## Verification

Run on Windows:

`powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase18-continuous-integrity-tests.ps1`

The verifier writes a transcript and a matching SHA-256 sidecar under `test-logs\`. Phase 18 must remain **Implementation Complete — Local Verification Pending** until that authoritative Windows run passes and the sidecar is independently checked.
