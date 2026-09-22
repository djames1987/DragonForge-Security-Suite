# Phase 12.6 — UX Consistency

## Status

**Implementation Complete — Local Verification Pending**

Phase 12.6 aligns the DragonForge desktop applications around one visible and behavioral suite contract without collapsing their product-specific security workflows.

## Suite UX contract

- Product identity uses the DragonForge Security Suite name consistently.
- Security Center is the recommended suite entry point; direct component launch remains supported for troubleshooting and advanced use.
- Current package version is visible as Version 0.1.0 across the desktop surfaces.
- Current suite milestone is visible as Suite Phase 12.6 instead of presenting an old component implementation phase as the current suite phase.
- Historical component-phase descriptions remain in About copy where they explain architecture or security scope.
- Primary actions remain visually distinct from secondary actions.
- Destructive or replacement operations use explicit confirmation before execution.
- Disabled controls use a common reduced-opacity/not-allowed treatment.
- Keyboard focus is visibly indicated for buttons, inputs, textareas, selects, and tabindex-enabled controls.
- About/version metadata uses a common subdued suite-version treatment.
- Status wording favors Ready, Active, Attention, Unknown, Unavailable, Locked, and Unlocked according to the component's actual state.
- Security limitations and non-claims remain explicit and are not hidden for visual consistency.

## Sensitive-action confirmation alignment

Phase 12.6 keeps existing confirmations for Password Manager deletion, Authenticator account deletion, and Integrity Monitor baseline replacement, and adds confirmations for:

- File Vault extraction;
- Backup & Recovery restore;
- Secure Share attachment extraction;
- Authenticator recovery-code replacement.

These prompts describe the operation and its overwrite/replacement consequence before invoking the native command.

## Accessibility baseline

Every desktop application stylesheet now includes the same Phase 12.6 baseline for:

- visible focus on keyboard-navigable controls;
- consistent disabled-control feedback;
- consistent suite-version metadata styling.

## Agent terminology

Pre-Phase-11 text that described the DragonForge Agent as a future component was removed from current application UI. Individual applications still accurately state when a capability is not provided or when Agent lifecycle support does not imply continuous monitoring/enforcement.

## Scope boundaries

Phase 12.6 does not:

- redesign cryptographic workflows;
- change vault, backup, share, or authenticator formats;
- add privileged enforcement;
- make visibility-only applications claim protection they do not provide;
- remove direct component launch;
- replace native application-specific security warnings with generic wording.

## Verification

Run:

    powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase12.6-ux-consistency-tests.ps1

The verifier checks Rust formatting/compile health, JavaScript syntax for all nine desktop applications, current version/suite-phase metadata, shared stylesheet accessibility markers, sensitive-action confirmation wiring, stale Agent wording, and the required Phase 12.6 documentation.
