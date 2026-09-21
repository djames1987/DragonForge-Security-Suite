param(
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$LogDirectory = Join-Path $RepoRoot "test-logs"
$Timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogPath = Join-Path $LogDirectory "dragonforge-phase7-integrity-monitor-$Timestamp.log"
$HashPath = "$LogPath.sha256"

New-Item -ItemType Directory -Force -Path $LogDirectory | Out-Null
Push-Location $RepoRoot

function Invoke-Checked {
    param(
        [Parameter(Mandatory = $true)] [string]$Command,
        [Parameter(ValueFromRemainingArguments = $true)] [string[]]$Arguments
    )
    Write-Host ""
    Write-Host ">>> $Command $($Arguments -join ' ')"
    & $Command @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "Command failed with exit code ${LASTEXITCODE}: $Command $($Arguments -join ' ')"
    }
}

try {
    Start-Transcript -Path $LogPath -Force | Out-Null
    Write-Host "DragonForge Security Suite - Phase 7 Integrity Monitor verification"
    Write-Host "Repository: $RepoRoot"
    Write-Host "Started: $(Get-Date -Format o)"

    Invoke-Checked cargo "fmt" "-p" "dragonforge-core" "--check"
    Invoke-Checked cargo "fmt" "-p" "dragonforge-security-center" "--check"
    Invoke-Checked cargo "fmt" "-p" "dragonforge-file-vault" "-p" "dragonforge-file-vault-app" "--check"
    Invoke-Checked cargo "fmt" "-p" "dragonforge-authenticator" "-p" "dragonforge-authenticator-app" "--check"
    Invoke-Checked cargo "fmt" "-p" "dragonforge-security-scanner" "-p" "dragonforge-security-scanner-app" "--check"
    Write-Host ""
    Write-Host ">>> rustfmt --edition 2024 --check crates/dragonforge-integrity-monitor/src/lib.rs"
    $PreviousErrorActionPreference = $ErrorActionPreference
    try {
        $ErrorActionPreference = "Continue"
        & rustfmt --edition 2024 --check crates/dragonforge-integrity-monitor/src/lib.rs 2>&1 | ForEach-Object { Write-Host $_ }
        $IntegrityEngineRustfmtExit = $LASTEXITCODE
    }
    finally {
        $ErrorActionPreference = $PreviousErrorActionPreference
    }
    if ($IntegrityEngineRustfmtExit -ne 0) {
        throw "Command failed with exit code ${IntegrityEngineRustfmtExit}: rustfmt --edition 2024 --check crates/dragonforge-integrity-monitor/src/lib.rs"
    }
    Invoke-Checked rustfmt "--edition" "2024" "--check" "apps/integrity-monitor/src/lib.rs"
    Invoke-Checked rustfmt "--edition" "2024" "--check" "apps/integrity-monitor/src/main.rs"
    Invoke-Checked cargo "fmt" "-p" "dragonforge-integrity-monitor" "-p" "dragonforge-integrity-monitor-app" "--check"

    Invoke-Checked cargo "check" "-p" "dragonforge-core" "-p" "dragonforge-security-center" "-p" "dragonforge-file-vault" "-p" "dragonforge-file-vault-app" "-p" "dragonforge-authenticator" "-p" "dragonforge-authenticator-app" "-p" "dragonforge-security-scanner" "-p" "dragonforge-security-scanner-app" "-p" "dragonforge-integrity-monitor" "-p" "dragonforge-integrity-monitor-app" "--all-targets"

    Invoke-Checked cargo "clippy" "-p" "dragonforge-core" "--all-targets" "--" "-D" "warnings"
    Invoke-Checked cargo "clippy" "-p" "dragonforge-security-center" "--all-targets" "--" "-D" "warnings"
    Invoke-Checked cargo "clippy" "-p" "dragonforge-file-vault" "-p" "dragonforge-file-vault-app" "--all-targets" "--" "-D" "warnings"
    Invoke-Checked cargo "clippy" "-p" "dragonforge-authenticator" "-p" "dragonforge-authenticator-app" "--all-targets" "--" "-D" "warnings"
    Invoke-Checked cargo "clippy" "-p" "dragonforge-security-scanner" "-p" "dragonforge-security-scanner-app" "--all-targets" "--" "-D" "warnings"

    Write-Host ""
    Write-Host ">>> cargo clippy -p dragonforge-integrity-monitor -p dragonforge-integrity-monitor-app --all-targets -- -D warnings"
    $PreviousErrorActionPreference = $ErrorActionPreference
    try {
        $ErrorActionPreference = "Continue"
        & cargo clippy -p dragonforge-integrity-monitor -p dragonforge-integrity-monitor-app --all-targets -- -D warnings 2>&1 | ForEach-Object { Write-Host $_ }
        $IntegrityClippyExit = $LASTEXITCODE
    }
    finally { $ErrorActionPreference = $PreviousErrorActionPreference }
    if ($IntegrityClippyExit -ne 0) {
        throw "Command failed with exit code ${IntegrityClippyExit}: cargo clippy Integrity Monitor"
    }

    Invoke-Checked cargo "test" "-p" "dragonforge-integrity-monitor" "-p" "dragonforge-integrity-monitor-app" "--all-targets"
    Invoke-Checked cargo "test" "-p" "dragonforge-security-scanner" "-p" "dragonforge-security-scanner-app" "--all-targets"
    Invoke-Checked cargo "test" "-p" "dragonforge-authenticator" "-p" "dragonforge-authenticator-app" "--all-targets"
    Invoke-Checked cargo "test" "-p" "dragonforge-file-vault" "-p" "dragonforge-file-vault-app" "--all-targets"
    Invoke-Checked cargo "test" "-p" "dragonforge-core" "-p" "dragonforge-security-center" "--all-targets"

    Invoke-Checked node "--check" "apps/integrity-monitor/ui/app.js"
    Invoke-Checked node "--check" "apps/security-scanner/ui/app.js"
    Invoke-Checked node "--check" "apps/authenticator/ui/app.js"
    Invoke-Checked node "--check" "apps/file-vault/ui/app.js"
    Invoke-Checked node "--check" "apps/security-center/ui/app.js"

    if (-not $SkipBuild) {
        Invoke-Checked cargo "build" "-p" "dragonforge-desktop" "--bin" "dragonforge-desktop"
        Invoke-Checked cargo "build" "-p" "dragonforge-file-vault-app"
        Invoke-Checked cargo "build" "-p" "dragonforge-authenticator-app"
        Invoke-Checked cargo "build" "-p" "dragonforge-security-scanner-app"
        Invoke-Checked cargo "build" "-p" "dragonforge-integrity-monitor-app"
        Invoke-Checked cargo "build" "-p" "dragonforge-security-center"
    }

    Write-Host ""
    Write-Host "PHASE 7 INTEGRITY MONITOR VERIFICATION: PASS"
}
catch {
    Write-Host ""
    Write-Host "PHASE 7 INTEGRITY MONITOR VERIFICATION: FAIL"
    Write-Host $_
    throw
}
finally {
    try { Stop-Transcript | Out-Null } catch {}
    Pop-Location
    if (Test-Path $LogPath) {
        $Hash = (Get-FileHash -Algorithm SHA256 $LogPath).Hash
        "$Hash  $(Split-Path -Leaf $LogPath)" | Set-Content -Encoding ascii $HashPath
        Write-Host "Log: $LogPath"
        Write-Host "SHA256: $Hash"
    }
}
