param(
    [string]$OutputDirectory = "backups"
)
$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
$RepoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$ServiceRoot = Join-Path $RepoRoot "services\password-manager-sync"
$OutputRoot = Join-Path $RepoRoot $OutputDirectory
New-Item -ItemType Directory -Force -Path $OutputRoot | Out-Null
Set-Location $ServiceRoot
$Container = (docker compose ps -q postgres).Trim()
if ([string]::IsNullOrWhiteSpace($Container)) { throw "PostgreSQL container is not running." }
$Stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$ContainerPath = "/tmp/dragonforge-sync-$Stamp.dump"
$LocalPath = Join-Path $OutputRoot "dragonforge-sync-$Stamp.dump"
docker exec $Container pg_dump -U dragonforge -d dragonforge_sync -Fc -f $ContainerPath
if ($LASTEXITCODE -ne 0) { throw "pg_dump failed." }
docker cp "$($Container):$ContainerPath" $LocalPath
if ($LASTEXITCODE -ne 0) { throw "docker cp failed." }
docker exec $Container rm -f $ContainerPath | Out-Null
if (-not (Test-Path -LiteralPath $LocalPath)) { throw "Backup file was not created." }
$Hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $LocalPath).Hash
"$Hash  $([IO.Path]::GetFileName($LocalPath))" | Set-Content -Encoding ascii "$LocalPath.sha256"
Write-Host "SYNC SERVER BACKUP: PASS"
Write-Host "Backup: $LocalPath"
Write-Host "SHA256: $Hash"
