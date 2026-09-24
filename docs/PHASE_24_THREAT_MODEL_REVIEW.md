# Phase 24 — 1.0 Threat Model Review

This review consolidates DragonForge's deployment-wide trust assumptions before 1.0.

## Protected assets

- Password Manager vault contents, Account Secret, master-password-derived material, sync/device credentials, and recovery material.
- Authenticator OTP seeds, HOTP counters, and recovery codes.
- File Vault, Backup & Recovery, Secure Share, and suite-recovery plaintext.
- Agent and privileged-service authenticated control channels.
- managed firewall policy state and rollback records.
- signed update manifests, release artifacts, signing identity metadata, and release source identity.
- diagnostic/support output that could otherwise expose sensitive user data.

## Principal trust boundaries

1. Webview/UI input -> native Tauri commands.
2. User-facing applications -> product security crates.
3. Security Center -> per-user Agent.
4. Agent -> privileged Windows service.
5. Application/browser extension -> Password Manager native host.
6. Password Manager clients -> sync service.
7. Local files/network payloads -> encrypted format/protocol parsers.
8. Release/update metadata -> update verifier.
9. Installer/release scripts -> Windows/OS administrative boundaries.
10. Repository source/dependencies -> release artifacts.

## Attacker classes considered

- malformed or hostile local encrypted packages;
- unauthenticated or replayed local IPC requests;
- untrusted same-host processes attempting privileged-service access;
- malicious/corrupt sync or update responses;
- path traversal, symlink/reparse and no-overwrite attacks;
- stale/interrupted state replacement;
- downgrade/channel-confusion attempts;
- tampered release artifacts;
- accidental source-tree secret inclusion;
- dependency advisories and unacceptable dependency licenses;
- abusive request rates and replay attempts at privileged boundaries.

## Explicit non-goals / residual threat classes

DragonForge 1.0 does not claim to defend against:

- a compromised Windows kernel;
- a malicious local administrator;
- arbitrary memory scraping/UI automation inside the same logged-in desktop session;
- physical attacks against unlocked machines;
- hardware/firmware compromise;
- full EDR/antivirus behavior detection;
- bit-for-bit reproducible Windows PE output across different machines/toolchains;
- remote revocation of offline Secure Share content;
- bare-metal recovery of all filesystem metadata.

Those limitations must remain visible in the Phase 24 residual-risk register and release documentation.

## Audit conclusion

No new trust boundary is introduced by Phase 24. The release audit must fail closed if an existing security boundary loses its documented identity, authentication, anti-replay, format validation, no-overwrite, signing, or release-source binding invariant.
