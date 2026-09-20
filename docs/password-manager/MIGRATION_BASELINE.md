# Password Manager Migration Baseline

Source repository: `djames1987/DragonForge-Password-Manager`

Tested source commit: `1ee25e751da094577a9c6bbe10af5bdca92e96ba`

Source commit message: `docs: record full Phase 11 Windows release verification`

Migration date: 2026-09-20

## Pre-migration status

- User-reported full Password Manager test run: passed.
- Phase 11 Windows release verification: recorded as passed in the source repository.
- Source repository working state used for migration: pushed commit above.
- Known RustSec RSA advisory exception is retained via `.cargo/audit.toml` and remains guarded by the Phase 11 validation script.

## Migration rule

This migration preserves product behavior and persistent formats. Crypto algorithms, vault formats, key-derivation parameters, sync protocol semantics, recovery semantics, and public APIs are not intentionally changed.

The standalone repository remains the historical source and rollback reference until the suite copy completes post-migration verification.
