# Phase 15 — Password Manager Ecosystem Production Hardening

**Status: Implementation Complete — Local Verification Pending**

Phase 15 hardens the Password Manager desktop/browser/sync ecosystem for production deployment without changing vault cryptography or the zero-knowledge server model.

## Delivered

- production sync mode that fails closed unless PostgreSQL persistence is compiled/configured;
- production-only 64-hex-character admin-token policy;
- explicit TLS reverse-proxy declaration and HTTPS public-base URL requirement;
- bounded request-rate windows keyed by hashed credential/device identity with bounded limiter state;
- existing 64 MiB body and 30 second request timeout retained;
- no-store, nosniff, and no-referrer HTTP response headers;
- protocol compatibility range published by the health endpoint;
- browser-extension packaging corrected to the migrated extension path and broad-permission checks added;
- native host origin parser tightened to exact Chrome/Edge extension-ID form;
- hardened Docker service defaults using read-only filesystem, dropped capabilities, no-new-privileges, and tmpfs;
- PostgreSQL backup/restore tooling with SHA-256 evidence and explicit destructive-restore acknowledgement;
- migration/protocol compatibility matrix and production deployment guidance;
- Phase 15 local verification harness and CI coverage.

## Protocol compatibility

Current server protocol: **2**. Minimum supported protocol: **2**. Maximum supported protocol: **2**.

Client sync sidecar versions 1 and 2 are migrated locally to the current sidecar format before normal use. Server database migrations remain forward-only and are applied by SQLx when PostgreSQL is opened.

A server/client protocol range mismatch must be treated as incompatible rather than silently guessing compatibility.

## Production boundary

The Rust sync service itself is HTTP and is intended to sit behind a trusted TLS reverse proxy. Production startup requires `DRAGONFORGE_SYNC_TLS_PROXY=true` and an `https://` public base URL. Do not expose the raw sync listener directly to the Internet.

The server remains zero-knowledge with respect to vault plaintext. Database backups still contain sensitive account/device metadata and encrypted vault/recovery material and must be protected accordingly.

## Recovery and migration

Use `scripts/password-manager/backup-sync-server.ps1` before upgrades or migration. Restore requires `-AcknowledgeDataReplacement` and stops the sync container while PostgreSQL is restored. Always retain the backup and its SHA-256 sidecar until post-migration verification is complete.
