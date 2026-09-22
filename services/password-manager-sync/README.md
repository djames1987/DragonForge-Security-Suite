# DragonForge Password Manager Sync Server

DragonForge Password Manager includes a zero-knowledge sync service. The server stores encrypted vault blobs and sync metadata; it does not need the user's vault password or plaintext vault contents.

## What this service does

- account provisioning;
- per-account sync tokens;
- encrypted vault upload/download with revision checking;
- device enrollment, approval, and revocation;
- secure-recovery metadata;
- PostgreSQL persistence when built with the postgres feature.

The PostgreSQL migrations in migrations/ are applied automatically when the server connects to the database.

## Recommended local Docker setup

The included docker-compose.yml starts PostgreSQL 16 and the DragonForge sync server with PostgreSQL support. The sync endpoint is published only to 127.0.0.1:8787 by default.

### 1. Create the environment file

From services/password-manager-sync:

~~~powershell
Copy-Item .env.example .env
~~~

Generate a strong database password and admin token. This PowerShell helper returns a 64-character hexadecimal 256-bit secret:

~~~powershell
function New-HexSecret([int]$Bytes = 32) {
    $buffer = New-Object byte[] $Bytes
    [System.Security.Cryptography.RandomNumberGenerator]::Create().GetBytes($buffer)
    return (($buffer | ForEach-Object { $_.ToString("x2") }) -join "")
}

New-HexSecret 32
~~~

Put different generated values into .env:

~~~text
POSTGRES_PASSWORD=...
DRAGONFORGE_SYNC_ADMIN_TOKEN=...
~~~

The admin token must be at least 32 characters. Do not commit the completed .env file.

### 2. Start PostgreSQL and the sync server

~~~powershell
docker compose up -d --build
~~~

Check status:

~~~powershell
docker compose ps
~~~

Follow the sync-server logs:

~~~powershell
docker compose logs -f sync
~~~

### 3. Verify the health endpoint

~~~powershell
Invoke-RestMethod http://127.0.0.1:8787/v1/health
~~~

A healthy server returns an object containing ok = true and the current sync protocol version.

## Provision a Password Manager sync account

Account creation is intentionally protected by the server admin token. The server generates the per-account 64-character hexadecimal sync token.

Set the local admin token:

~~~powershell
$AdminToken = "your-DRAGONFORGE_SYNC_ADMIN_TOKEN-value"
~~~

Create the account:

~~~powershell
$Account = Invoke-RestMethod -Method Post `
    -Uri http://127.0.0.1:8787/v1/accounts `
    -Headers @{ "x-dragonforge-admin-token" = $AdminToken }

$Account
~~~

The response contains accountId and syncToken. The syncToken is the value entered into Password Manager. Treat it like a credential. The server stores only a hash of it.

## Configure Password Manager

Open the vault, choose Security settings, then use the Multi-device sync section.

For a sync server running on the same computer:

~~~text
Sync server URL: http://127.0.0.1:8787
Sync token: <syncToken returned by POST /v1/accounts>
~~~

Click Save sync settings, then Enroll / refresh device. The first device for an account becomes active automatically. Additional devices enroll as pending and must be approved from an already active device using the device list in Security settings.

The Sync vault sidebar action and Sync now button then synchronize the encrypted vault.

On Windows, the sync token and device signing secret are stored through the Password Manager's secure local credential-storage path rather than remaining as plaintext secrets in the sync sidecar.

## Important network rule

Password Manager accepts plaintext HTTP only for loopback development addresses:

- 127.0.0.1
- localhost
- ::1

For a sync server on another computer, LAN, VPS, or Internet host, Password Manager requires an https:// URL.

Do not merely expose port 8787 as plaintext HTTP on a network. Put the sync service behind a TLS reverse proxy such as Caddy or Nginx, use a certificate trusted by the client computer, expose only HTTPS, and configure Password Manager with that HTTPS URL.

The default Compose file intentionally binds the service to 127.0.0.1 so it cannot accidentally expose the sync API over plaintext LAN traffic.

## Database lifecycle

The server reads DRAGONFORGE_SYNC_DATABASE_URL. The included Compose deployment points this to the postgres container.

PostgresStore automatically runs the embedded SQLx migrations when it connects, so no manual SQL migration step is required.

Persistent data lives in the Docker named volume dragonforge_sync_postgres.

Stop without deleting database data:

~~~powershell
docker compose down
~~~

Start again:

~~~powershell
docker compose up -d
~~~

Do not use docker compose down -v unless you intentionally want to delete the PostgreSQL volume. Removing that volume deletes server-side accounts, encrypted vault blobs, device records, and recovery metadata.

## Back up the sync database

Example PostgreSQL dump:

~~~powershell
docker compose exec -T postgres pg_dump -U dragonforge -d dragonforge_sync > dragonforge-sync-backup.sql
~~~

Database backups contain encrypted vault ciphertext plus account, device, and recovery metadata. Protect the backup appropriately.

## Run without Docker

A developer build with PostgreSQL support can run directly:

~~~powershell
$env:DRAGONFORGE_SYNC_BIND = "127.0.0.1:8787"
$env:DRAGONFORGE_SYNC_DATABASE_URL = "postgres://dragonforge:password@127.0.0.1:5432/dragonforge_sync"
$env:DRAGONFORGE_SYNC_ADMIN_TOKEN = "replace-with-at-least-32-characters"

cargo run -p dragonforge-sync-server --features postgres
~~~

If the server is built without the postgres feature, it uses volatile in-memory storage and loses data when the process exits. If DRAGONFORGE_SYNC_DATABASE_URL is set while the server was built without PostgreSQL support, startup fails rather than silently ignoring the database configuration.

## Environment variables

- DRAGONFORGE_SYNC_BIND — listener address. Default: 127.0.0.1:8787.
- DRAGONFORGE_SYNC_DATABASE_URL — PostgreSQL connection URL. Requires the postgres feature.
- DRAGONFORGE_SYNC_ADMIN_TOKEN — protects account provisioning. Must be at least 32 characters. If omitted, account provisioning is disabled.

## Useful Docker commands

~~~powershell
# Start
docker compose up -d

# Rebuild after source changes
docker compose up -d --build

# Status
docker compose ps

# Sync-server logs
docker compose logs -f sync

# PostgreSQL logs
docker compose logs -f postgres

# Stop without deleting DB
docker compose down

# Restart sync service
docker compose restart sync
~~~

## Security notes

- The server stores encrypted vault data rather than plaintext vault contents.
- Sync authentication uses a generated per-account token; the server stores its SHA-256-derived token hash.
- Active devices sign protected sync requests with ML-DSA-65 device identities.
- New devices after the first require approval.
- Revision checks prevent blind overwrite during normal synchronization.
- The client validates downloaded encrypted vault structure and vault identity before local replacement.
- Plain HTTP is intentionally limited to loopback by the client.


## Phase 15 production hardening

For production deployment set `DRAGONFORGE_SYNC_ENVIRONMENT=production`. Production startup fails closed unless the server has PostgreSQL support, `DRAGONFORGE_SYNC_DATABASE_URL` is configured, the admin token is a 64-character hexadecimal secret, `DRAGONFORGE_SYNC_TLS_PROXY=true`, and `DRAGONFORGE_SYNC_PUBLIC_BASE_URL` is a bounded `https://` URL.

The raw Rust listener is HTTP. Keep it private to the host/container network and terminate public TLS at a trusted reverse proxy. The default Compose publication remains loopback-only.

Phase 15 also applies a bounded request-rate window in addition to the existing request body limit and timeout. Defaults are 240 requests per 60 seconds per hashed credential/device bucket and can be adjusted with `DRAGONFORGE_SYNC_RATE_LIMIT_REQUESTS` and `DRAGONFORGE_SYNC_RATE_LIMIT_WINDOW_SECONDS`.

Before deployment upgrades, create a verified database backup:

~~~powershell
.\..\..\scripts\password-manager\backup-sync-server.ps1
~~~

Restores require a matching SHA-256 sidecar and explicit acknowledgement:

~~~powershell
.\..\..\scripts\password-manager\restore-sync-server.ps1 -BackupPath <dump> -AcknowledgeDataReplacement
~~~

See `docs/PASSWORD_MANAGER_SYNC_COMPATIBILITY.md` and `docs/PHASE_15_PASSWORD_MANAGER_HARDENING.md` for the compatibility and migration policy.
