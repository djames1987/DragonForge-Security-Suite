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

The Phase 11 DragonForge Agent runs as the normal user. Future selected monitoring/enforcement features may require a separate elevated Windows service; any elevated code must remain narrow and expose a small authenticated command surface.

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


## Authenticator secret boundary

Phase 5 introduces the local `.dfauth` encrypted Authenticator store.

Security requirements:
- OTP secrets and recovery codes are encrypted with AES-256-GCM;
- the store key is derived with Argon2id;
- store/KDF version metadata is authenticated as associated data;
- account list responses omit OTP secrets and recovery-code contents;
- recovery codes require an explicit reveal operation;
- HOTP counter updates are persisted in the encrypted store;
- malformed OTP URIs, weak/invalid secrets, unsupported parameters, and oversized stores are rejected;
- Authenticator master passwords are not stored in settings or logs.

Authenticator is not a hardware-backed credential vault in Phase 5. Hardware-key/passkey support requires a later threat model and OS/hardware integration review.


## Security Scanner observation boundary

Phase 6 introduces a read-only Windows-first posture scanner.

Security requirements:
- platform probes are fixed native code; the webview cannot inject shell fragments or select arbitrary commands;
- the scanner does not request administrator elevation or automatically remediate findings;
- inaccessible or unsupported checks return Unknown instead of being treated as secure;
- listener enumeration is bounded and reports local exposure context without claiming that an open port is inherently vulnerable;
- update metadata is advisory and does not claim completeness of Windows Update availability;
- scanner output must not read, include, or inspect Password Manager secrets, File Vault plaintext, Authenticator seeds, or recovery codes;
- Security Center starts the scanner only by its exact co-located sibling executable path.

The scanner is not a traditional antivirus engine, exploit scanner, EDR agent, or authenticated vulnerability-feed service. Those capabilities are outside the Phase 6 trust boundary.


## Integrity Monitor baseline boundary

Phase 7 introduces local integrity baselines and on-demand change detection.

Security requirements:
- collection is limited to fixed Windows persistence/configuration surfaces and bounded Startup/hosts file hashing;
- the UI cannot inject shell commands, registry locations, service/task selectors, or arbitrary filesystem paths;
- persisted baselines contain identifiers and SHA-256 fingerprints rather than monitored command or file contents;
- malformed, oversized, duplicate-entry, unsupported-version, and symlink baseline inputs are rejected;
- baseline replacement is explicit and uses staged replacement rather than in-place partial writes;
- unavailable collection surfaces are tracked and excluded from comparison so probe failure does not become a false removal;
- a fingerprint difference is evidence of change only and must not be represented as proof of compromise;
- Phase 7 does not remediate changes, quarantine files, or provide privileged/tamper-resistant monitoring.

Continuous monitoring outside the desktop application remains a future DragonForge Agent responsibility.


## Network Guard observation boundary

Phase 8 introduces Windows-first, on-demand network visibility.

Security requirements:
- TCP, UDP, and DNS-cache probes are fixed native commands; the UI cannot inject shell fragments or arbitrary selectors;
- collection is bounded by probe output size and result count;
- endpoint inventory may include process names, local/remote addresses, ports, TCP state, and DNS cache records needed for local visibility;
- packet payloads, credentials, Password Manager secrets, Authenticator seeds, and File Vault plaintext are not inspected;
- Phase 8 does not request elevation, block traffic, kill processes, terminate connections, or alter firewall rules;
- wildcard listeners and remote endpoints are contextual visibility, not proof of compromise;
- probe failures are surfaced as warnings rather than silently represented as an empty network;
- Security Center starts Network Guard only by its exact co-located sibling executable path.

Persistent enforcement belongs behind the future DragonForge Agent's authenticated local IPC and narrowly scoped privilege boundary.


## Backup & Recovery encrypted archive boundary

Phase 9 introduces the local `.dfbackup` encrypted backup format.

Security requirements:
- AES-256-GCM authenticated encryption protects backup metadata, source/path information, integrity hashes, and file contents;
- Argon2id derives the backup key from a user-supplied password;
- the backup format is explicitly versioned and owns its KDF parameters;
- the application does not persist backup passwords in settings, logs, or archive metadata;
- symbolic links, absolute/traversal-like archive paths, duplicate paths, malformed encodings, unsupported versions, and oversized inputs are rejected;
- full verification recomputes SHA-256 for every decrypted file entry before reporting a verified backup;
- wrong-password and ciphertext-tamper failures are indistinguishable at the public error boundary;
- restore requires a non-existing destination and never merges with or overwrites existing data;
- restore writes into a randomized sibling temporary directory and cleans it up on failure before final rename;
- Security Center starts Backup & Recovery only by its exact co-located sibling executable path.

Phase 9 does not provide secure deletion, filesystem ACL/owner preservation, reparse-point backup, bare-metal recovery, or live application-consistent snapshots. Applications that are actively modifying protected data should be closed before backup.


## Secure Share offline package boundary

Phase 10 introduces the local `.dfshare` encrypted package format.

Security requirements:
- AES-256-GCM authenticated encryption protects recipient/sender labels, expiration, secret text, attachment paths, integrity hashes, and attachment contents;
- Argon2id derives the package key from a user-supplied password;
- package passwords are not persisted in settings, logs, or package metadata;
- the recipient label is encrypted context, not proof of recipient identity;
- expiration is checked by native reveal/extract operations, while integrity verification remains available after expiration;
- expiration relies on the local system clock and therefore is not tamper-resistant against clock rollback by a controlling local user;
- symbolic links, absolute/traversal-like attachment paths, duplicate paths, malformed encodings, unsupported versions, wrong passwords, tampering, and oversized inputs are rejected;
- attachment extraction requires a non-existing destination and uses staged temporary-directory finalization;
- Security Center starts Secure Share only by its exact co-located sibling executable path.

Offline `.dfshare` files can be copied. Phase 10 therefore does not claim remote revocation, guaranteed deletion after expiration, reliable one-time-open/open-count enforcement, server-authenticated recipients, or delivery tracking. Those properties require an online service with a separate identity, authorization, and availability threat model.


## DragonForge Agent local IPC boundary

Phase 11 introduces a real per-user background Agent.

Security requirements:
- the Agent listens only on IPv4 loopback;
- each Agent lifetime gets a fresh random 256-bit session credential;
- request and response payloads are authenticated with HMAC-SHA256;
- request timestamps are accepted only within a bounded freshness window;
- random nonces are tracked in a bounded replay cache;
- authenticated source/destination metadata must still satisfy the Phase 2 `LocalIpcPolicy`;
- runtime descriptors do not contain the session credential;
- runtime/credential/lock files are kept in the current user's DragonForge data directory;
- messages and socket operations are bounded;
- invalid clients fail per connection rather than terminating the Agent;
- the exposed Phase 11 action surface is fixed to health/status only;
- Security Center launches only the exact co-located Agent executable.

The Phase 11 session credential is designed to reject unauthenticated local socket clients and spoofed protocol traffic. It is not a defense against a process that has already compromised the same user account and can read that user's runtime files. Phase 11 does not claim elevated/tamper-resistant enforcement.
