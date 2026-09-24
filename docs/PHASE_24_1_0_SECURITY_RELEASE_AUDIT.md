# Phase 24 — 1.0 Security & Release Audit

**Status: Implementation Complete — Local Verification Pending**

Phase 24 is the release-decision gate for DragonForge Security Suite 1.0. It introduces no new end-user capability. Instead it freezes the 1.0 feature set, composes prior security evidence into one authoritative audit, and makes unresolved-risk acceptance explicit.

## Delivered

- 1.0 feature-freeze policy under `release/PHASE24_FEATURE_FREEZE.md`;
- deployment-wide threat-model review;
- explicit residual-risk and release-blocker register;
- live RustSec advisory audit reused from Phase 12.7;
- locked dependency-license inventory with reviewed SPDX identifier allowlist;
- deterministic same-commit Git source-archive reproducibility check;
- tracked-source private-key/secret-material scan;
- release engineering fail-closed invariant review;
- privileged-service identity/capability invariant review;
- secure-update signing/downgrade/channel invariant review;
- installer privilege and optional-service invariant review;
- hostile-input/abuse regression gate using the existing parser, replay, rate-limit, tamper and malformed-input tests;
- machine-readable Phase 24 audit-gate definition;
- CI coverage and authoritative Windows verifier.

## Release freeze

Phase 24 freezes the encrypted formats, protocol semantics, privileged capability set, component scope, and stable update trust model for 1.0.

Only release-blocking security/correctness/accessibility/privacy/dependency/release-engineering fixes should land before Phase 25. New features and broader privileged capabilities are post-1.0 work.

## Dependency and license audit

The audit requires:

- tracked and locked `Cargo.lock`;
- live RustSec `cargo audit`;
- every external Cargo package to expose license metadata;
- every license identifier to belong to the reviewed Phase 24 permissive/weak-copyleft allowlist;
- a generated JSON license inventory retained with local verification evidence.

The license gate is intentionally conservative. A new identifier fails the release audit until explicitly reviewed. Complex SPDX expressions also fail closed unless their exact expression has been explicitly reviewed. Phase 24 currently records three exact reviewed complex expressions already present in the locked dependency graph: `Apache-2.0 WITH LLVM-exception`, `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT`, and `(MIT OR Apache-2.0) AND Unicode-3.0`.

## Threat / crypto / format review

Phase 24 does not replace the detailed cryptographic and format reviews from earlier phases. It verifies that their release-blocking invariants remain present:

- AES-GCM/Argon2id and authenticated package handling stay product-owned and versioned;
- hostile/truncated/unsupported format tests remain in the workspace suite;
- Password Manager post-quantum primitives retain their round-trip/tamper tests;
- recovery formats remain bounded and future-schema fail-closed;
- update signatures and downgrade/channel checks remain fail-closed.

## Privilege and installer review

The privileged-service boundary remains capability-scoped to the Phase 19 firewall policy capability. Generic shell/command execution remains prohibited.

The release gate checks that:

- caller OS identity is required;
- exact Agent path is required;
- Authenticode is required;
- publisher pinning is required;
- replay/rate abuse protections remain tested;
- installer normal path remains `PrivilegesRequired=lowest`;
- privileged-service installation remains an explicit optional task requiring Administrator approval.

## Release engineering review

Stable releases must:

- originate from a clean working tree;
- build from an exact Git tag matching HEAD;
- use tracked/locked dependencies;
- require Authenticode signing;
- require an installer for update-capable publication;
- verify artifact SHA-256 manifests;
- verify timestamped signatures when signing is required;
- refuse mutation of an existing GitHub release;
- require signed update-manifest credentials.

## Reproducibility statement

Phase 24 verifies deterministic source reproduction by creating two Git archives from the same exact commit and requiring equal SHA-256 hashes.

DragonForge does **not** claim bit-for-bit reproducible Windows binaries across machines/toolchains. Windows PE build metadata and RFC 3161 signing timestamps are intentionally allowed to differ. Release identity instead relies on exact source/tag binding, explicit artifact hashes, Authenticode, timestamp validation, and signed update metadata.

## Secret scanning

The Phase 24 verifier scans tracked text source/configuration files for committed private-key blocks and literal update-signing secret assignments. Environment-variable names and documented placeholders are allowed; real secret material is not.

## 1.0 decision semantics

A Phase 24 PASS means the repository and release machinery are qualified to enter Phase 25. It does not itself publish 1.0.

Phase 25 must still build/sign/verify the actual stable tag and artifacts.

## Verification

Run on Windows:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase24-security-release-audit.ps1
```

The verifier writes a timestamped transcript, SHA-256 sidecar, and dependency-license JSON inventory under `test-logs`.

Phase 24 remains **Implementation Complete — Local Verification Pending** until the authoritative Windows run reaches its PASS marker.
