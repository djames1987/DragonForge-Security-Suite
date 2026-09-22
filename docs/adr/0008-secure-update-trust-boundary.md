# ADR 0008 — Secure update trust boundary

## Status

Accepted — Phase 14

## Decision

DragonForge treats update transport, hosting, and release discovery as untrusted delivery mechanisms. Authorization to install comes from a pinned ML-DSA-65 signature over deterministic release metadata, followed by SHA-256 artifact verification and Windows Authenticode verification.

Security Center never accepts an unsigned fallback and never automatically executes a downloaded installer.

## Consequences

- Compromise of a CDN, release URL, or ordinary HTTPS transport does not by itself authorize modified update bytes.
- The update signing seed becomes high-value release infrastructure and must remain outside source control and ordinary developer builds.
- Rotating the update key requires an intentionally shipped application update containing the new pinned public key/key ID.
- Authenticode remains independently required for the executable installer.
- Update installation is user initiated; unattended privileged updating is outside the Phase 14 boundary.
