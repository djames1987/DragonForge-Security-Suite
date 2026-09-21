# Password Manager Migration Plan

## Status

**Complete.**

DragonForge Password Manager was migrated into DragonForge Security Suite and passed the full post-migration Phase 11 verification gate before PR #2 was merged.

## Frozen standalone baseline

- Repository: https://github.com/djames1987/DragonForge-Password-Manager
- Source commit: `1ee25e751da094577a9c6bbe10af5bdca92e96ba`
- Source tree: `346e5707717f8b9d2b2d51aa72b7f9d9bbc1477f`

## Suite migration baseline

- Final migration branch commit verified locally: `fb48621503561255b2b0ae21248124a05515f985`
- Merge commit on `main`: `e4d2c6debbb28c1cfb58816f1f97ccca51767b41`
- Phase 11 result: **PASS**
- Verification log SHA-256: `DDCEF374F6DD632146CFEFB6F20780B01FAD918ABF2DC5B2089481A91B281ADB`

See [password-manager/MIGRATION_RECORD.md](password-manager/MIGRATION_RECORD.md) and [PHASE_1_1_BASELINE.md](PHASE_1_1_BASELINE.md).

## Final path mapping

| Standalone Password Manager | Security Suite |
|---|---|
| `apps/desktop` | `apps/password-manager/` |
| `apps/sync-server` | `services/password-manager-sync/` |
| `apps/browser-extension` | `extensions/password-manager-browser/` |
| `crates/dragonforge-crypto` | `crates/dragonforge-crypto/` |
| `crates/dragonforge-vault` | `crates/dragonforge-vault/` |
| Password Manager docs | `docs/password-manager/` |
| Password Manager scripts | `scripts/password-manager/` |

## Rule preserved

**Move first, refactor second.**

No intentional cryptographic, vault-format, key-derivation, sync-protocol, recovery, or public-API redesign was part of the migration.

The migration-only Windows test isolation fixes serialize integration tests that share Windows Credential Manager. They do not alter production credential-storage behavior.

## Result

The Security Suite repository is now the canonical development home for Password Manager work. The standalone repository remains the historical pre-migration source and baseline.

Any shared-component extraction now belongs to Phase 2 and must be reviewed/tested independently from the completed migration.
