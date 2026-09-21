# Security Model

## Assets to protect

DragonForge components may eventually handle:
- passwords and passkeys;
- encryption keys;
- TOTP seeds and recovery codes;
- encrypted vaults/files;
- sync credentials/tokens;
- security events and policy;
- backup recovery material.

## Primary threats

The suite architecture should account for:
- malicious local processes;
- compromised browser/web content;
- untrusted network peers;
- corrupted or malicious serialized data;
- dependency/supply-chain compromise;
- accidental secret disclosure through logs/crash reports;
- downgrade/rollback attacks;
- tampered updates;
- unauthorized local IPC clients;
- privilege-boundary mistakes.

## Baseline controls

- authenticated encryption for protected data;
- strong, domain-separated key derivation;
- explicit format/protocol versions;
- strict input validation;
- secret redaction;
- least privilege;
- minimal dependency surface;
- fail-closed authorization;
- authenticated local IPC where sensitive actions cross process boundaries;
- reproducible/verifiable test procedures;
- signed releases/updates when distribution begins.

## Cryptographic ownership

The suite must have one clearly documented source of truth for each cryptographic primitive and data format.

The existing Password Manager cryptographic crate is considered product-proven code, not automatically a suite-wide API. After migration it can be evaluated for broader use under a separate security review.

## Logs and diagnostics

Logs must never intentionally contain:
- master passwords;
- plaintext vault items;
- TOTP seeds;
- recovery keys;
- encryption keys;
- authentication tokens;
- full sensitive payloads.

Identifiers should be minimized or pseudonymized where practical. The shared LogPolicy defaults to redacting identifiers and bounds/sanitizes public fields; Secret values format only as [REDACTED].

## Privilege model

Most UI applications should run as the normal user.

The future DragonForge Agent may need elevated capabilities for selected monitoring/enforcement features. Elevated code must be kept narrow and expose a small authenticated command surface.

## Network model

Components should default to local-only operation unless network functionality is explicitly enabled.

Sync and remote services must assume the transport/network is hostile and should minimize server visibility into user secrets.

## Update model

Before public production distribution:
- releases should be signed;
- update metadata must be authenticated;
- rollback/downgrade behavior must be defined;
- failed updates must be recoverable.

## Security testing

Security-sensitive changes should include:
- unit tests;
- negative/tamper tests;
- format/version compatibility tests;
- migration tests where persistent data changes;
- fuzzing/property testing where practical;
- dependency audit;
- documented manual verification for OS-specific behavior.


## Local IPC foundation

Phase 2 defines transport-neutral authenticated IPC metadata and authorization rules.

Security requirements:
- unauthenticated peers fail closed;
- the authenticated component identity must match the envelope source;
- requests for the wrong destination are rejected;
- callers must appear in the destination allow-list;
- incompatible protocol major versions are rejected;
- actual peer verification remains the responsibility of the selected OS transport or session-authentication mechanism.

Constructing an authenticated peer context is a trust-boundary operation. Future pipe/socket implementations must not accept caller-supplied component identity without independent verification.


## Security Center native boundary

Phase 3 adds a Tauri webview dashboard backed by Rust commands.

Security requirements:
- webview code must not receive arbitrary filesystem/process primitives;
- native commands must validate inputs and expose only narrowly-scoped actions;
- Password Manager launch resolves only the expected executable beside Security Center;
- settings are preferences, not a secret store;
- dashboard events and log messages use safe/redaction-oriented text;
- invalid local settings fall back to safe defaults instead of disabling the dashboard;
- the future DragonForge Agent remains explicitly unavailable until a real authenticated transport/service exists.

The current in-memory activity view is not a tamper-resistant audit log and must not be represented as one.


## File Vault container boundary

Phase 4 introduces the `.dfvault` local encrypted-container format.

Security requirements:
- AES-256-GCM authenticated encryption protects the entire logical archive payload;
- Argon2id derives the container key from a user password;
- filenames and directory structure are encrypted along with file contents;
- format version and KDF parameters are explicit and authenticated;
- symbolic links, traversal components, absolute archive paths, duplicate paths, and oversized inputs are rejected;
- existing container/extraction destinations are never overwritten;
- extraction is staged into a temporary sibling directory and cleaned up on failure;
- File Vault passwords are not stored in settings, logs, or container metadata.

The current Phase 4 implementation is a local container product, not a secure-deletion tool and not a replacement for full-disk encryption. Plaintext source files remain under the user's control after container creation.
