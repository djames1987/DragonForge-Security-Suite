# Phase 14 — Secure Update System

## Status

**Implementation Complete — Local Verification Pending**

Phase 14 adds a fail-closed update trust path for DragonForge Security Suite. Network transport is treated only as delivery; it is not trusted to authorize an update.

## Trust model

A DragonForge update is eligible for installation only when all of these checks succeed:

1. the update manifest is valid JSON using schema version 1;
2. the manifest is signed with ML-DSA-65 by the pinned DragonForge update release key;
3. the manifest key ID matches the configured release key identity;
4. the manifest channel exactly matches the user's selected alpha, beta, or stable channel;
5. the candidate SemVer does not downgrade the installed suite;
6. the manifest contains a bounded HTTPS Windows-installer artifact;
7. the downloaded installer size and SHA-256 match the signed manifest;
8. the installer has a valid Windows Authenticode signature;
9. Security Center rechecks SHA-256 and Authenticode immediately before launch;
10. the user explicitly selects **Install verified update**.

There is no silent update execution and no unsigned fallback.

## Update channels

Security Center persists one explicit channel:

- **stable** — accepts only non-prerelease SemVer versions;
- **beta** — accepts only versions with a `beta` prerelease label;
- **alpha** — accepts only versions with an `alpha` prerelease label.

A manifest from another channel is rejected even if its signature is otherwise valid.

## Release-key configuration

Release builds configure these compile-time environment variables:

- `DRAGONFORGE_UPDATE_PUBLIC_KEY_HEX` — pinned ML-DSA-65 public key;
- `DRAGONFORGE_UPDATE_KEY_ID` — stable public key identifier;
Security Center discovers channel manifests from the repository's GitHub Releases assets by the exact names
`DragonForge-Security-Suite-update-alpha.json`, `...-beta.json`, and `...-stable.json`.
Optional compile-time overrides (`DRAGONFORGE_UPDATE_FEED_ALPHA`, `DRAGONFORGE_UPDATE_FEED_BETA`, and
`DRAGONFORGE_UPDATE_FEED_STABLE`) may point a channel at another bounded HTTPS source for private/test deployments.

A build without a public key remains usable but reports secure updates as unconfigured; it does not fall back to unsigned metadata.

Release publication additionally requires:

- `DRAGONFORGE_UPDATE_SIGNING_KEY_HEX` — the 32-byte ML-DSA-65 private seed, supplied only to the offline/release publishing environment.

The private signing seed is never stored in the repository or application. The signing utility also
requires the derived public key to match `DRAGONFORGE_UPDATE_PUBLIC_KEY_HEX` and zeroizes seed material on drop.

## Release pipeline

`scripts/generate-signed-update-manifest.ps1` converts an already verified release artifact manifest into a signed channel update manifest. The signed payload binds:

- version, tag, and full Git commit;
- explicit release channel;
- publication time;
- installer filename and HTTPS release URL;
- exact installer byte length;
- exact installer SHA-256;
- the requirement for Authenticode verification.

`publish-windows-release.ps1` now requires `-SignRelease`, a real installer, and update-manifest signing credentials. Portable-only or unsigned publication is not update-capable under Phase 14.

## Failure and rollback behavior

- Older versions are rejected before download.
- Same-version manifests are valid signed metadata but are reported as current rather than installable.
- Cross-channel manifests are rejected.
- Signature, size, hash, or Authenticode failure deletes/refuses the staged installer and never launches it.
- The existing installed suite is not directly overwritten by Security Center. Security Center launches the verified installer only after explicit user approval, leaving installer lifecycle/rollback behavior at the established package boundary.
- A staged installer is rehashed and Authenticode-checked again immediately before execution.

## Security Center UX

The **Updates** view provides three deliberately separate actions:

1. **Check for updates**
2. **Download & verify**
3. **Install verified update**

The third action remains disabled until preparation succeeds.

## Verification

Run:

    powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase14-secure-update-tests.ps1

The verifier covers workspace formatting/check/Clippy/tests, update-engine tests, JavaScript validation, locked Cargo metadata, Phase 14 source/document invariants, release-script PowerShell syntax, and current suite-phase metadata.

A Phase 14 implementation PASS validates the update system implementation. It does not replace Phase 13's separate multi-machine beta qualification gate.
