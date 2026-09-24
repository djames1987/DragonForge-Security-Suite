# DragonForge Security Suite Support Policy

## Supported release line

After the signed `v1.0.0` stable release is published, DragonForge supports the current stable 1.x release line.

Security and correctness fixes are delivered as new immutable releases. Published release artifacts are never silently replaced.

## Platforms

DragonForge 1.0's supported packaged desktop target is Windows x64 on Windows 10 or later with Microsoft Edge WebView2 Runtime available for the desktop UI.

The source workspace contains platform-aware libraries and tests, but the 1.0 production distribution commitment is the verified Windows x64 installer and portable package.

## Update policy

Stable users should remain on the stable channel. Stable updates require:

- signed ML-DSA-65 update metadata from the pinned DragonForge update key;
- matching stable channel/version policy;
- SHA-256/size verification;
- Authenticode-verified installer;
- explicit user approval before installer launch.

There is no unsigned stable fallback.

## Support expectations

For bug reports include the DragonForge version, Windows version/build, affected component, reproduction steps, and redaction-safe diagnostics when available.

Never attach real passwords, Account Secrets, recovery kits/codes, OTP seeds, private keys, sync/admin tokens, vault contents, or sensitive personal files.

## End of support

When a future major line replaces 1.x, DragonForge will document migration requirements and the support transition. A security issue may require accelerating retirement of a vulnerable release, but existing published artifacts will not be rewritten.
