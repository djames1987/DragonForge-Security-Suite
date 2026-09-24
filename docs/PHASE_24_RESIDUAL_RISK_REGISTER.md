# Phase 24 — Residual Risk Register

This register is part of the DragonForge Security Suite 1.0 release decision.

| ID | Residual risk | 1.0 disposition |
| --- | --- | --- |
| R24-01 | Same-user malware or a local administrator can attack process memory, UI automation, user-owned files, or installed binaries. | Accepted / documented. DragonForge does not claim hostile-admin or kernel resistance. |
| R24-02 | Windows user-scoped data may rely on inherited profile ACLs outside privileged-service state. | Accepted / documented. Secret-at-rest protection remains cryptographic where designed. |
| R24-03 | Authenticode reputation/SmartScreen reputation is external to signature validity and may be immature for a new publisher. | Accepted / operational. Signatures/timestamps remain mandatory for stable releases. |
| R24-04 | Windows binary builds are not claimed bit-for-bit reproducible across toolchains because PE metadata and RFC 3161 timestamps vary. | Accepted. Exact source archive reproducibility, tag/commit binding, artifact hashes, and signatures are required instead. |
| R24-05 | Secure Share offline expiration depends on recipient clock and cannot revoke already distributed data. | Accepted / documented product limitation. |
| R24-06 | Backup/Recovery does not provide bare-metal imaging or preserve every ACL/reparse/filesystem semantic. | Accepted / documented product scope. |
| R24-07 | User-selected external storage can be deleted, rolled back, corrupted, or denied outside DragonForge control. | Accepted / documented. |
| R24-08 | Accessibility implementation is not a formal third-party WCAG certification. | Accepted with Phase 24 hands-on acceptance requirement. |
| R24-09 | Dependency/advisory/license state can change after release. | Accepted only with ongoing scheduled RustSec and release-time audit gates. |
| R24-10 | A signing-key compromise can undermine update/release trust until key rotation/revocation response completes. | Accepted operational risk with documented external key custody and rotation/revocation procedures. |

## Release blockers

Phase 24 must not be marked Verified Complete if any of the following are present:

- known unreviewed RustSec advisory applicable to shipped code;
- missing/unreviewed dependency license metadata;
- failing workspace tests or strict Clippy;
- generic privileged command execution;
- loss of privileged caller path/signature/publisher validation;
- update downgrade/channel/signature fail-open behavior;
- stable release path that permits unsigned artifacts;
- release publication from dirty or non-tag-matching source;
- artifact/hash/signature verification bypass;
- detected private key material committed to tracked source;
- unresolved critical/high security defect without explicit mitigation.
