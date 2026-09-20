# DragonForge Security Suite Architecture

## 1. Architectural model

DragonForge Security Suite is a modular security platform, not one monolithic executable.

The intended runtime model is:

```text
                         DragonForge Security Center
                                    |
                 +------------------+------------------+
                 |                                     |
          User-facing apps                    DragonForge Agent
                 |                                     |
     +-----------+-----------+              +----------+----------+
     |           |           |              |          |          |
 Password     File Vault  Authenticator  Integrity  Network   Future guards
 Manager                                Monitor     Guard
     |
     +---- optional sync service / browser extension

Shared libraries sit below applications and services:
- dragonforge-core
- future shared crypto/storage/IPC/policy crates
```

The Security Center is an orchestration and visibility layer. It must not become a dumping ground for security-sensitive implementation details.

The future DragonForge Agent is responsible for protections that need to continue when the Security Center UI is closed.

## 2. Repository zones

### apps/
Interactive end-user applications.

Initial:
- `security-center`

Reserved for later:
- Password Manager desktop application
- File Vault
- Authenticator
- Security Scanner
- Network Guard UI

### services/
Long-running or server-side processes.

Planned:
- `dragonforge-agent`
- Password Manager sync service, after migration

### crates/
Reusable Rust libraries.

Current:
- `dragonforge-core`: suite-wide, non-cryptographic foundation.

Planned only after requirements are proven:
- `dragonforge-crypto`
- `dragonforge-storage`
- `dragonforge-ipc`
- `dragonforge-policy`
- `dragonforge-events`
- `dragonforge-platform`

The existing Password Manager already contains `dragonforge-crypto` and `dragonforge-vault`. Those crates are intentionally **not copied or recreated here yet**. They should be migrated from a known-good Password Manager commit, then refactored only after tests pass inside the suite.

### extensions/
Non-Rust or separately packaged integrations such as the Password Manager browser extension.

### docs/
Architecture, threat model, migration plan, roadmap, ADRs, release/security procedures.

### scripts/
Workspace validation, developer tooling, packaging, release and migration helpers.

## 3. Trust boundaries

The architecture treats the following as distinct trust boundaries:

1. UI input versus security-sensitive core logic.
2. Security Center versus background agent.
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

`dragonforge-core` must remain small and should not absorb cryptographic code simply because multiple products need it.

## 5. Password Manager placement after migration

The current Password Manager repository contains:

- `apps/desktop`
- `apps/sync-server`
- `apps/browser-extension`
- `crates/dragonforge-crypto`
- `crates/dragonforge-vault`

Tentative suite destinations:

```text
apps/desktop                  -> apps/password-manager/
apps/sync-server              -> services/password-manager-sync/
apps/browser-extension        -> extensions/password-manager-browser/
crates/dragonforge-crypto     -> crates/dragonforge-crypto/
crates/dragonforge-vault      -> crates/dragonforge-vault/
```

These paths are migration targets, not authorization to refactor them during the move.

## 6. Migration rule: move first, refactor second

The Password Manager migration must preserve behavior.

Sequence:

1. Freeze/select a tested Password Manager commit.
2. Record its commit SHA.
3. Import it with history preservation.
4. Adapt paths/workspace metadata only as necessary.
5. Run the original Password Manager automated test suite.
6. Resolve only migration-related failures.
7. Establish a passing suite baseline.
8. Only then extract or redesign shared components.

Crypto, vault format, sync protocol, key derivation, or serialization changes are explicitly out of scope for the migration itself.

## 7. Versioning

During early development:
- suite workspace: `0.x`;
- applications may release independently;
- persistent formats and IPC/network protocols require their own explicit format/protocol versions.

Repository version and encrypted-data format version must never be assumed to be the same thing.

## 8. Future ADRs

Significant decisions should be recorded under `docs/adr/`, including:
- cryptography ownership;
- IPC transport and authentication;
- Windows service model;
- event database format;
- update signing;
- plugin model;
- cross-platform strategy.
