param(
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$LogDirectory = Join-Path $RepoRoot "test-logs"
$Timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogPath = Join-Path $LogDirectory "dragonforge-phase8-network-guard-$Timestamp.log"
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
    $PreviousErrorActionPreference = $ErrorActionPreference
    try {
        $ErrorActionPreference = "Continue"
        & $Command @Arguments 2>&1 | ForEach-Object { Write-Host $_ }
        $CommandExitCode = $LASTEXITCODE
    }
    finally {
        $ErrorActionPreference = $PreviousErrorActionPreference
    }
    if ($CommandExitCode -ne 0) {
        throw "Command failed with exit code ${CommandExitCode}: $Command $($Arguments -join ' ')"
    }
}

try {
    Start-Transcript -Path $LogPath -Force | Out-Null
    Write-Host "DragonForge Security Suite - Phase 8 Network Guard verification"
    Write-Host "Repository: $RepoRoot"
    Write-Host "Started: $(Get-Date -Format o)"

    Invoke-Checked cargo "fmt" "-p" "dragonforge-core" "--check"
    Invoke-Checked cargo "fmt" "-p" "dragonforge-security-center" "--check"
    Invoke-Checked cargo "fmt" "-p" "dragonforge-file-vault" "-p" "dragonforge-file-vault-app" "--check"
    Invoke-Checked cargo "fmt" "-p" "dragonforge-authenticator" "-p" "dragonforge-authenticator-app" "--check"
    Invoke-Checked cargo "fmt" "-p" "dragonforge-security-scanner" "-p" "dragonforge-security-scanner-app" "--check"
    Invoke-Checked cargo "fmt" "-p" "dragonforge-integrity-monitor" "-p" "dragonforge-integrity-monitor-app" "--check"
    Invoke-Checked cargo "fmt" "-p" "dragonforge-network-guard" "-p" "dragonforge-network-guard-app" "--check"

    Invoke-Checked cargo "check" "-p" "dragonforge-core" "-p" "dragonforge-security-center" "-p" "dragonforge-file-vault" "-p" "dragonforge-file-vault-app" "-p" "dragonforge-authenticator" "-p" "dragonforge-authenticator-app" "-p" "dragonforge-security-scanner" "-p" "dragonforge-security-scanner-app" "-p" "dragonforge-integrity-monitor" "-p" "dragonforge-integrity-monitor-app" "-p" "dragonforge-network-guard" "-p" "dragonforge-network-guard-app" "--all-targets"

    Invoke-Checked cargo "clippy" "-p" "dragonforge-core" "--all-targets" "--" "-D" "warnings"
    Invoke-Checked cargo "clippy" "-p" "dragonforge-security-center" "--all-targets" "--" "-D" "warnings"
    Invoke-Checked cargo "clippy" "-p" "dragonforge-file-vault" "-p" "dragonforge-file-vault-app" "--all-targets" "--" "-D" "warnings"
    Invoke-Checked cargo "clippy" "-p" "dragonforge-authenticator" "-p" "dragonforge-authenticator-app" "--all-targets" "--" "-D" "warnings"
    Invoke-Checked cargo "clippy" "-p" "dragonforge-security-scanner" "-p" "dragonforge-security-scanner-app" "--all-targets" "--" "-D" "warnings"
    Invoke-Checked cargo "clippy" "-p" "dragonforge-integrity-monitor" "-p" "dragonforge-integrity-monitor-app" "--all-targets" "--" "-D" "warnings"

    Write-Host ""
    Write-Host ">>> cargo clippy -p dragonforge-network-guard -p dragonforge-network-guard-app --all-targets -- -D warnings"
    $PreviousErrorActionPreference = $ErrorActionPreference
    try {
        $ErrorActionPreference = "Continue"
        & cargo clippy -p dragonforge-network-guard -p dragonforge-network-guard-app --all-targets -- -D warnings 2>&1 | ForEach-Object { Write-Host $_ }
        $NetworkClippyExit = $LASTEXITCODE
    }
    finally { $ErrorActionPreference = $PreviousErrorActionPreference }
    if ($NetworkClippyExit -ne 0) {
        throw "Command failed with exit code ${NetworkClippyExit}: cargo clippy Network Guard"
    }

    Invoke-Checked cargo "test" "-p" "dragonforge-network-guard" "-p" "dragonforge-network-guard-app" "--all-targets"
    Invoke-Checked cargo "test" "-p" "dragonforge-integrity-monitor" "-p" "dragonforge-integrity-monitor-app" "--all-targets"
    Invoke-Checked cargo "test" "-p" "dragonforge-security-scanner" "-p" "dragonforge-security-scanner-app" "--all-targets"
    Invoke-Checked cargo "test" "-p" "dragonforge-authenticator" "-p" "dragonforge-authenticator-app" "--all-targets"
    Invoke-Checked cargo "test" "-p" "dragonforge-file-vault" "-p" "dragonforge-file-vault-app" "--all-targets"
    Invoke-Checked cargo "test" "-p" "dragonforge-core" "-p" "dragonforge-security-center" "--all-targets"

    Invoke-Checked node "--check" "apps/network-guard/ui/app.js"
    Invoke-Checked node "--check" "apps/integrity-monitor/ui/app.js"
    Invoke-Checked node "--check" "apps/security-scanner/ui/app.js"
    Invoke-Checked node "--check" "apps/authenticator/ui/app.js"
    Invoke-Checked node "--check" "apps/file-vault/ui/app.js"
    Invoke-Checked node "--check" "apps/security-center/ui/app.js"

    if (-not $SkipBuild) {
        & (Join-Path $PSScriptRoot "build-all-apps-for-testing.ps1")
        if ($LASTEXITCODE -ne 0) {
            throw "Build-all-apps verification failed with exit code ${LASTEXITCODE}"
        }
    }

    Write-Host ""
    Write-Host "PHASE 8 NETWORK GUARD VERIFICATION: PASS"
}
catch {
    Write-Host ""
    Write-Host "PHASE 8 NETWORK GUARD VERIFICATION: FAIL"
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
