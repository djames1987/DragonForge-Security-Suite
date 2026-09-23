# Phase 20 — Security Center Policy & Event Hub

**Status: Implementation Complete — Local Verification Pending**

Phase 20 turns Security Center's original in-memory activity list into a persistent, bounded suite event hub and adds a narrow coordinated policy layer for Security Center presentation/retention behavior. It does not introduce generic automation, arbitrary remediation, or a new privileged command surface.

## Delivered

- versioned persistent Security Center event hub;
- monotonic event IDs and normalized component/kind/severity/status metadata;
- bounded redaction-safe summaries with no secret-bearing arbitrary payload field;
- persistent open/acknowledged event state with acknowledgement timestamps;
- notification center derived from unacknowledged events at or above a configured severity threshold;
- acknowledge-one and acknowledge-all workflows;
- persistent bounded component-health history;
- health-history deduplication so unchanged dashboard refreshes do not create noise;
- coordinated suite policy for notification severity and health-history retention;
- existing event-retention and update-channel settings retained;
- crash-recoverable .bak replacement for event and health-history state;
- invalid persistent state quarantined to .invalid rather than silently disabling persistence;
- existing Integrity Monitor continuous alerts routed into the persistent event hub;
- Security Center dashboard snapshot includes current notifications and recent health history;
- Notification Center and policy controls in the Security Center UI;
- dedicated Phase 20 Windows verifier, ADR, and CI coverage.

## Event schema

Each retained event contains only bounded Security Center metadata: monotonic numeric ID, timestamp, component identifier, event kind, normalized severity, stable code, redaction-safe summary, status (open or acknowledged), and optional acknowledgement timestamp.

Event codes are capped at 160 characters and summaries at 512 characters. Control characters/newlines are removed before persistence. Phase 20 does not add an arbitrary JSON payload, command field, environment capture, process arguments, file contents, credentials, vault data, or secrets to the event schema.

## Persistence and bounds

Event hub: schema version 1; configured retention 50 through 2,000 events; hard event capacity 2,000; file-size bound 2 MiB; crash-recoverable temporary/backup replacement.

Health history: schema version 1; configured retention 10 through 500 snapshots; hard capacity 500; file-size bound 1 MiB; unchanged health snapshots are deduplicated; crash-recoverable temporary/backup replacement.

Malformed/incompatible state is quarantined to an .invalid sibling before a fresh persistent store is created. The system does not silently downgrade Phase 20 to memory-only storage.

## Notification center

Notifications are a view over the persistent event hub, not a separate duplicate store. The suite policy chooses the minimum visible severity: debug, info, notice, warning, or critical.

Only events with status open and severity at or above the policy threshold appear as pending notifications. Acknowledgement changes event state in the same durable record and therefore survives Security Center restart.

Acknowledgement is informational workflow state. It does not suppress future independent security events and does not change the underlying component configuration.

## Component-health history

Security Center records normalized suite/component state when a dashboard snapshot is produced. Consecutive equivalent snapshots are deduplicated. History contains state identifiers and aggregate counts only; it does not persist component logs, diagnostics, process identifiers, file paths, or event bodies.

## Coordinated suite policy

Phase 20 policy coordinates persistent event retention, minimum notification severity, component-health history retention, and the already-existing signed update channel selection.

It does not execute scripts, run arbitrary scheduled tasks, silently remediate findings, alter firewall rules, terminate processes, quarantine files, change Integrity Monitor suppression rules, or mutate Password Manager/security-product state. Phase 21 remains the separately reviewed scheduled-protection and automation phase.

## Security boundary

Phase 20 is normal-user Security Center state management. It does not expand the Phase 17/19 privileged service, add privileged commands, or bypass the signed-Agent boundary. Event acknowledgement is never treated as authorization to perform a privileged action.

## Verification

Run on Windows:

powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase20-security-center-policy-event-hub-tests.ps1

The verifier writes a transcript and matching SHA-256 sidecar under test-logs. Phase 20 remains **Implementation Complete — Local Verification Pending** until the authoritative Windows verifier reaches its PASS marker and the sidecar is independently verified.
