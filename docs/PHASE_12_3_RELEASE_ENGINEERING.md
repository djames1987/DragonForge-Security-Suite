# Phase 12.3 — Release Engineering

## Status

**Verified Complete**

Phase 12.3 converts DragonForge release creation from manually coordinated packaging into a tag-bound, fail-closed release workflow.

## Release invariants

A publishable release must satisfy all of the following:

- the working tree is clean;
- the requested Git tag exists;
- the tag resolves exactly to the checked-out commit;
- all suite version surfaces match the tag version;
- `Cargo.lock` is tracked;
- `cargo metadata --locked` succeeds;
- the portable ZIP and installer are built from the tagged commit;
- packaged BUILD-INFO identifies the exact tag and commit;
- artifact SHA-256 sidecars match;
- the portable package's internal SHA256SUMS manifest matches every packaged file;
- all ten expected executables exist in the portable package;
- a release manifest records tag, commit, platform, expected executable set, artifact hashes, sizes, and signing state;
- the release manifest has its own SHA-256 sidecar;
- GitHub publication refuses to overwrite an existing release.

## Release preparation

From a clean working tree:

~~~powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\prepare-release.ps1 -Version 0.1.0-alpha.3
~~~

This:

1. stamps the workspace package version;
2. stamps all nine Tauri application versions;
3. stamps Security Center's visible About version;
4. stamps numeric Inno Setup version metadata;
5. refreshes `Cargo.lock` through Cargo;
6. checks the full workspace;
7. re-validates version consistency.

Review and commit the changed files. Then create an annotated tag on that exact commit and push both the commit and tag.

## Tagged release build

~~~powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-tagged-windows-release.ps1 -Tag v0.1.0-alpha.3
~~~

The script refuses to build if `HEAD` is not the tagged commit.

It builds the portable ZIP and Windows installer, generates release notes from the repository-controlled template and Git history, verifies the artifacts, and writes a machine-readable release manifest.

Use `-NoInstaller` only when intentionally creating a portable-only release.

## Generated artifacts

For tag `vX.Y.Z[-prerelease]`:

- `DragonForge-Security-Suite-vX.Y.Z-win-x64.zip`
- ZIP `.sha256`
- `DragonForge-Security-Suite-vX.Y.Z-win-x64-setup.exe`
- installer `.sha256`
- `release-notes-vX.Y.Z.md`
- `release-manifest-vX.Y.Z.json`
- release manifest `.sha256`

Temporary expanded staging and verification directories are removed automatically.

## Publishing

~~~powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\publish-windows-release.ps1 -Tag v0.1.0-alpha.3
~~~

The publisher:

- requires GitHub CLI authentication;
- requires a clean exact-tag checkout;
- refuses to mutate an existing GitHub release;
- runs the complete tagged build and artifact verification before upload;
- uploads only the verified assets;
- marks tags containing a prerelease suffix as GitHub prereleases;
- uses `--verify-tag` so GitHub publication cannot create an unexpected tag.

## Cargo.lock policy

`Cargo.lock` is mandatory for releases. Tagged builds verify that it is tracked and that Cargo accepts it with `--locked`.

Release builds must not run dependency-updating commands after the release tag is created.

## Signing boundary

Phase 12.3 records `code_signing = "unsigned-phase-12.3"` in the release manifest.

The pipeline is deliberately ready for a signing step, but Authenticode certificate/key handling is not implemented here. That belongs to Phase 12.4.

## Verification

Run:

~~~powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase12.3-release-engineering-tests.ps1
~~~

The verifier checks formatting, targeted compile/Clippy/tests, JavaScript syntax, PowerShell syntax for every release script, version consistency, locked Cargo metadata, release-script invariants, and required Phase 12.3 artifacts.

## Verified Windows result

Phase 12.3 completed authoritative Windows verification on `DRACO`.

- Machine: `DRACO`
- Windows: `Microsoft Windows NT 10.0.26200.0`
- Result: **PASS**
- Verified log: `dragonforge-phase12.3-release-engineering-20260922-112111.log`
- Log SHA-256: `E896E0343DD622D1116AD3D63A29FBC8E9E10B9A157900EF762BE354B4C9E037`

The passing run covered rustfmt, targeted compile checks, strict Clippy, all 9 Agent tests, all 22 Security Center tests, JavaScript syntax validation, tracked `Cargo.lock`, `cargo metadata --locked`, suite-wide release-version consistency checks, PowerShell syntax validation for every release script, and the required exact-tag/publication/artifact-verification invariants.
