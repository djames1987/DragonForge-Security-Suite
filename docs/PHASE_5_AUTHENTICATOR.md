# Phase 5 — Authenticator

## Status

**Verified Complete**

Phase 5 adds DragonForge Authenticator as a dedicated local TOTP/HOTP application with its own encrypted store.

## Delivered

- product-owned `dragonforge-authenticator` Rust crate;
- Tauri desktop application under `apps/authenticator/`;
- RFC 4226 HOTP generation;
- RFC 6238 TOTP generation;
- SHA-1, SHA-256, and SHA-512 OTP algorithms;
- 6- and 8-digit OTP support;
- `otpauth://totp` and `otpauth://hotp` URI import;
- manual TOTP/HOTP account entry;
- encrypted recovery-code storage and explicit reveal workflow;
- persistent HOTP counter advancement;
- encrypted store master-password rotation with fresh salt/nonce;
- explicit local lock action that clears the retained password, displayed codes, and revealed recovery material;
- Security Center integration and strict sibling executable launch;
- local Phase 5 verification script and CI coverage.

## Encrypted store

The Authenticator store uses the `.dfauth` format and is intentionally separate from the Password Manager and File Vault formats.

The encrypted payload contains:
- account labels and issuers;
- Base32 OTP secrets;
- OTP algorithm/digit/type parameters;
- HOTP counters;
- recovery codes.

The store uses AES-256-GCM authenticated encryption and Argon2id password derivation. The version/KDF header is authenticated as associated data. A new random salt and nonce are used on each write.

The application does not persist the master password. The user can rotate the master password; the store is decrypted with the current password and re-encrypted with a fresh salt and nonce. Password strings owned by native Tauri commands are zeroized after each command.

## Validation and limits

Phase 5 rejects:
- master passwords shorter than 12 characters;
- invalid/too-short Base32 secrets;
- unsupported OTP algorithms;
- unsupported OTP digit counts;
- unreasonable TOTP periods;
- malformed or unsupported `otpauth://` URIs;
- stores larger than 8 MiB;
- more than 500 accounts;
- more than 100 recovery codes per account;
- oversized or multiline recovery-code entries.

Encrypted-store replacement uses a temporary file plus a recoverable backup path so an interrupted replacement can restore the previous store.

## Recovery-code behavior

Recovery codes are encrypted inside the Authenticator store and are not returned in ordinary account listings. The user must explicitly choose **Reveal** for a selected account.

Phase 5 does not claim to synchronize or securely share recovery codes. Those concerns remain separate from local Authenticator storage.

## Verification

Run on Windows:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase5-authenticator-tests.ps1
```

The verifier runs rustfmt, compile checks, strict Clippy, Authenticator/File Vault/Security Center regression tests, JavaScript syntax checks, and builds all integrated desktop applications so Security Center can launch them as sibling executables.

Authoritative Windows verification passed on 2026-09-21 using `dragonforge-phase5-authenticator-20260921-105934.log`.

## Phase 6 handoff

Phase 6 is Security Scanner. It should consume shared non-secret platform primitives where appropriate but remain isolated from Authenticator secrets and encrypted storage.
