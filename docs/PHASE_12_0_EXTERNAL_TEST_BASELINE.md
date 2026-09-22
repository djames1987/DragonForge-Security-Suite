# Phase 12.0 — External Test Baseline

## Status

**Verified Complete**

Phase 12.0 shifts DragonForge from feature expansion to repeatable external testing. It establishes the support, diagnostics, test-matrix, and portable-package verification baseline needed before installer work.

## Delivered

- redaction-safe Security Center diagnostic report;
- one-click Copy diagnostics action in Security Center;
- build metadata including package version, Phase 12.0 marker, release channel, Git commit, OS, architecture, component states, and Agent state/capabilities;
- diagnostic identifiers such as Agent PID included only when the existing diagnostic-identifiers setting is enabled;
- explicit exclusion of vault contents, credentials, filesystem paths, activity/event content, sync tokens, recovery material, OTP secrets, and other user secrets from the report;
- external Windows test matrix;
- external tester checklist;
- structured GitHub bug-report template with explicit secret-handling warnings;
- portable-package integrity verifier generated into future release ZIPs;
- Phase 12.0 verification script covering formatting, compile, strict Clippy, Security Center/core/Agent tests, JavaScript syntax, and release-packaging script syntax.

## External-test policy

The frozen v0.1.0-alpha.1 release remains unchanged. Bugs found against it should identify that exact release. Fixes belong on main and should ship in a new pre-release such as v0.1.0-alpha.2 rather than replacing assets in place.

Testers should use disposable data only during alpha testing and must not submit real passwords, account secrets, recovery kits, OTP seeds, tokens, vault contents, or unredacted sensitive logs.

## Diagnostics

Security Center > About > Copy diagnostics produces a JSON support report designed for bug reports.

The report is intentionally metadata-only. It does not read vault files or component secret stores and does not include Security Center activity-event bodies or log-file contents.

## Verification

Run on Windows:

~~~powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase12-external-test-baseline.ps1
~~~

The script writes a timestamped log and SHA-256 sidecar under test-logs/.

## Verified local result

Phase 12.0 completed its authoritative Windows verification on a second development machine, providing a cross-machine validation of the baseline.

- Machine: `DRACO`
- Windows: `Microsoft Windows NT 10.0.26200.0`
- PowerShell: `5.1.26100.9444`
- Result: **PASS**
- Verified log: `dragonforge-phase12-external-test-baseline-20260922-093452.log`
- SHA-256: `96A158EA38F10E8A2F46F092E25C55CBE27E0C3F8BDF359EE844141BE18CDE88`

The passing run covered `cargo fmt --all --check`, targeted `cargo check`, strict Clippy with `-D warnings`, Agent and Security Center tests including the Phase 12.0 diagnostic-redaction test, JavaScript syntax validation, and required external-test/release-support artifacts.
