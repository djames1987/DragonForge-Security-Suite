# DragonForge Beta Release Gate

A release may be labeled **beta** only when all of the following are true.

## Source and build
- exact release tag resolves to the built commit;
- working tree is clean for release preparation/build;
- `Cargo.lock` is tracked and `cargo metadata --locked` succeeds;
- workspace formatting, strict Clippy, tests, JavaScript checks, and dependency advisory audit pass;
- all ten expected executables build in release profile.

## Portable package and installer
- portable ZIP SHA-256 sidecar matches;
- internal `SHA256SUMS.txt` verifies every packaged file;
- all ten expected executables are present;
- installer SHA-256 sidecar matches;
- stable/signed-channel requirements are enforced by the existing release pipeline;
- build metadata identifies exact version/tag/commit.

## Runtime qualification
- Security Center starts;
- DragonForge Agent starts, authenticates, survives Security Center close/reopen, stops cleanly, and recovers from stale runtime state;
- all nine desktop apps launch from Security Center by exact sibling path;
- diagnostics/support bundles remain redaction-safe;
- disposable functional smoke tests pass for Password Manager, File Vault, Authenticator, Security Scanner, Integrity Monitor, Network Guard, Backup & Recovery, and Secure Share.

## Installer lifecycle
- fresh per-user install works without an unnecessary elevation prompt;
- in-place upgrade preserves user data;
- Agent is handled cleanly during upgrade;
- uninstall removes program files/shortcuts while preserving user-created data by design.

## Matrix and defect gate
- every required row in `BETA_QUALIFICATION_MATRIX.md` has retained evidence;
- no release-blocking defect remains open;
- known limitations are present in release notes.

A Phase 13 implementation PASS is not itself permission to label a release beta unless the full matrix and defect gate are satisfied.
