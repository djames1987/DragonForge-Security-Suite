$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot
$LogDir = Join-Path $RepoRoot "test-logs"
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
$Stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogPath = Join-Path $LogDir "dragonforge-phase15-password-manager-hardening-$Stamp.log"
$Failed = $false

Start-Transcript -Path $LogPath -Force | Out-Null
try {
    function Run([string]$Label, [scriptblock]$Command) {
        Write-Host ""
        Write-Host ">>> $Label"
        & $Command
        if ($LASTEXITCODE -ne 0) { throw "Command failed with exit code $LASTEXITCODE - $Label" }
    }

    Write-Host "DragonForge Security Suite - Phase 15 Password Manager Ecosystem Production Hardening verification"
    Run "cargo fmt --all --check" { cargo fmt --all --check }
    Run "cargo check --workspace --all-targets --all-features --locked" { cargo check --workspace --all-targets --all-features --locked }
    Run "cargo clippy --workspace --all-targets --all-features --locked -- -D warnings" { cargo clippy --workspace --all-targets --all-features --locked -- -D warnings }
    Run "cargo test --workspace --all-features --locked" { cargo test --workspace --all-features --locked }
    Run "browser extension tests" { node --test extensions/password-manager-browser/tests/*.test.mjs }

    foreach ($File in @(
        "docs/PHASE_15_PASSWORD_MANAGER_HARDENING.md",
        "docs/PASSWORD_MANAGER_SYNC_COMPATIBILITY.md",
        "scripts/password-manager/backup-sync-server.ps1",
        "scripts/password-manager/restore-sync-server.ps1"
    )) {
        if (-not (Test-Path -LiteralPath $File)) { throw "Missing Phase 15 artifact: $File" }
    }

    $Api = Get-Content -Raw "services/password-manager-sync/src/api.rs"
    foreach ($Needle in @("RateLimited","MAX_RATE_LIMIT_KEYS","no-store","nosniff","no-referrer")) {
        if (-not $Api.Contains($Needle)) { throw "Missing sync hardening invariant: $Needle" }
    }

    $Main = Get-Content -Raw "services/password-manager-sync/src/main.rs"
    foreach ($Needle in @("DRAGONFORGE_SYNC_ENVIRONMENT","DRAGONFORGE_SYNC_TLS_PROXY","DRAGONFORGE_SYNC_PUBLIC_BASE_URL")) {
        if (-not $Main.Contains($Needle)) { throw "Missing production runtime invariant: $Needle" }
    }

    $Package = Get-Content -Raw "scripts/password-manager/package-browser-extension.ps1"
    if (-not $Package.Contains("extensions\password-manager-browser")) { throw "Browser extension packaging path is stale." }

    foreach ($File in @(
        "scripts/password-manager/backup-sync-server.ps1",
        "scripts/password-manager/restore-sync-server.ps1",
        "scripts/password-manager/package-browser-extension.ps1",
        "scripts/password-manager/install-browser-native-host.ps1",
        "scripts/run-phase15-password-manager-hardening-tests.ps1"
    )) {
        $Tokens = $null
        $Errors = $null
        [System.Management.Automation.Language.Parser]::ParseFile((Resolve-Path $File), [ref]$Tokens, [ref]$Errors) | Out-Null
        if ($Errors.Count -gt 0) { throw "PowerShell syntax errors in $File" }
    }

    Write-Host ""
    Write-Host "PHASE 15 PASSWORD MANAGER ECOSYSTEM PRODUCTION HARDENING VERIFICATION: PASS"
}
catch {
    $Failed = $true
    Write-Host ""
    Write-Host "PHASE 15 PASSWORD MANAGER ECOSYSTEM PRODUCTION HARDENING VERIFICATION: FAIL"
    Write-Host $_
}
finally {
    Stop-Transcript | Out-Null
    $Hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $LogPath).Hash
    "$Hash  $([IO.Path]::GetFileName($LogPath))" | Set-Content -Encoding ascii "$LogPath.sha256"
    Write-Host "Log: $LogPath"
    Write-Host "SHA256: $Hash"
}
if ($Failed) { exit 1 }
