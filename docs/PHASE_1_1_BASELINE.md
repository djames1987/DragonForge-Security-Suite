# Phase 1.1 — Post-Migration Baseline

## Status

**Complete**

Phase 1.1 establishes the first clean DragonForge Security Suite baseline after the Password Manager migration.

## Canonical development repository

DragonForge Security Suite is now the canonical active development repository for DragonForge Password Manager and future suite components:

- https://github.com/djames1987/DragonForge-Security-Suite

The standalone Password Manager repository remains the historical pre-migration source and verification baseline:

- https://github.com/djames1987/DragonForge-Password-Manager

No new Password Manager feature development should intentionally diverge between the two repositories.

## Migration provenance

Standalone source:
- commit: `1ee25e751da094577a9c6bbe10af5bdca92e96ba`
- tree: `346e5707717f8b9d2b2d51aa72b7f9d9bbc1477f`

Suite verification:
- final tested migration branch commit: `fb48621503561255b2b0ae21248124a05515f985`
- PR #2 merge commit: `e4d2c6debbb28c1cfb58816f1f97ccca51767b41`
- Phase 11 automated verification: **PASS**
- verification log SHA-256: `DDCEF374F6DD632146CFEFB6F20780B01FAD918ABF2DC5B2089481A91B281ADB`

## Verified migration scope

The following components are now part of the Security Suite workspace:

- `apps/password-manager/`
- `services/password-manager-sync/`
- `extensions/password-manager-browser/`
- `crates/dragonforge-crypto/`
- `crates/dragonforge-vault/`
- `docs/password-manager/`
- `scripts/password-manager/`

The Password Manager migration retained its production architecture and behavior. Migration fixes were limited to repository-relative path changes and test isolation for Windows Credential Manager integration tests.

## Test-isolation note

Windows recovery and multi-device integration tests share an operating-system credential store. During migration verification, parallel test execution exposed nondeterministic missing-entry failures.

The fix serializes those integration tests with process-local guards and uses `--test-threads=1` in explicit Phase 11/CI invocations.

Production credential-storage, sync, recovery, vault, and cryptographic behavior were not changed by these isolation fixes.

## CI responsibilities

### Suite Foundation CI

`.github/workflows/ci.yml` is responsible for:
- `dragonforge-core`;
- Security Center foundation;
- suite-level formatting affecting those components.

It intentionally does not duplicate the full Password Manager regression suite.

### Password Manager CI

`.github/workflows/password-manager-ci.yml` is responsible for:
- Password Manager desktop application;
- sync service;
- browser extension;
- crypto/vault crates;
- product-specific regression suites;
- Windows Password Manager verification where a self-hosted runner is available.

GitHub-hosted jobs have previously failed before receiving a runner or executing workflow steps. The successful local Phase 11 run remains the verified migration gate; runner availability is an infrastructure issue, not a replacement for code-level regression evidence.

## Baseline constraints for Phase 2

Phase 2 may refactor only with explicit tests and narrow review.

Do not casually combine shared-foundation extraction with:
- cryptographic algorithm changes;
- vault-format changes;
- KDF changes;
- sync protocol changes;
- account recovery redesign;
- credential-storage redesign;
- serialization format changes.

Each security-sensitive change should remain independently reviewable and testable.

## Known repository follow-up

The `main` branch is not currently protected. Branch protection should be enabled when repository settings/permissions permit requiring stable checks without blocking development on unavailable runners.

A workspace `Cargo.lock` is also not currently committed. Reproducible dependency locking should be addressed deliberately before public production release/distribution.
