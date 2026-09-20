# Password Manager Migration Plan

## Status

**M0 complete; M1/M2 in progress.**

The Password Manager remains in:
https://github.com/djames1987/DragonForge-Password-Manager

The tested Password Manager snapshot is now being imported on `migration/password-manager-1ee25e7` and normalized into the reserved suite paths.

## Why migration is deliberately deferred

Testing is complete for the selected baseline. Migration now proceeds from the frozen tested commit so repository restructuring can be verified independently from product development.

## Selected migration source snapshot

Selected tested source commit: `1ee25e751da094577a9c6bbe10af5bdca92e96ba` (2026-09-20). The Password Manager is a Rust workspace with:
- `apps/desktop`
- `apps/sync-server`
- `apps/browser-extension`
- `crates/dragonforge-crypto`
- `crates/dragonforge-vault`

Its workspace forbids unsafe Rust and targets Rust 1.85 / edition 2024.

This commit is the immutable pre-suite migration baseline. See `docs/password-manager/MIGRATION_BASELINE.md`.

## Migration gate

Do not begin migration until:

- current Password Manager testing is complete;
- the chosen source commit SHA is recorded;
- all expected automated tests pass or known failures are documented;
- any required manual Windows verification is recorded;
- working tree/repository state is clean and pushed.

## Planned path mapping

| Current Password Manager | Suite destination |
|---|---|
| `apps/desktop` | `apps/password-manager/` |
| `apps/sync-server` | `services/password-manager-sync/` |
| `apps/browser-extension` | `extensions/password-manager-browser/` |
| `crates/dragonforge-crypto` | `crates/dragonforge-crypto/` |
| `crates/dragonforge-vault` | `crates/dragonforge-vault/` |
| Password Manager docs | `docs/password-manager/` where appropriate |
| Password Manager scripts | merge into `scripts/password-manager/` or suite-level equivalents |

Final mapping may change after the source repo is re-inspected.

## History-preserving strategy

Preferred approach: preserve Password Manager commit history rather than copying only the latest files.

A migration implementation should use a history-preserving Git technique such as subtree/filter-repo based import into a dedicated migration branch, followed by path normalization.

Do not archive the original Password Manager repository until:
1. the suite copy builds;
2. original automated tests pass;
3. required manual tests pass;
4. migration is reviewed;
5. the suite becomes the canonical development location.

## Migration phases

### M0 — Snapshot
- record source commit SHA and date;
- capture automated/manual test status;
- document current workspace members and build requirements.

### M1 — Import
- create dedicated migration branch;
- import source with history;
- avoid functionality changes.

### M2 — Normalize paths
- move desktop, sync server and browser extension to suite destinations;
- update Cargo paths and scripts;
- retain package behavior and data formats.

### M3 — Verification
- run original Password Manager tests;
- run suite workspace tests;
- compare results to the pre-migration baseline;
- perform required Windows/manual verification.

### M4 — Stabilize
- fix migration-only issues;
- document any intentional path/build changes;
- merge migration PR only when green.

### M5 — Shared component extraction
Only after migration is stable:
- evaluate which crates are genuinely suite-wide;
- extract shared logging/configuration/platform abstractions first;
- treat cryptographic refactoring as a separate security-reviewed change.

## Explicit non-goals during migration

Do not:
- redesign cryptography;
- change vault formats;
- change key derivation parameters;
- change sync protocol semantics;
- rename public APIs merely for aesthetics;
- combine Password Manager and File Vault;
- introduce an antivirus engine;
- delete or archive the old repo prematurely.
