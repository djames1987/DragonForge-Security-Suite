# Password Manager Sync Compatibility Matrix

| Surface | Current | Supported | Failure behavior |
|---|---:|---:|---|
| Sync API protocol | 2 | 2 only | incompatible versions must fail closed |
| Sync sidecar | 3 | 1, 2, 3 | v1/v2 migrate locally to v3 |
| PostgreSQL schema | migrations 0001-0003 | forward migration from tracked migrations | startup fails if migration cannot complete |
| Browser extension | Manifest V3 | packaged repository extension only | packaging fails on broad permissions |
| Native messaging | protocol 1 | protocol 1 | unsupported request version rejected |

Before server upgrades: create a database backup, preserve its SHA-256 sidecar, deploy the new server, allow SQLx migrations to complete, verify `/v1/health`, then run client sync smoke tests. Roll back application binaries only with a database backup compatible with that binary/schema set; do not assume SQL migrations are reversible.
