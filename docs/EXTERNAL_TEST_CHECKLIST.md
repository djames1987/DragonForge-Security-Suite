# DragonForge External Test Checklist

Use this checklist with a downloaded portable pre-release on a Windows machine that does not require a development environment.

## Record first

- Release/tag:
- BUILD-INFO.txt Git commit:
- Windows edition/version/build:
- 64-bit architecture:
- WebView2 detected:
- Standard user or administrator account:
- Fresh Windows user profile or established profile:
- VM or physical machine:

## Package integrity and startup

- Verify the ZIP SHA-256 against the release sidecar.
- Extract the complete top-level DragonForge folder.
- Run Verify-Package.cmd and confirm every packaged file passes its SHA-256 check.
- Run Check-Prerequisites.cmd.
- Start Launch-Security-Center.cmd.
- Confirm Security Center opens without Rust, Cargo, Node.js, Git, or the source repository installed.
- Open About and copy diagnostics.
- Confirm the diagnostic report does not contain usernames, vault paths, passwords, tokens, recovery material, OTP secrets, or vault/event contents.

## Agent

- Start DragonForge Agent from Security Center.
- Confirm Agent status changes to running/healthy.
- Close Security Center and confirm the Agent continues running.
- Reopen Security Center and confirm it reconnects.
- Stop the Agent with Stop-DragonForge-Agent.cmd.
- Move the extracted package folder while the Agent is stopped and repeat launch.

## Suite launch

Launch each component from Security Center and confirm it opens:

- Password Manager
- File Vault
- Authenticator
- Security Scanner
- Integrity Monitor
- Network Guard
- Backup & Recovery
- Secure Share

## Functional smoke tests

Use disposable test data only.

- Password Manager: create/unlock vault, add/edit/delete test entry, verify vault, test local loopback sync if a test sync server is available.
- File Vault: create encrypted container, inspect/verify, extract to a new destination.
- Authenticator: add a test TOTP/HOTP secret and verify code generation; do not use a production OTP seed.
- Security Scanner: run scan and confirm read-only findings render.
- Integrity Monitor: create baseline and compare after a deliberate disposable change.
- Network Guard: refresh connections/listeners and confirm visibility without enforcement.
- Backup & Recovery: create, verify, and restore a backup using disposable files.
- Secure Share: create, verify, reveal/extract, and test expiration behavior with disposable content.

## Restart and error handling

- Close/reopen each application.
- Restart Windows and retest Security Center and Agent behavior.
- Test with WebView2 unavailable if a disposable VM can safely reproduce that condition.
- Test launching with one sibling executable temporarily renamed; confirm a clear error instead of silent failure.
- Test read-only/unwritable extraction locations only with disposable copies.
- Confirm failures do not expose secrets in UI errors.

## Report a failure

Use the repository bug-report template and attach/paste the redaction-safe Security Center diagnostic report.

Never submit real passwords, Account Secrets, recovery kits/codes, OTP seeds, sync/admin tokens, vault contents, private encryption keys, or sensitive personal files.


## Phase 13 beta evidence

For a required beta matrix machine:
- run `scripts/run-phase13-beta-readiness-tests.ps1` from the source qualification checkout;
- retain the generated log and SHA-256 sidecar;
- complete the reboot/Agent/application/installer checks relevant to the assigned BQ scenario;
- create the machine-readable record with `scripts/new-beta-qualification-record.ps1`;
- do not mark an untested item as passed and do not reuse one machine's evidence for another required matrix row.
