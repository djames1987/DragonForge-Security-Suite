# DragonForge Security Suite 1.0 Migration Guide

## From current pre-release builds

The 1.0 installer uses the established DragonForge installer AppId and is intended to upgrade an existing installation in place while preserving DragonForge user data.

Before upgrading:

1. create an encrypted DragonForge suite recovery package or product-specific backup;
2. preserve the backup/recovery file and its password somewhere separate from the installation directory;
3. if Password Manager sync is deployed, back up the PostgreSQL database and retain its SHA-256 sidecar;
4. close or allow the installer to close DragonForge desktop applications;
5. leave externally stored vaults, backups, shares, and recovery packages in place.

The normal installer path remains per-user under `%LOCALAPPDATA%\Programs\DragonForge Security Suite`. Installing the optional privileged service is a separate administrator-approved task.

## Password Manager sync deployments

Password Manager sync API protocol 2 is the 1.0 baseline. Sync sidecars v1/v2 migrate locally to v3. PostgreSQL migrations remain forward-only through the tracked migration set.

Before server upgrades, follow `docs/PASSWORD_MANAGER_SYNC_COMPATIBILITY.md`: back up the database, deploy the new server, allow tracked SQLx migrations to complete, verify health, then run client sync smoke tests.

## Recovery and rollback

Do not assume database migrations or encrypted-format migrations are automatically reversible.

If an upgrade must be rolled back:

- use a backup compatible with the older binary/schema;
- do not replace a newer persistent state file with an older binary unless its compatibility is documented;
- preserve the failed/newer state for diagnosis rather than repeatedly overwriting it;
- never bypass format/schema version checks.

## 1.x guarantees

DragonForge 1.x preserves the documented 1.0 compatibility baselines in `docs/1_0_COMPATIBILITY_POLICY.md`. Any future incompatible change requires an explicit new version and migration path.
