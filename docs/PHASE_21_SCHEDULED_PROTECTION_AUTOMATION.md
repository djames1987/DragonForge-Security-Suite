# Phase 21 — Scheduled Protection & Automation

**Status: Implementation Complete — Local Verification Pending**

Phase 21 adds restart-persistent, capability-scoped scheduling to the normal-user DragonForge Agent. It deliberately does not introduce a generic task runner, shell surface, arbitrary executable launch API, or silent privileged remediation.

## Delivered

- versioned persistent Agent automation state;
- fixed approved jobs only:
  - scheduled Security Scanner posture scan;
  - scheduled explicit Integrity Monitor comparison;
  - scheduled encrypted-backup reminder;
- opt-in defaults for every automation job;
- interval policy bounded from 15 minutes through 7 days;
- next-run and last-run tracking across Agent restarts;
- overdue-job recovery when the Agent was not running at the scheduled time;
- bounded retry handling for execution failures with a one-minute retry delay and three short retries before returning to the normal cadence;
- bounded 250-event automation history with monotonic IDs;
- crash-recoverable state replacement using temporary/backup files;
- malformed automation state quarantine to a sibling `.invalid` file;
- stale cross-process automation lock recovery;
- Security Scanner execution on a dedicated worker thread so scheduled scans do not block authenticated Agent IPC;
- Phase 21 execution results routed into the Phase 20 persistent Security Center event/notification hub;
- Security Center Automation page with enable/disable, bounded interval, Run now, next/last run, failure count, and recent execution history;
- Agent CLI diagnostics for automation status/configuration/manual execution;
- Agent health capability advertisement for `scheduled-protection-automation`;
- dedicated Phase 21 verifier, ADR, and CI coverage.

## Approved automation capabilities

Phase 21 accepts only three typed job identifiers:

- `security_scan`
- `integrity_check`
- `backup_reminder`

Unknown identifiers such as `exec`, `shell`, `powershell`, `cmd`, executable paths, script bodies, command arguments, or environment payloads are rejected. The scheduler has no generic command field.

### Security Scanner

The Agent calls the existing read-only Security Scanner engine. Only the aggregate pass/attention/unknown/informational counts are retained in automation history. Scanner evidence payloads are not copied into scheduler state.

### Integrity check

The Agent invokes an explicit comparison using the existing Phase 18 sealed baseline and suppression policy. The Phase 18 background cadence remains intact; Phase 21 adds an explicit forced-check path for a scheduled or manual protection job.

If continuous integrity monitoring is not configured, the automation run returns an attention result rather than silently creating or replacing a baseline.

### Encrypted backup reminder

Phase 21 intentionally does not persist a Backup & Recovery password and does not create an unattended archive whose decryption credential would need to be stored by the scheduler.

The backup job therefore raises a persistent actionable notification telling the user that an encrypted backup is due and to open Backup & Recovery. This preserves the existing explicit password boundary while still providing restart-persistent recurring backup policy/reminders.

A future unattended encrypted-backup design would require a separately reviewed credential-storage/recovery model rather than weakening the existing backup format.

## Persistence and bounds

State file: `scheduled-automation-v1.json` under the current user's Agent data directory.

Hard limits:

- schema version: 1;
- exactly three approved job policies;
- interval: 15 through 10,080 minutes;
- retained history: maximum 250 events;
- state file: maximum 1 MiB;
- event summary: maximum 512 single-line characters;
- retry streak: bounded to the retry policy.

State writes use temporary + backup replacement. A missing primary file recovers from `.bak`. Malformed state is quarantined to `.invalid` before a safe default state is created.

A create-new lock coordinates Agent scheduler ticks with Security Center policy/manual-run operations. A lock older than 15 minutes is treated as stale and may be recovered, avoiding overlap with legitimately long scanner or integrity runs.

## Retry and missed-job behavior

When an enabled job is due, the Agent runs it at the next scheduler poll. If the scheduled time was missed by more than one configured interval, the recovered execution is identified in its summary and successful jobs may be marked `missed_recovered`.

A retryable execution error schedules a one-minute retry. After three short retries the scheduler returns to the job's configured cadence rather than retrying without bound.

Attention findings and backup reminders are successful scheduler executions, not transport failures; they become Warning notifications in Security Center.

## Event Hub integration

Security Center retains a hidden automation-event cursor alongside the existing Phase 18 integrity cursor. New scheduler events are imported once into the Phase 20 durable event hub.

Severity mapping:

- `success` -> Info;
- `missed_recovered` -> Notice;
- `attention` / `action_required` -> Warning;
- `failed` -> Critical.

Acknowledging a notification changes only notification workflow state. It does not modify the automation policy or count as authorization for any other action.

## Security boundary

Phase 21 remains current-user and normal-user.

It does not:

- change the Phase 17/19 privileged-service command set;
- add arbitrary task execution;
- add PowerShell/cmd/script submission;
- accept executable paths or free-form arguments;
- terminate processes;
- quarantine files;
- silently alter firewall policy;
- silently create or replace integrity baselines;
- persist backup passwords;
- silently install updates.

Privileged actions remain behind their existing authenticated typed boundaries and are not scheduler capabilities in Phase 21.

## Verification

Run on Windows:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase21-scheduled-protection-automation-tests.ps1
```

The verifier produces a transcript and SHA-256 sidecar under `test-logs`. Phase 21 remains **Implementation Complete — Local Verification Pending** until the authoritative Windows run reaches its PASS marker and its sidecar is independently verified.
