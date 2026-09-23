# ADR 0011 — Continuous integrity monitoring runtime

## Status

Accepted for Phase 18.

## Context

Phase 7 provided explicit on-demand integrity baseline creation and comparison. Phase 11 introduced the per-user DragonForge Agent, and Phase 17 introduced a separately authenticated privileged Windows service. Phase 18 needs continuous scheduling, restart persistence, bounded alert history, suppression rules, and Security Center notification without turning the privileged service into a generic execution host or overstating integrity findings as malware verdicts.

## Decision

The per-user Agent owns the Phase 18 schedule because the existing probes are read-only and already execute in the user's security context. The integrity engine owns baseline comparison, baseline sealing, suppression evaluation, and the bounded persistent event model. Security Center consumes retained unsuppressed events and presents them as review-required warnings.

The Phase 17 privileged service is not given a new Phase 18 command merely to make the architecture appear more privileged. Its fixed trust boundary remains available for future integrity capabilities that can demonstrate a real protected-access requirement.

The continuous state is versioned, bounded, persisted separately from the baseline, and contains a SHA-256 seal of the explicitly approved baseline. Unexpected baseline changes create an alert and block that comparison from accepting the modified baseline.

Suppression rules are fixed surface + key-prefix data. They do not execute user text. Suppressed events remain in retained history.

## Consequences

- monitoring survives normal Agent restarts and user logon restart behavior;
- scheduled checks continue without Security Center remaining open;
- Security Center can surface bounded integrity alerts;
- baseline changes outside the explicit workflow are detected;
- no new privileged mutation capability is introduced;
- a same-user attacker who can rewrite all local files is outside the protection claimed by the baseline seal;
- future service-owned storage or protected probes require another explicit trust-boundary review.
