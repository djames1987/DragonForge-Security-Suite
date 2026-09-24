# DragonForge Security Suite v1.0.0 Release Checklist

- [ ] Phase 24 is Verified Complete.
- [ ] Phase 25 readiness verifier passes on the release host.
- [ ] Working tree is clean.
- [ ] Exact release commit is pushed to `main`.
- [ ] Annotated tag `v1.0.0` points to that exact commit.
- [ ] Authenticode signing identity is provisioned outside the repository.
- [ ] RFC 3161 timestamp service is configured.
- [ ] ML-DSA-65 update public key and key ID match the built Security Center.
- [ ] ML-DSA-65 update private signing seed is supplied only to the release environment.
- [ ] Stable installer and all packaged executables verify Authenticode/timestamp requirements.
- [ ] Portable ZIP internal hashes verify.
- [ ] Installer/ZIP/release-manifest SHA-256 sidecars verify.
- [ ] Signed stable update manifest verifies and references the exact installer.
- [ ] GitHub release is created as stable, not prerelease.
- [ ] Published release assets match the locally verified asset names/hashes.
- [ ] Release receipt is retained with the Phase 25 verifier log.
- [ ] Stable update check from a disposable installed build sees the published 1.0 metadata without bypassing trust checks.
