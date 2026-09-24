# ADR 0014 — Capability-Scoped Scheduled Protection

## Status

Accepted for Phase 21.

## Context

DragonForge already has a normal-user background Agent, continuous integrity monitoring, a read-only Security Scanner, encrypted Backup & Recovery, a persistent Security Center event hub, and a separately authenticated privileged-service boundary.

Scheduling security work introduces a new risk: a generic scheduler can become an arbitrary local-code-execution primitive, particularly if it accepts command lines, scripts, executable paths, environment variables, or user-supplied privileged actions.

Encrypted backups introduce a separate credential problem. The existing Backup & Recovery format requires a user password. Persisting that password in scheduler state would weaken the existing protection boundary.

## Decision

Phase 21 uses a fixed enum of automation capabilities rather than a general task model.

The only scheduled jobs are:

1. Security Scanner;
2. explicit Integrity Monitor comparison;
3. encrypted-backup due reminder.

All jobs run under the current-user Agent. No Phase 21 job is a privileged-service command.

Automation policy and execution history are persisted in a bounded versioned state file with atomic replacement, backup recovery, invalid-state quarantine, a short-lived cross-process lock, and bounded retry state.

Security Scanner execution is moved to a dedicated worker thread so a potentially slow posture scan cannot block authenticated Agent IPC.

Backup scheduling is reminder-based in Phase 21. The scheduler does not store a backup password. Fully unattended encrypted backups require a separately reviewed credential-storage and recovery design.

## Rejected alternatives

### Generic command scheduler

Rejected because a string command/argument model would create an unnecessary code-execution surface and undermine the narrow Agent/privileged-service capability boundaries.

### Windows Task Scheduler as the Phase 21 policy database

Rejected because DragonForge would still need to validate and reconcile external task definitions, executable paths, arguments, ownership, drift, and privilege context. Phase 21 keeps one bounded product-owned policy format.

### Persisting the backup password in automation JSON

Rejected because this would turn the scheduler state into a decryption-secret store and materially weaken Backup & Recovery.

### Scheduling privileged firewall mutations

Rejected. Phase 19 firewall changes require explicit typed requests through the authenticated privileged service. Phase 21 does not reinterpret scheduling as standing authorization for privileged mutation.

## Consequences

- automation remains auditable and narrow;
- users must opt into each schedule;
- missed jobs recover after the Agent resumes;
- execution failures retry in a bounded manner;
- Security Center can surface automation results through the existing event hub;
- encrypted backup creation still requires explicit user interaction in Phase 21;
- future automation capabilities require code and review rather than data-only configuration.
