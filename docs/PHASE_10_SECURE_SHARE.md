# Phase 10 — Secure Share

## Status

**Implementation Complete — Local Verification Pending**

Phase 10 adds offline encrypted sharing for recipient-oriented secrets and files.

## Delivered

- dedicated `dragonforge-secure-share` engine crate;
- dedicated Tauri desktop application under `apps/secure-share/`;
- versioned `.dfshare` encrypted package format;
- AES-256-GCM authenticated encryption;
- Argon2id password-based key derivation;
- encrypted sender label, recipient label, expiration timestamp, secret text, attachment paths, hashes, and attachment bytes;
- optional protected secret text and optional file/folder attachments;
- required recipient label and optional sender label;
- package expiration with native reveal/extract enforcement;
- integrity verification that remains available after expiration;
- SHA-256 verification for every decrypted attachment;
- bounded labels, secret size, file count, individual file size, total attachment size, path length, and package size;
- symbolic-link, traversal-like path, duplicate-path, malformed-format, wrong-password, and ciphertext-tamper rejection;
- no-overwrite staged attachment extraction;
- Security Center integration with exact sibling executable launch;
- Phase 10 CI and Windows local verification tooling;
- suite-wide test build extended to nine desktop applications.

## Package format

Secure Share packages use the extension:

`.dfshare`

The format is independent from Password Manager, File Vault, Authenticator, and Backup & Recovery persistent formats.

The authenticated encrypted payload includes:

- format version;
- creation and expiration timestamps;
- sender and recipient labels;
- optional secret text;
- normalized attachment paths;
- attachment lengths and SHA-256 digests;
- attachment contents.

The password is not persisted by the application or stored inside the package.

## Expiration behavior

Expiration is checked by the native engine before protected content access.

- Verification may still authenticate and inspect an expired package.
- Secret reveal fails after expiration.
- Attachment extraction fails after expiration.
- Expiration is limited to 366 days from package creation.

Phase 10 uses the local system clock. It does not claim tamper-resistant time enforcement against a user who controls and changes that clock.

## Recipient-oriented controls

The encrypted recipient label helps the sender and recipient identify package intent after successful decryption. It is not a cryptographic identity proof.

Phase 10 is password-based and offline. The package password should be transferred to the intended recipient through a separate trusted channel.

Because an offline file can be copied without the sender's knowledge, Phase 10 deliberately does not claim:

- remote revocation;
- guaranteed deletion after expiration;
- reliable one-time-open or open-count enforcement;
- server-enforced recipient identity;
- delivery tracking.

Those capabilities require a future online coordination service and a separate authentication/trust model.

## Extraction safety

1. the package must authenticate successfully;
2. format and bounded limits are validated;
3. every archive path must remain relative and traversal-free;
4. duplicate paths are rejected;
5. every attachment is checked against its encrypted length and SHA-256 digest;
6. expiration must not have passed;
7. the destination must not already exist;
8. extraction is staged into a randomized sibling temporary directory;
9. the temporary directory is renamed into place only after success;
10. failed extraction attempts clean up the temporary directory.

## Verification

Run on Windows:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase10-secure-share-tests.ps1
```

The verifier covers formatting, workspace compile checks, strict Clippy, Secure Share tests, prior-suite regression tests, JavaScript syntax checks, and the complete nine-application desktop build.

A passing local verifier log and SHA-256 sidecar are required before Phase 10 is marked **Verified Complete**.

## Phase 11 handoff

Phase 11 is DragonForge Agent: authenticated local IPC plus narrowly scoped background monitoring/protection that must continue when suite UIs are closed.
