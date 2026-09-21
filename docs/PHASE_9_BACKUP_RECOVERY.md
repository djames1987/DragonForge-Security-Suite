# Phase 9 — Backup & Recovery

## Status

**Implementation Complete — Local Verification Pending**

Phase 9 adds local encrypted backup and recovery for DragonForge suite data and user-selected files/folders.

## Delivered

- dedicated `dragonforge-backup-recovery` engine crate;
- dedicated Tauri desktop application under `apps/backup-recovery/`;
- versioned `.dfbackup` encrypted container format;
- AES-256-GCM authenticated encryption for backup metadata, paths, hashes, and file bytes;
- Argon2id password-based key derivation with Phase 9 format-owned parameters;
- SHA-256 per-entry integrity manifest inside the authenticated ciphertext;
- suite config/data root discovery plus arbitrary user-selected local file/folder sources;
- bounded file count, individual file size, aggregate data size, path length, and archive size;
- symbolic-link and traversal-like path rejection;
- wrong-password, malformed-format, duplicate-path, and tamper rejection;
- explicit inspect and full verify operations;
- restore into a new destination only, with no overwrite behavior;
- staged restore through a randomized sibling temporary directory before final rename;
- Security Center integration using the exact sibling executable path;
- Phase 9 CI and Windows local verification tooling;
- suite-wide all-app build extended to include Backup & Recovery.

## Backup format

The Phase 9 format uses the extension:

`.dfbackup`

The format is intentionally independent from `.dfvault`, Password Manager vault formats, and Authenticator storage. A repository/application version is not treated as the backup-format version.

The authenticated encrypted payload contains:

- format version;
- backup creation timestamp;
- source-selection metadata;
- normalized archive-relative paths;
- original source paths;
- per-file byte lengths;
- per-file SHA-256 digests;
- file contents.

The password is never written into the backup or persisted by the desktop application.

## Restore safety

Restore is deliberately conservative:

1. the backup password must decrypt the authenticated payload;
2. the payload version and limits are validated;
3. every archive path must remain relative and traversal-free;
4. duplicate paths are rejected;
5. every file is decoded and checked against its stored length and SHA-256 digest;
6. the requested destination must not already exist;
7. files are written under a randomized sibling temporary directory;
8. the temporary directory is renamed into the final destination only after the complete restore succeeds;
9. a failed restore attempts to remove its temporary directory.

Phase 9 does not merge restored files into an existing tree and does not overwrite existing user data.

## Scope limits

Phase 9 is a local backup/recovery product, not a disk-image or bare-metal restore system.

It does not currently preserve:

- Windows ACLs or ownership metadata;
- alternate data streams;
- reparse points or symbolic links;
- filesystem timestamps;
- installed applications or the Windows operating system;
- live database-consistency snapshots for applications that are actively writing.

Users should close DragonForge applications that are actively modifying data before creating a suite-data backup.

## Verification

Run on Windows:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase9-backup-recovery-tests.ps1
```

The verifier covers:

- `cargo fmt --all --check`;
- compile checks for the current suite applications/crates;
- strict Clippy with warnings denied;
- Phase 9 engine/application tests;
- Phase 8 through Phase 3 regression tests;
- JavaScript syntax checks for all current suite UIs;
- the all-app desktop build and executable verification.

A passing local verifier log and SHA-256 file are required before Phase 9 is marked **Verified Complete**.

## Phase 10 handoff

Phase 10 is Secure Share: encrypted recipient-oriented packages/secrets with expiration and recipient-oriented controls.
