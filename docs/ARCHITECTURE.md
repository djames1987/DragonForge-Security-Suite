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
 Password     File Vault  Authenticator
 Manager      (future)    (future)
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

Future:
- File Vault
- Authenticator
- Security Scanner
- Network Guard UI
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
