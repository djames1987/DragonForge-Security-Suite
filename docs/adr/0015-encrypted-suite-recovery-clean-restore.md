# ADR 0015 — Encrypted Suite Recovery and Clean-Machine Restore

## Status

Accepted for Phase 22.

## Context

DragonForge already had a verified encrypted backup product, but disaster recovery requires more than copying arbitrary files. A machine-migration feature must distinguish authoritative persistent state from runtime credentials and transient files, support format evolution, refuse unsafe merge semantics, and provide a recovery path for interrupted state replacement.

The suite now contains several versioned JSON state files and OS-bound secrets. Blindly copying every file from one machine to another could migrate stale Agent credentials, lock files, quarantined state, logs, or machine-specific trust material.

## Decision

Phase 22 introduces a dedicated `.dfrecovery` container inside the existing Backup & Recovery engine.

The recovery container:

- uses Argon2id + AES-256-GCM;
- authenticates all metadata and file bytes;
- stores logical `config` / `data` namespaces instead of source absolute paths;
- records a separate logical schema version;
- writes only the current schema;
- explicitly migrates known older schemas in memory;
- rejects unknown future schemas;
- excludes runtime/session/transient artifacts;
- restores only into missing or empty roots;
- stages restored trees before finalization.

Configuration-only and full-suite packages share the same format and validation path.

Phase 22 also provides a narrow JSON backup-repair operation. Only valid `.json.bak` files under the discovered DragonForge roots can replace a missing or malformed JSON primary. Corrupt primaries are quarantined before promotion.

## Why not reuse .dfbackup directly?

Phase 9 backups preserve user-selected source metadata and restore under a generic destination tree. Disaster recovery needs logical suite namespaces, cross-machine root remapping, transient-state exclusions, schema migration, and clean-root import semantics. Giving those meanings to old `.dfbackup` files would make existing backup behavior ambiguous.

## Why not copy the entire AppData trees?

The suite data roots contain runtime and diagnostic artifacts that should not cross machines. Agent session credentials in particular must be regenerated. A curated logical package is safer than filesystem cloning.

## Why no overwrite/merge restore?

Merging recovery data into an active or non-empty suite state creates difficult partial-upgrade and rollback problems. Phase 22 instead establishes a deterministic clean-machine contract. Existing state must be backed up or moved explicitly before import.

## Why no Windows Credential Manager migration?

Those entries are intentionally OS/user-bound secret storage. Exporting them into a portable recovery package would weaken that boundary. The destination machine must re-establish those secrets through the owning feature's normal authentication/enrollment flow.

## Consequences

- machine migration is explicit and auditable;
- package readers have a real schema migration point;
- future incompatible schemas fail closed;
- transient and machine-bound state is regenerated instead of cloned;
- disaster restore requires a clean destination root;
- users may need to re-authenticate or re-enroll OS-bound features after migration;
- Phase 23 can focus on privacy/accessibility/product polish without carrying unresolved restore semantics.
