# Phase 25 — DragonForge Security Suite 1.0

**Status: Release Readiness Verified — Signed Stable Release Pending**

Phase 25 converts the verified Phase 24 source baseline into the DragonForge Security Suite 1.0 release line.

## Delivered in source

- suite/workspace/Tauri/installer version baseline stamped to `1.0.0`;
- application UI release marker updated to DragonForge 1.0;
- stable 1.x format/protocol compatibility policy;
- 1.0 migration/rollback guide;
- stable support policy;
- vulnerability-response policy;
- corrected public security/release language for the optional privileged-service boundary;
- machine-readable Phase 25 release gate;
- dedicated authoritative Phase 25 release-readiness verifier;
- dedicated `publish-dragonforge-1.0.ps1` final stable release orchestrator;
- CI awareness for Phase 25 release artifacts and verifier.

## Stable-release trust boundary

Phase 25 does not weaken the Phase 24 release gate.

The actual `v1.0.0` publication requires external production credentials that are intentionally absent from Git:

- Authenticode code-signing identity;
- RFC 3161 timestamp service configuration;
- pinned ML-DSA-65 update public key/key ID;
- ML-DSA-65 private update signing seed;
- authenticated GitHub CLI release access.

The final release cannot be truthfully marked published or Verified Complete until a trusted Windows release host builds, signs, verifies, and publishes those artifacts.

## Required final artifacts

The stable GitHub release must include:

- `DragonForge-Security-Suite-v1.0.0-win-x64.zip`;
- ZIP SHA-256 sidecar;
- signed `DragonForge-Security-Suite-v1.0.0-win-x64-setup.exe`;
- installer SHA-256 sidecar;
- release manifest and SHA-256 sidecar;
- signed `DragonForge-Security-Suite-update-stable.json`;
- update-manifest SHA-256 sidecar.

The release must not be marked prerelease.

## Verification flow

First, on a clean checkout of the Phase 25 source:

~~~powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase25-1.0-release-tests.ps1
~~~

After that verifier passes and the exact release commit is tagged `v1.0.0`, configure the protected signing/update credentials and run:

~~~powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\publish-dragonforge-1.0.ps1
~~~

The final publisher reuses the existing exact-tag build/sign/verify/update publication pipeline. It does not introduce an alternate signing or upload mechanism.

## Authoritative release-readiness verification

The final authoritative Phase 25 verifier passed on `DRACO` (Microsoft Windows NT 10.0.26200.0) on 2026-09-24 against the post-uninstaller-hardening source commit `de39b05888708df2de9b3156226463aada0551d8`.

Final verified evidence:
- log: `dragonforge-phase25-1.0-release-readiness-20260924-103700.log`;
- independently verified SHA-256: `EA7867BCFE370FF5D8B3828F7B7969BAC143F374B2C0D8EA3D92BE5295DCC61F`;
- locked dependency license inventory: 525 packages;
- exact-commit source reproducibility: PASS for `de39b05888708df2de9b3156226463aada0551d8`;
- stable signing fail-closed checks: PASS;
- PowerShell syntax checks: PASS;
- release-profile builds verified for all 11 expected DragonForge executables with `--locked`;
- final marker: `PHASE 25 DRAGONFORGE SECURITY SUITE 1.0 RELEASE READINESS: PASS`.

The earlier pre-uninstaller readiness result remains historical evidence only.

## Beta installer/uninstaller qualification

The dedicated Windows prerelease `v1.0.0-beta.1` was published on 2026-09-24 from annotated tag commit `18d1ad8fb8f940c49213ac499411051b1655b557`.

The guarded DRACO publisher completed a real isolated install -> Agent start -> uninstall lifecycle and the final retained evidence passed:
- log: `dragonforge-installer-uninstall-1.0.0-beta.1-20260924-102957.log`;
- independently verified SHA-256: `3CC2B2E210863C0E1308E5B8B3C9CB454AC5A0AB53CA8EB9743A64C9FAAFE107`;
- final marker: `DRAGONFORGE INSTALLER/UNINSTALLER LIFECYCLE: PASS`;
- generated uninstaller existed and executed successfully;
- exact installed Agent was running before uninstall and was stopped by uninstall;
- installer-managed application directory and shortcuts were removed;
- data outside the installation directory was preserved;
- GitHub release is marked prerelease and remains outside the stable update channel.

The beta release includes the installer, portable ZIP, their SHA-256 sidecars, release manifest + sidecar, and lifecycle log + sidecar.

## Completion semantics

- **Release Readiness Verified — Signed Stable Release Pending** means the final post-uninstaller-hardening 1.0 source baseline has passed the authoritative Windows readiness gate and is ready for the protected signing/tag/publication step.
- **Verified Complete / Released** still requires a successful signed `v1.0.0` publication receipt.
