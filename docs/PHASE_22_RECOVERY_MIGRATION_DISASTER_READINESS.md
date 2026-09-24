# Phase 22 — Recovery, Migration & Disaster Readiness

**Status: Verified Complete**

Phase 22 adds a suite-owned disaster-recovery and machine-migration layer on top of the verified Phase 9 Backup & Recovery engine. The design remains local-first, password-protected, bounded, and no-overwrite.

## Delivered

- encrypted `.dfrecovery` package format with its own magic/version;
- configuration-only export/import scope;
- full-suite configuration + persistent-data migration scope;
- Argon2id + AES-256-GCM authenticated encryption;
- per-entry SHA-256 integrity verification inside the authenticated payload;
- clean-machine restore into missing or empty configuration/data roots only;
- staged two-root restore with rollback of configuration state if data finalization fails;
- exclusion of Agent runtime/session credentials and transient state;
- exclusion of locks, temporary files, backup copies, quarantined copies, logs, cache data, and support bundles;
- explicit recovery schema versioning separate from package format version;
- legacy schema v1 -> current schema v2 in-memory migration;
- future/unknown schema fail-closed behavior;
- conservative interrupted-state repair for valid `.json.bak` files;
- corrupt JSON primary quarantine to `.invalid` before a valid backup is promoted;
- bounded repair traversal and candidate counts;
- Backup & Recovery desktop UI for create, inspect, verify/migrate, clean restore, and state repair;
- Phase 22 Windows verifier, CI coverage, roadmap update, and ADR.

## Recovery package

Extension:

`.dfrecovery`

Outer format version:

`1`

Current authenticated payload schema:

`2`

The outer format controls the encrypted container framing. The payload schema controls the logical recovery manifest and can evolve independently.

The encrypted payload contains:

- recovery schema version;
- creation timestamp;
- recovery scope;
- source platform identifier;
- suite version;
- logical namespace (`config` or `data`);
- normalized relative paths;
- per-file sizes;
- per-file SHA-256 digests;
- file contents.

Original absolute source roots are not persisted into the recovery manifest.

## Recovery scopes

### Configuration

Exports the DragonForge configuration root only. This supports configuration transfer without moving application data.

### Full suite

Exports both DragonForge configuration and persistent application-data roots.

The following are deliberately excluded from migration:

- `agent-session.key`;
- `agent-runtime.json`;
- `agent.lock`;
- `*.tmp`;
- `*.lock`;
- `*.bak`;
- `*.invalid`;
- cache directories;
- log directories;
- support bundles.

These are runtime/transient/recovery artifacts rather than authoritative state and should be regenerated on the destination machine.

## OS-bound secrets and installed state

A recovery package does not attempt to clone machine/OS-bound trust material.

Examples include:

- Windows Credential Manager entries used by Password Manager sync hardening;
- privileged-service installation identity and Windows service registration;
- Authenticode trust/certificate private keys;
- installer state;
- operating-system configuration and ACL/ownership metadata.

After a machine migration, features backed by OS-bound secret storage may require re-authentication, re-enrollment, or explicit setup on the destination system.

## Clean-machine restore

Restore is intentionally conservative:

1. decrypt and authenticate the package;
2. migrate a supported legacy payload schema in memory if required;
3. validate schema, scope, namespace, path, count, and size bounds;
4. decode every entry and verify size + SHA-256;
5. require configuration/data destination roots to be missing or empty;
6. stage restored trees under randomized sibling directories;
7. finalize configuration first;
8. finalize data second for full-suite packages;
9. if data finalization fails, remove the newly restored configuration tree and recreate a previously empty configuration root when applicable;
10. never merge silently with existing DragonForge state.

Users should stop DragonForge applications and the normal-user Agent before importing into live suite roots.

## Format migration

The current writer emits schema v2.

The reader contains an explicit compatibility path for schema v1 packages. Schema v1 is converted to the current in-memory model before validation and restore. Because schema v1 predates source-platform and suite-version metadata, migrated values are represented as `legacy-unknown`.

Unknown future schemas fail closed instead of being guessed or partially interpreted.

The Phase 22 tests cover:

- current-schema round trip;
- legacy-schema migration;
- future-schema rejection;
- ciphertext tamper rejection;
- clean-machine no-overwrite behavior.

## Corrupted-state recovery

Phase 22 adds a narrow repair operation over the discovered DragonForge configuration and data roots.

It considers only files ending in:

`.json.bak`

A backup is eligible only when:

- it is at most 4 MiB;
- it parses as valid JSON;
- traversal remains inside the discovered DragonForge roots.

Behavior:

- missing primary -> valid backup promoted;
- valid primary -> left unchanged;
- malformed/oversized primary + valid backup -> primary quarantined to `.invalid`, backup promoted;
- invalid backup -> skipped.

Repair traversal is bounded to 32 directory levels and 4,096 backup candidates.

This is not a generic file-repair engine and does not attempt to interpret or rewrite arbitrary application formats.

## Security boundary

Phase 22 does not:

- restore over non-empty existing suite roots;
- accept archive paths outside the logical config/data namespaces;
- follow symbolic links;
- preserve or migrate Agent session credentials;
- migrate Windows Credential Manager secrets;
- recreate privileged-service installation state;
- install software;
- change Windows Firewall;
- change ACLs or ownership;
- execute scripts or commands from a recovery package;
- infer support for unknown future recovery schemas.

## Verification

Run on Windows:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase22-recovery-migration-disaster-readiness-tests.ps1
```

The authoritative Windows verification passed on 2026-09-23 on `DRACO` (Microsoft Windows NT 10.0.26200.0).

Verified stages:
- `cargo fmt --all --check`;
- locked Cargo metadata;
- `cargo check --workspace --all-targets --all-features --locked`;
- strict `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`;
- complete workspace tests and doc-tests;
- Phase 22 recovery regression tests for schema migration, future-schema rejection, clean-root restore, tamper rejection, transient/session exclusion, overlapping-root refusal, explicit extension enforcement, and JSON backup repair;
- Security Center and Backup & Recovery JavaScript syntax checks;
- required Phase 22 artifact/invariant checks;
- PowerShell syntax validation;
- all expected Windows suite application builds.

Verified log: `dragonforge-phase22-recovery-migration-disaster-readiness-20260923-222646.log`

Verified log SHA-256: `E51144C9ED85DB6F577D06AF49CD4AE167CFDD31BB5857E29445E6AF58655837`

The SHA-256 sidecar was independently verified before Phase 22 was marked **Verified Complete**.
