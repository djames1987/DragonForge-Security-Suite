# Public Screenshot Capture Checklist

This checklist defines the screenshots that may be added to the public-facing README after they are captured from a clean demo environment. Phase 5 deliberately does not fabricate screenshots or reuse visual assets covered by unresolved provenance gate `DF-P3-ASSET-001`.

## Capture environment

- Use a disposable Windows demo account with no personal files, browser profile, production credentials, recovery material, OTP seeds, vault contents, or private network data.
- Use only synthetic DragonForge demo data. Suggested labels: `Demo Workstation`, `Example Vault`, `Example Account`, `example.test`, and RFC 5737 documentation addresses where an IP address is needed.
- Keep local usernames, home paths, machine names, email addresses, tokens, database connection strings, real event records, and real network endpoints out of frame.
- Capture the application window only; do not include the desktop, taskbar notifications, terminal history, browser account chrome, or unrelated windows.
- Prefer PNG at 1600×900 or 1440×900. Crop consistently and optimize before committing.

## Required captures

1. `docs/assets/readme/security-center-overview.png` — Security Center overview with synthetic healthy/demo component states and no host-specific diagnostics.
2. `docs/assets/readme/security-center-events.png` — bounded event/activity view populated only with synthetic demo events.
3. `docs/assets/readme/security-scanner-demo.png` — scanner results from a prepared demo fixture, with machine-identifying evidence hidden or replaced by synthetic values.
4. `docs/assets/readme/backup-recovery-demo.png` — backup/restore workflow using a synthetic demo destination and no real user paths.
5. Optional `docs/assets/readme/password-manager-demo.png` — only if the Password Manager view contains entirely synthetic credentials and no recovery/device secrets.

## Pre-commit review

For every image, zoom to 200% and verify that it contains none of the following: credentials, tokens, private keys, recovery material, OTP seeds, vault plaintext, personal mailbox addresses, real usernames, machine names, private filesystem paths, real IP/DNS data, real Security Center event content, or account identifiers.

Record the capture date, app/source commit, synthetic dataset used, image dimensions, optimized file size, and reviewer in the Phase 5/public-readiness report. Do not use the existing branding PNGs, app icons, or Password Manager documentation SVGs as public-facing visuals until `DF-P3-ASSET-001` is genuinely resolved.
