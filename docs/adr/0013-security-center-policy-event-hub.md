# ADR 0013 — Security Center persistent policy and event hub

## Status

Accepted for Phase 20.

## Context

Security Center originally retained activity only in memory. Phase 18 added durable integrity events outside Security Center and Phase 19 added controlled firewall mutation through the privileged service. The suite now needs one Security Center history/notification model without introducing a generic automation bus or duplicating privileged control paths.

## Decision

Security Center owns a local persistent event hub containing bounded redaction-safe event metadata and acknowledgement state. Notifications are a filtered view of the same records, not another queue.

Security Center also owns a separate bounded component-health history that stores normalized state transitions and aggregate counts. Equivalent consecutive health snapshots are deduplicated.

A narrow suite policy in Security Center settings controls event retention, notification severity threshold, health-history retention, and the already-existing signed update channel. It cannot contain executable commands or arbitrary action definitions.

Event and health-history files use versioned schemas, fixed size/count limits, crash-recoverable replacement, and invalid-state quarantine. Failure to initialize durable storage is an initialization error rather than a silent downgrade to memory-only Phase 20 behavior.

The existing privileged service is unchanged. Event acknowledgement never authorizes remediation or privileged mutation.

## Consequences

- activity and acknowledgement survive Security Center restart;
- warning/critical events can remain pending until explicitly acknowledged;
- health transitions can be reviewed without storing sensitive diagnostics;
- bounded local persistence replaces the old in-memory-only activity history;
- Phase 20 remains local-first and normal-user;
- scheduled actions/remediation remain deferred to separately reviewed future phases.
