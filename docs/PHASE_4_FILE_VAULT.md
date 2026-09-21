# Phase 4 — File Vault

## Status

**Complete**

Phase 4 adds a dedicated DragonForge File Vault product for encrypting local files and folders into authenticated `.dfvault` containers.

## Architecture

- `crates/dragonforge-file-vault/` — product-owned encrypted-container engine.
- `apps/file-vault/` — Tauri desktop application.
- Security Center marks File Vault as Integrated and can launch the co-located File Vault executable.

File Vault deliberately uses its own format rather than reusing the Password Manager vault format.

## Cryptography

Phase 4 uses AES-256-GCM for authenticated encryption and Argon2id for password-based key derivation. Each container uses a random 16-byte salt and random 12-byte AES-GCM nonce, with the encoded header authenticated as associated data. The format is explicitly versioned as `DFV1`.

## Protected metadata

The encrypted payload contains filenames, relative directory paths, directory structure, and file contents. Only the versioned header, KDF parameters, salt, nonce, and ciphertext are outside the encrypted payload.

## Safety limits

Phase 4 rejects passwords shorter than 12 bytes, symbolic-link sources, traversal-like archive paths, duplicate archive paths, more than 10,000 entries, archive paths over 4,096 bytes, more than 512 MiB of aggregate plaintext file data, existing output containers, and extraction into an existing destination.

Extraction is staged into a randomized temporary sibling directory and renamed only after all files are written successfully. Failed extraction attempts clean up the temporary directory.

## Desktop application

The File Vault app can create containers from files/folders, inspect an authenticated manifest, verify integrity, and extract into a new destination. It is local-only. Password strings owned by native commands are zeroized after each operation.

## Verification

Run on Windows:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase4-file-vault-tests.ps1
```

The script checks formatting, compile, strict Clippy, File Vault/core/Security Center tests, both JavaScript frontends, and application builds. It also builds the Password Manager desktop binary so Security Center can launch both integrated sibling applications from a clean development checkout. It writes a timestamped log and SHA-256 checksum under `test-logs/`.

## Phase 5 handoff

Phase 5 can proceed with Authenticator without changing the File Vault container format.
