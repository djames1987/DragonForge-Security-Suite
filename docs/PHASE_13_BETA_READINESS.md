# Phase 13 — Beta Readiness & Release Qualification

## Status

**Implementation Complete — Local Verification Pending**

Phase 13 converts DragonForge's external-test baseline into a release-qualification system suitable for deciding when an alpha build may become a beta candidate.

## What Phase 13 delivers

- a required Windows beta qualification matrix with scenario IDs and retained evidence requirements;
- a release-blocking versus advisory defect policy;
- an explicit beta release gate;
- a machine-readable qualification-record generator that validates the verifier log SHA-256 before recording evidence;
- a fail-closed evidence evaluator that requires all six scenarios to pass for the same candidate commit;
- an authoritative Phase 13 Windows verifier;
- real release-profile builds for all ten executables;
- qualification-only portable ZIP and Inno Setup installer construction;
- portable/installer SHA-256 validation and portable internal-manifest validation;
- workspace formatting, strict Clippy, tests, JavaScript checks, browser-extension tests, locked metadata, and RustSec audit coverage;
- PowerShell 5.1 syntax validation for qualification/release/security scripts;
- cleanup of qualification-only build artifacts while retaining the reusable Cargo target cache;
- updated external-test documentation and CI awareness.

## Important distinction: Phase implementation versus beta qualification

A successful Phase 13 verifier proves that the qualification machinery works on the authoritative development machine. It does **not** invent evidence for machines that were not tested.

DragonForge may be labeled beta only after every required row in `BETA_QUALIFICATION_MATRIX.md` has retained evidence and the release-blocking defect gate is clear.

## Automated qualification gate

Run:

    powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase13-beta-readiness-tests.ps1

The verifier:
1. checks Rust formatting;
2. checks and Clippy-lints the complete workspace with all features;
3. runs complete workspace tests with all features;
4. validates all desktop/browser JavaScript;
5. runs browser extension Node tests;
6. verifies locked Cargo metadata;
7. runs the RustSec dependency audit;
8. builds all ten desktop/service executables in release profile;
9. creates a qualification-only portable ZIP;
10. creates a qualification-only Inno Setup installer;
11. validates external/internal SHA-256 package integrity;
12. validates the required qualification/release documentation and source invariants;
13. parses relevant PowerShell scripts under the current Windows PowerShell parser;
14. removes qualification-only `dist` artifacts but intentionally retains the Cargo `target` cache;
15. writes a timestamped log plus SHA-256 sidecar.

## Manual evidence record

After the automated verifier passes on a matrix machine, record the manual/runtime parts with `scripts/new-beta-qualification-record.ps1`.

The record does not collect a username and requires the tester to explicitly classify:
- scenario ID;
- physical or VM;
- standard user or administrator account running non-elevated;
- fresh or established profile;
- WebView2 present/missing;
- installer or portable;
- reboot result;
- installer lifecycle result;
- Agent lifecycle result;
- suite launch result.

The script verifies the qualification log against its sidecar before writing the record.

## Disk-space policy

Phase 13 qualification removes temporary portable/installer staging and qualification-only output artifacts after verification. It intentionally keeps Cargo's `target` directory so subsequent development/test runs can reuse compiled dependencies.

## Beta gate

See:
- `docs/BETA_QUALIFICATION_MATRIX.md`
- `docs/BETA_RELEASE_GATE.md`
- `docs/EXTERNAL_TEST_CHECKLIST.md`
- `docs/INSTALLER_TEST_CHECKLIST.md`

No beta label should be applied merely because one machine passes the automated verifier.


## Evaluate collected evidence

After records from all required machines are gathered under `test-logs\beta-qualification`, run:

    powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\evaluate-beta-qualification.ps1 -CandidateCommit <commit>

The evaluator requires BQ-01 through BQ-06 for the same candidate commit and fails if required automated, Agent, suite-launch, reboot, installer-lifecycle, or disposable-data attestations fail.
