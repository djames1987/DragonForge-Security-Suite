# Password Manager Migration Record

## Status
Imported into the DragonForge Security Suite migration branch. Regression verification is required before merge.

## Frozen source baseline
- Repository: https://github.com/djames1987/DragonForge-Password-Manager
- Commit: `1ee25e751da094577a9c6bbe10af5bdca92e96ba`
- Tree: `346e5707717f8b9d2b2d51aa72b7f9d9bbc1477f`
- Pre-migration status: Phase 11 Windows release verification passed.

## Path mapping
- `apps/desktop/` → `apps/password-manager/`
- `apps/sync-server/` → `services/password-manager-sync/`
- `apps/browser-extension/` → `extensions/password-manager-browser/`
- `crates/dragonforge-crypto/` → unchanged
- `crates/dragonforge-vault/` → unchanged
- `docs/` → `docs/password-manager/`
- `scripts/` → `scripts/password-manager/`

## Migration-only edits
No cryptographic algorithm, vault format, key derivation, sync protocol, account recovery behavior, or product behavior was intentionally changed.

Edits are restricted to workspace membership and repository-relative path changes required by the new suite layout. PowerShell scripts were updated to locate the suite repository root from their new namespaced directory.

## History
The standalone repository remains the authoritative historical Git record for development prior to this migration. This suite commit records the exact frozen source SHA used for the import.
