# ADR 0016 — Suite-Wide Privacy and Accessibility UI Contract

## Status

Accepted for Phase 23.

## Context

DragonForge's desktop applications evolved across multiple phases. They share a visual language, but accessibility behavior, dynamic-state semantics, privacy guidance, and branding integration were still largely app-specific.

A security product should not require users to infer whether diagnostics are uploaded, whether keyboard focus is visible, or whether high-contrast and reduced-motion preferences are respected. These are product trust properties even though they do not belong in cryptographic or privileged-service code.

## Decision

Phase 23 introduces a small, presentation-only `phase23.js` and `phase23.css` layer in every desktop webview.

The layer owns:

- accessibility landmarks and skip navigation;
- keyboard-visible focus;
- navigation keyboard movement;
- screen-reader status announcements;
- reduced motion;
- forced-colors/high contrast;
- scalable control sizing;
- consistent empty/loading/error/status presentation;
- new shared-shell localization keys.

Security Center additionally owns suite-level first-run and persistent privacy/accessibility guidance.

Approved repository branding assets are referenced from Tauri application bundle metadata. Existing Password Manager branding is preserved.

## Privacy boundary

The accessibility layer performs no fetch, IPC, filesystem, process, or privileged operation.

It must not:

- send telemetry;
- load remote UI resources;
- announce secrets;
- change security decisions;
- persist anything except the local first-run acknowledgement in Security Center.

## Localization boundary

Only new shared Phase 23 shell strings are catalogued in this phase. Existing product copy stays English until a future locale pack explicitly migrates it. This prevents automatic translation from changing security-sensitive wording without review.

## Branding boundary

Canonical logos remain in `assets/branding/logos`. Application configs reference them rather than creating untracked duplicates. Password Manager keeps its migrated product icon.

Horizontal logo source artwork is not programmatically cropped or recolored in Phase 23.

## Consequences

- every desktop application receives the same baseline accessibility behavior;
- privacy expectations are visible to users;
- future UI code can use one state/announcement contract;
- high-contrast/reduced-motion behavior is testable by source invariants;
- product icons are tied to reviewed repository assets;
- Phase 24 can audit hands-on accessibility and privacy behavior against an explicit contract.
