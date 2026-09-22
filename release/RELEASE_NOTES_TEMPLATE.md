# DragonForge Security Suite {{TAG}}

This is an automated DragonForge release-engineering note generated from repository-controlled metadata.

## Release identity

- Tag: `{{TAG}}`
- Commit: `{{COMMIT}}`
- Channel: {{CHANNEL}}
- Platform: Windows x64
- Build type: portable ZIP + Windows installer

## Included applications

- DragonForge Security Center
- DragonForge Password Manager
- DragonForge File Vault
- DragonForge Authenticator
- DragonForge Security Scanner
- DragonForge Integrity Monitor
- DragonForge Network Guard
- DragonForge Backup & Recovery
- DragonForge Secure Share
- DragonForge Agent

## Changes since previous tag

{{CHANGES}}

## Security / distribution notes

- Release artifacts are verified against SHA-256 sidecars and a release manifest before publication.
- Artifact build metadata must identify the exact tagged commit.
- The current release pipeline supports unsigned builds; Authenticode signing is Phase 12.4 work.
- Until signing is enabled, Windows SmartScreen may show an unknown-publisher warning.
- DragonForge Agent remains per-user and non-elevated.
