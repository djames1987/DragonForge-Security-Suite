# Phase 25 — DragonForge Security Suite 1.0

**Status: Installer/Uninstaller Release-Candidate Validation Pending**

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

The Phase 25 verifier passed on `DRACO` (Microsoft Windows NT 10.0.26200.0) on 2026-09-24 against source commit `d5f0b819d03537cd9cf7e9d32f90894a0052a596`.

Before stable publication, installer/uninstaller hardening was intentionally added after that verified commit. The prior evidence remains valid for its recorded source commit, but the new final source must pass the Phase 25 readiness verifier again after the testing prerelease validates the complete uninstall lifecycle.

Verified evidence:
- log: `dragonforge-phase25-1.0-release-readiness-20260924-092829.log`;
- independently verified SHA-256: `7071C7D9EA1FC4CF17CF2F33F786AEA279559C14568344370BA5A1F4E56643C5`;
- locked dependency license inventory: 525 packages;
- release-profile builds verified for all 11 expected DragonForge executables;
- final marker: `PHASE 25 DRAGONFORGE SECURITY SUITE 1.0 RELEASE READINESS: PASS`.

## Completion semantics

- **Release Readiness Verified — Signed Stable Release Pending** means the exact 1.0 source baseline has passed the authoritative Windows readiness gate and is ready for the protected signing/tag/publication step.
- **Verified Complete / Released** still requires a successful signed `v1.0.0` publication receipt.
