# Password Manager Migration Provenance

- Source repository: https://github.com/djames1987/DragonForge-Password-Manager
- Frozen source commit: `1ee25e751da094577a9c6bbe10af5bdca92e96ba`
- Source verification status: Phase 11 full Windows release verification recorded as passed before migration.
- Migration branch: `migration/password-manager`

## Path mapping

- `apps/desktop` -> `apps/password-manager`
- `apps/sync-server` -> `services/password-manager-sync`
- `apps/browser-extension` -> `extensions/password-manager-browser`
- `crates/dragonforge-crypto` -> `crates/dragonforge-crypto`
- `crates/dragonforge-vault` -> `crates/dragonforge-vault`
- `docs/*` -> `docs/password-manager/*`
- `scripts/*` -> `scripts/password-manager/*`

The standalone repository remains the historical source and pre-migration verification baseline. Migration changes are limited to repository placement, workspace/CI metadata, relative paths required by the new layout, and script path normalization. Cryptography, vault formats, key derivation, sync protocol semantics, recovery behavior, and public product behavior are intentionally unchanged.
