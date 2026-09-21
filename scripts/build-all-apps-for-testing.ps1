param(
    [ValidateSet("debug", "release")]
    [string]$Profile = "debug"
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
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
    $ReleaseArgs = @()
    $TargetProfile = "debug"
    if ($Profile -eq "release") {
        $ReleaseArgs = @("--release")
        $TargetProfile = "release"
    }

    Write-Host "DragonForge Security Suite - Build all desktop apps for testing"
    Write-Host "Repository: $RepoRoot"
    Write-Host "Profile: $Profile"

    Invoke-Checked cargo "build" "-p" "dragonforge-desktop" "--bin" "dragonforge-desktop" @ReleaseArgs
    Invoke-Checked cargo "build" "-p" "dragonforge-security-center" @ReleaseArgs
    Invoke-Checked cargo "build" "-p" "dragonforge-file-vault-app" @ReleaseArgs
    Invoke-Checked cargo "build" "-p" "dragonforge-authenticator-app" @ReleaseArgs
    Invoke-Checked cargo "build" "-p" "dragonforge-security-scanner-app" @ReleaseArgs
    Invoke-Checked cargo "build" "-p" "dragonforge-integrity-monitor-app" @ReleaseArgs
    Invoke-Checked cargo "build" "-p" "dragonforge-network-guard-app" @ReleaseArgs
    Invoke-Checked cargo "build" "-p" "dragonforge-backup-recovery-app" @ReleaseArgs
    Invoke-Checked cargo "build" "-p" "dragonforge-secure-share-app" @ReleaseArgs

    $Extension = if ($env:OS -eq "Windows_NT") { ".exe" } else { "" }
    $Expected = @(
        "dragonforge-desktop$Extension",
        "dragonforge-security-center$Extension",
        "dragonforge-file-vault$Extension",
        "dragonforge-authenticator$Extension",
        "dragonforge-security-scanner$Extension",
        "dragonforge-integrity-monitor$Extension",
        "dragonforge-network-guard$Extension",
        "dragonforge-backup-recovery$Extension",
        "dragonforge-secure-share$Extension"
    )

    $TargetDirectory = Join-Path $RepoRoot "target\$TargetProfile"
    Write-Host ""
    Write-Host "Verifying expected test executables in $TargetDirectory"

    foreach ($Name in $Expected) {
        $Path = Join-Path $TargetDirectory $Name
        if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
            throw "Expected application executable was not produced: $Path"
        }
        Write-Host "OK  $Path"
    }

    Write-Host ""
    Write-Host "ALL DRAGONFORGE TEST APPS BUILT: PASS"
}
finally {
    Pop-Location
}
