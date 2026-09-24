# DragonForge Security Suite 1.x Compatibility Policy

DragonForge Security Suite 1.0 establishes the first stable compatibility baseline.

## Versioning

DragonForge uses semantic release versions for the suite. Within the 1.x line:

- patch releases may fix security, correctness, compatibility, accessibility, packaging, or documentation defects without intentionally breaking documented 1.0 formats/protocols;
- minor releases may add backward-compatible fields/capabilities and explicitly versioned migrations;
- incompatible format or protocol changes require a new major protocol/format version and a documented migration path. They must never silently reinterpret older data.

Unknown future major/schema/format versions fail closed where the current implementation requires exact versions.

## Stable 1.0 baselines

| Surface | 1.0 baseline | Compatibility commitment |
| --- | ---: | --- |
| File Vault `.dfvault` | format 1 | 1.x must continue to read valid v1 containers or provide an explicit migration before support is removed |
| Backup `.dfbackup` | format 1 | 1.x must continue to verify/restore valid v1 backups |
| Suite Recovery `.dfrecovery` | format 1, schema 2 | schema 1 migration to schema 2 remains supported; unknown future schemas fail closed |
| Secure Share `.dfshare` | format 1 | 1.x must continue to verify valid v1 packages subject to expiration policy |
| Shared local IPC | 1.0 | protocol-major 1 peers are compatible; major changes require a new compatibility contract |
| DragonForge Agent | 1.1 | protocol-major 1 remains the 1.x line; additive minor capability evolution must remain fail-closed for unknown actions |
| Privileged service | 1.1 | protocol-major 1 and the typed capability boundary remain stable; no generic command execution |
| Secure update manifest | schema 1 | signed schema-1 stable manifests remain supported through the 1.x line unless a security migration explicitly supersedes them |
| Password Manager sync API | 2 | protocol 2 is the stable 1.0 sync API |
| Password Manager sync sidecar | 3 | sidecars 1/2 migrate locally to 3; unknown future versions fail closed |
| Browser/native messaging | protocol 1 | unsupported versions are rejected |

## Cryptographic format changes

Cryptographic algorithm or KDF changes that affect stored data must be introduced by an explicit versioned envelope/format migration. DragonForge will not silently change the meaning of existing format version numbers.

## Update compatibility

Stable-channel clients accept only stable SemVer candidates, reject downgrade attempts, require ML-DSA-65 signed update metadata from the pinned key identity, verify artifact hashes/sizes, and require Authenticode on the Windows installer.

## Support boundary

Compatibility applies to valid data produced by supported DragonForge releases. It does not guarantee recovery of physically corrupted media, maliciously modified ciphertext, unsupported future schema versions, or files that exceed documented safety bounds.
