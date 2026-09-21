# Password Manager Migration Plan

## Status
**Imported to migration branch; verification pending.**

Frozen source baseline: `djames1987/DragonForge-Password-Manager@1ee25e751da094577a9c6bbe10af5bdca92e96ba`.

See [password-manager/MIGRATION_RECORD.md](password-manager/MIGRATION_RECORD.md).

## Rule
**Move first, refactor second.**

No cryptographic, vault-format, key-derivation, sync-protocol, recovery, or public-API redesign belongs in the migration.

## Verification gate
1. Run the Security Suite and Password Manager CI workflows.
2. Compare results with the green standalone baseline.
3. Fix migration-only path/workspace failures.
4. Complete Windows verification.
5. Merge only when the migration branch is green.
6. Extract shared suite components only in a later phase.
