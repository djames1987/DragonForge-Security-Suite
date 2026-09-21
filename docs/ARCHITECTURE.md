# DragonForge Security Suite Architecture

## 1. Architectural model

DragonForge Security Suite is a modular security platform, not one monolithic executable.

The current runtime/repository model is:

```text
                         DragonForge Security Center
                                    |
                 +------------------+------------------+
                 |                                     |
          User-facing apps                    DragonForge Agent
                 |                              (future service)
     +-----------+-----------+
     |           |           |
 Password     File Vault  Authenticator  Security Scanner
 Manager      (current)   (current)       (current)
     |
     +---- Password Manager sync service
     |
     +---- Password Manager browser extension

Shared libraries sit below applications and services:
- dragonforge-core
- dragonforge-crypto
- dragonforge-vault
- Phase 2 shared primitives inside dragonforge-core
```

The Security Center is an orchestration and visibility layer. It must not become a dumping ground for security-sensitive implementation details.

The future DragonForge Agent will be responsible for protections that need to continue when the Security Center UI is closed.

## 2. Repository zones

### apps/

Interactive end-user applications.

Current:
- `security-center/`
- `password-manager/`
- `file-vault/`
- `authenticator/`
- `security-scanner/`
- `integrity-monitor/`
- `network-guard/`

Future:
- other user-facing suite applications

### services/

Long-running or server-side processes.

Current:
- `password-manager-sync/`

Planned:
- `dragonforge-agent/`

### crates/

Reusable Rust libraries.

Current:
- `dragonforge-core`: suite-wide, non-cryptographic foundation.
- `dragonforge-file-vault`: File Vault product-owned encrypted-container engine.
- `dragonforge-authenticator`: Authenticator product-owned OTP and encrypted-store engine.
- `dragonforge-security-scanner`: Security Scanner product-owned posture assessment engine.
- `dragonforge-integrity-monitor`: Integrity Monitor product-owned baseline/change engine.
- `dragonforge-network-guard`: Network Guard product-owned visibility engine.
- `dragonforge-crypto`: migrated Password Manager cryptographic foundation.
- `dragonforge-vault`: migrated encrypted vault implementation.

Potential future crates should be created only when real cross-product requirements justify them:
- `dragonforge-storage`
- `dragonforge-ipc`
- `dragonforge-policy`
- `dragonforge-events`
- `dragonforge-platform`

`dragonforge-core` owns the small Phase 2 cross-product foundation: component IDs, configuration metadata, safe error/event types, platform path discovery, logging/redaction policy, and transport-neutral IPC authorization metadata. It must not absorb cryptographic code simply because multiple products need it.

### extensions/

Current:
- `password-manager-browser/`

Future separately packaged integrations should remain isolated from trusted native components.

### docs/

Architecture, threat model, migration records, roadmap, ADRs, release/security procedures, and product-specific documentation.

### scripts/

Suite-level development/validation tools and namespaced product-specific tooling.

Current:
- `scripts/password-manager/`

## 3. Trust boundaries

The architecture treats the following as distinct trust boundaries:

1. UI input versus security-sensitive core logic.
2. Security Center versus the future background agent.
3. Local applications versus sync/network services.
4. Browser extension versus native application.
5. Untrusted serialized data versus validated domain objects.
6. Operating-system APIs versus portable business logic.
7. Shared libraries versus product-specific policy.

Every future IPC/network boundary should authenticate its peer and validate all input.

## 4. Dependency direction

Preferred dependency direction:

```text
apps/services
    |
    v
product-domain crates
    |
    v
shared suite crates
    |
    v
well-vetted external dependencies / OS APIs
```

Shared crates must not depend on application UI crates.

## 5. Password Manager placement

Phase 1 migrated the tested standalone Password Manager into these suite paths:

```text
apps/desktop                  -> apps/password-manager/
apps/sync-server              -> services/password-manager-sync/
apps/browser-extension        -> extensions/password-manager-browser/
crates/dragonforge-crypto     -> crates/dragonforge-crypto/
crates/dragonforge-vault      -> crates/dragonforge-vault/
docs                          -> docs/password-manager/
scripts                       -> scripts/password-manager/
```

The migrated components passed the post-migration Phase 11 regression gate before merge.

## 6. Migration rule retained after Phase 1

The migration followed **move first, refactor second**:

1. Freeze a tested standalone Password Manager commit.
2. Record its source SHA.
3. Import the product into the suite.
4. Adapt paths/workspace metadata only where necessary.
5. Run the original Password Manager regression suite.
6. Resolve migration-only failures.
7. Establish a passing suite baseline.
8. Only then allow shared-component extraction.

That sequence is complete. Phase 2 added only non-cryptographic cross-product primitives and did not refactor Password Manager security-sensitive behavior.

Crypto, vault format, sync protocol, key derivation, recovery, and serialization changes remain separate security-sensitive work and must not be bundled casually into architectural refactors.

## 7. Versioning

During early development:
- suite workspace: `0.x`;
- applications may release independently;
- persistent formats and IPC/network protocols require explicit format/protocol versions.

Repository version and encrypted-data format version must never be assumed to be the same thing.

## 8. ADRs

Significant decisions should be recorded under `docs/adr/`, including:
- cryptography ownership;
- IPC transport and authentication;
- Windows service model;
- event database format;
- update signing;
- plugin model;
- cross-platform strategy.


## 9. Phase 2 shared foundation

Phase 2 established the following modules in `dragonforge-core`:

- `config` — validated stable keys, component-scoped environment names, and configuration source metadata.
- `error` — stable error categories plus messages explicitly designated safe for logs/UI.
- `event` — cross-product severity, event categories, and redaction-safe event records.
- `platform` — operating-system identification and conventional per-user suite paths without creating directories.
- `redaction` — formatting-safe secret wrappers and conservative logging policy.
- `ipc` — versioned request envelopes, transport-authentication metadata, and fail-closed destination/caller authorization policy.

The IPC module is intentionally transport-neutral. It does not generate credentials, open sockets/pipes, or claim that a peer is authenticated. The platform transport must verify a peer first and only then construct an authenticated peer context.

If any Phase 2 module grows into a large subsystem or gains security-sensitive dependencies, it should be split into a dedicated crate through a separate ADR.


## 10. Phase 3 Security Center

Security Center is now a real Tauri desktop application under `apps/security-center/`.

Its internal boundaries are:

- `model` — component registry and aggregate suite health.
- `events` — bounded, redaction-safe in-memory dashboard activity.
- `settings` — versioned local settings stored under the user-specific Security Center configuration directory.
- `logging` — conservative local diagnostic log writer using the shared LogPolicy contract.
- `agent` — explicit unavailable-agent client and validation against the shared IPC policy.
- `orchestration` — strict sibling-process launch for integrated suite applications.
- `state` — synchronized application state exposed to Tauri commands.
- `ui` — static HTML/CSS/JavaScript dashboard.

### Security Center trust rules

1. The browser/webview UI does not directly perform filesystem or process operations.
2. Tauri commands are the native boundary for dashboard actions.
3. Integrated application launching resolves only exact expected executables beside the running Security Center binary. It does not search PATH or execute a user-supplied path.
4. Settings contain preferences only; passwords, keys, tokens, vault content, or recovery material must never be stored there.
5. Activity records and log messages must use safe summaries and the shared redaction policy.
6. The future Agent remains unavailable until an authenticated OS transport and service are implemented. The dashboard must not infer an authenticated peer from UI or payload data.


## 11. Phase 4 File Vault

File Vault is split into a product-owned container engine and a Tauri desktop UI:

- `crates/dragonforge-file-vault/` owns the `.dfvault` format, cryptographic envelope, validation, limits, and extraction rules.
- `apps/file-vault/` exposes only narrow native commands for create, inspect, verify, and extract operations.
- Security Center marks File Vault as Integrated and launches only the expected sibling executable.

The File Vault format is deliberately independent from the Password Manager vault format. Phase 4 does not refactor or broaden Password Manager cryptographic APIs.

### Container trust rules

1. Filenames, relative paths, directory structure, and file contents are inside one AES-256-GCM authenticated ciphertext.
2. Header/version/KDF parameters are authenticated as associated data.
3. Argon2id parameters are encoded and validated as part of the format contract.
4. Symbolic links and traversal-like paths are rejected.
5. Creation and extraction do not overwrite existing destinations.
6. Extraction occurs in a randomized temporary sibling directory and is renamed into place only after success.
7. File Vault passwords are never persisted by the File Vault application.


## 12. Phase 5 Authenticator

Authenticator is split into a product-owned OTP/storage engine and a Tauri desktop UI:

- `crates/dragonforge-authenticator/` owns TOTP/HOTP generation, otpauth parsing, encrypted store format, account validation, HOTP counters, and recovery-code storage.
- `apps/authenticator/` exposes narrow native commands for store creation/unlock, import, account management, code generation, and recovery-code management.
- Security Center marks Authenticator as Integrated and launches only the expected sibling executable.

Authenticator secrets are not moved into `dragonforge-core`, Password Manager storage, or File Vault containers. Its `.dfauth` format has a separate version and lifecycle.

### Authenticator trust rules

1. OTP secrets and recovery codes are encrypted at rest.
2. Ordinary account listings never expose OTP secrets or recovery-code values.
3. Recovery codes require an explicit reveal action.
4. HOTP counters are advanced and persisted only through the HOTP consume action.
5. The app does not claim hardware-backed credential support in Phase 5.
6. The local master password is command-scoped and not persisted by the application.


## 13. Phase 6 Security Scanner

Security Scanner is split into a product-owned posture engine and a Tauri desktop UI:

- `crates/dragonforge-security-scanner/` owns fixed platform probes, result semantics, evidence bounding, and posture findings.
- `apps/security-scanner/` exposes only scanner metadata and one read-only scan command.
- Security Center marks Security Scanner as Integrated and launches only the expected sibling executable.

Phase 6 is intentionally a one-shot assessment component, not a privileged background monitor. Continuous baseline/change monitoring remains Phase 7 Integrity Monitor work.

### Scanner trust rules

1. The webview cannot supply shell commands, PowerShell fragments, registry paths, executable paths, or remediation instructions.
2. Windows probes are fixed in native Rust code and execute without requested elevation.
3. Probe failure or insufficient visibility produces an Unknown result rather than a fabricated pass.
4. Scanner evidence is bounded and avoids Password Manager, File Vault, and Authenticator secret material.
5. Phase 6 does not mutate firewall, encryption, update, Defender, UAC, SMB1, Remote Desktop, service, or network configuration.
6. Listening ports are posture evidence, not proof of vulnerability.
7. Latest-hotfix metadata is informational and does not claim that no newer update is available.


## 14. Phase 7 Integrity Monitor

Integrity Monitor is split into a product-owned baseline/change engine and a Tauri desktop UI:

- `crates/dragonforge-integrity-monitor/` owns baseline format, collection limits, hashing, validation, persistence, and comparison semantics.
- `apps/integrity-monitor/` exposes narrow commands for baseline status, baseline creation/replacement, and on-demand integrity comparison.
- Security Center marks Integrity Monitor as Integrated and launches only the expected sibling executable.

Phase 7 remains user-session scoped and on-demand. It does not claim continuous monitoring when the UI is closed; that requires the future DragonForge Agent.

### Integrity trust rules

1. The webview cannot submit PowerShell, arbitrary registry paths, service/task names, filesystem roots, or remediation commands.
2. Fixed Windows probes are defined in native code.
3. Baselines persist identifiers and SHA-256 fingerprints, not monitored command contents or file contents.
4. Baseline files are versioned, bounded, validated, and reject symbolic-link targets.
5. Baseline replacement is explicit because it resets the trusted comparison reference.
6. Startup collection skips symbolic links and limits recursion, file count, and hashed file size.
7. Probe/collection failures mark affected surfaces unavailable so they do not create false removal findings.
8. Detected differences are review signals, not malware verdicts.


## 15. Phase 8 Network Guard

Network Guard is split into a product-owned visibility engine and a Tauri desktop UI:

- `crates/dragonforge-network-guard/` owns bounded Windows TCP, UDP, and DNS-cache collection plus normalization and summary semantics.
- `apps/network-guard/` exposes narrow commands for product metadata and refreshing an on-demand network snapshot.
- Security Center marks Network Guard as Integrated and launches only the expected sibling executable.

Phase 8 is intentionally visibility-only. Persistent application-level traffic enforcement requires a background service that can survive UI closure and a carefully reviewed privilege boundary; that responsibility remains with the future DragonForge Agent.

### Network Guard trust rules

1. The webview cannot submit PowerShell, process IDs, commands, firewall rules, packet filters, or arbitrary probe parameters.
2. Native probes are fixed and invoke PowerShell with `-NoProfile -NonInteractive`.
3. Probe output and row counts are bounded.
4. Network Guard reads endpoint/process metadata and DNS cache records; it does not capture packet payloads.
5. Phase 8 does not terminate processes/connections or mutate Windows Firewall.
6. Wildcard-listener status is visibility context, not a vulnerability verdict.
7. Probe failures surface warnings instead of being treated as a healthy/empty network.
