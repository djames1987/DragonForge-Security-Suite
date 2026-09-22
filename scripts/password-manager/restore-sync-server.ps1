param(
    [Parameter(Mandatory=$true)]
    [string]$BackupPath,
    [switch]$AcknowledgeDataReplacement
)
$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

if (-not $AcknowledgeDataReplacement) {
    throw "Restore replaces current sync database contents. Re-run with -AcknowledgeDataReplacement."
}

$ResolvedBackup = (Resolve-Path -LiteralPath $BackupPath).Path
$Sidecar = "$ResolvedBackup.sha256"
if (-not (Test-Path -LiteralPath $Sidecar)) { throw "Backup SHA-256 sidecar is required." }
$Expected = ((Get-Content -Raw -LiteralPath $Sidecar).Trim() -split '\s+')[0].ToUpperInvariant()
$Actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $ResolvedBackup).Hash.ToUpperInvariant()
if ($Expected -ne $Actual) { throw "Backup SHA-256 does not match its sidecar." }

$RepoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$ServiceRoot = Join-Path $RepoRoot "services\password-manager-sync"
Set-Location $ServiceRoot

$Container = (docker compose ps -q postgres).Trim()
if ([string]::IsNullOrWhiteSpace($Container)) { throw "PostgreSQL container is not running." }

docker compose stop sync
if ($LASTEXITCODE -ne 0) { throw "Unable to stop sync service before restore." }

$ContainerPath = "/tmp/dragonforge-sync-restore.dump"
try {
    docker cp $ResolvedBackup "$($Container):$ContainerPath"
    if ($LASTEXITCODE -ne 0) { throw "Unable to copy backup into PostgreSQL container." }
    docker exec $Container pg_restore -U dragonforge -d dragonforge_sync --clean --if-exists --no-owner $ContainerPath
    if ($LASTEXITCODE -ne 0) { throw "pg_restore failed." }
}
finally {
    docker exec $Container rm -f $ContainerPath 2>$null | Out-Null
    docker compose start sync | Out-Null
}

Write-Host "SYNC SERVER RESTORE: PASS"
Write-Host "Restored SHA256: $Actual"
