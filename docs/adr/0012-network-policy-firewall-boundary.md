# ADR 0012 — Typed Windows Firewall mutation boundary

## Status

Accepted for Phase 19.

## Context

Phase 8 established read-only Network Guard visibility. Phase 17 established an authenticated privileged Windows service with all mutation capabilities disabled. Phase 19 needs controlled firewall enforcement without introducing a generic elevated shell, raw firewall-rule language, or direct UI access to the privileged named pipe.

## Decision

DragonForge exposes four fixed privileged firewall operations: status, apply, remove, and rollback. Requests use typed structures containing an absolute executable path, SHA-256, bounded display name, allow/block action, and a rollback token only when required.

Only `FirewallPolicyMutation` is enabled by the Phase 19 capability policy. Every other privileged capability remains denied.

Network Guard does not open the privileged pipe. It launches only the exact sibling DragonForge Agent using fixed CLI verbs. The existing Phase 17 pipe boundary then verifies the Agent's operating-system peer identity, exact executable path, Authenticode signature, and pinned publisher.

The privileged service independently re-hashes the target executable and uses native Windows Firewall COM interfaces. It creates outbound, all-profile, application-scoped rules only in the DragonForge rule namespace/group. Before mutation it requires Windows to report that local firewall policy changes are effective; Group Policy/local-policy override states fail closed, while read-only status remains available.

Service-owned state tracks managed rules and bounded rollback records. Rules that collide with the deterministic DragonForge name without a corresponding ownership record are not overwritten.

## Consequences

- the UI cannot supply arbitrary elevated firewall commands;
- firewall mutations are auditable through the existing privileged-service audit path;
- changed executables fail identity verification before mutation;
- unrelated Windows Firewall rules are outside the mutation surface;
- rollback is narrow and rule-specific rather than a global firewall restore;
- administrator or Group Policy changes outside DragonForge may override or remove managed rules and are reported through status rather than silently treated as owned state;
- richer WFP enforcement or port/address policy requires a separate reviewed expansion.
