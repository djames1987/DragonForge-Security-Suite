param(
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$LogDirectory = Join-Path $RepoRoot "test-logs"
$Timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogPath = Join-Path $LogDirectory "dragonforge-phase3-security-center-$Timestamp.log"
$HashPath = "$LogPath.sha256"

New-Item -ItemType Directory -Force -Path $LogDirectory | Out-Null
Push-Location $RepoRoot

function Invoke-Checked {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Command,
        [Parameter(ValueFromRemainingArguments = $true)]
        [string[]]$Arguments
    )

    Write-Host ""
    Write-Host ">>> $Command $($Arguments -join ' ')"
    & $Command @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "Command failed with exit code $LASTEXITCODE: $Command $($Arguments -join ' ')"
    }
}

try {
    Start-Transcript -Path $LogPath -Force | Out-Null
    Write-Host "DragonForge Security Suite - Phase 3 Security Center verification"
    Write-Host "Repository: $RepoRoot"
    Write-Host "Started: $(Get-Date -Format o)"

    Invoke-Checked cargo "fmt" "--all" "--check"
    Invoke-Checked cargo "check" "-p" "dragonforge-core" "-p" "dragonforge-security-center" "--all-targets"
    Invoke-Checked cargo "clippy" "-p" "dragonforge-core" "-p" "dragonforge-security-center" "--all-targets" "--" "-D" "warnings"
    Invoke-Checked cargo "test" "-p" "dragonforge-core" "-p" "dragonforge-security-center" "--all-targets"
    Invoke-Checked node "--check" "apps/security-center/ui/app.js"

    if (-not $SkipBuild) {
        Invoke-Checked cargo "build" "-p" "dragonforge-security-center"
    }

    Write-Host ""
    Write-Host "PHASE 3 SECURITY CENTER VERIFICATION: PASS"
}
catch {
    Write-Host ""
    Write-Host "PHASE 3 SECURITY CENTER VERIFICATION: FAIL"
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
