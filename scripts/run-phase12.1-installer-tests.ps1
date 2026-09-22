param(
    [switch]$SkipInstallerBuild
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$LogDirectory = Join-Path $RepoRoot "test-logs"
$Timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogPath = Join-Path $LogDirectory "dragonforge-phase12.1-installer-$Timestamp.log"
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
    Write-Host "DragonForge Security Suite - Phase 12.1 Windows Installer verification"
    Write-Host "Repository: $RepoRoot"
    Write-Host "Started: $(Get-Date -Format o)"

    Invoke-Checked cargo "fmt" "--all" "--check"
    Invoke-Checked cargo "check" "-p" "dragonforge-core" "-p" "dragonforge-agent" "-p" "dragonforge-agent-service" "-p" "dragonforge-security-center" "--all-targets"
    Invoke-Checked cargo "clippy" "-p" "dragonforge-core" "-p" "dragonforge-agent" "-p" "dragonforge-agent-service" "-p" "dragonforge-security-center" "--all-targets" "--" "-D" "warnings"
    Invoke-Checked cargo "test" "-p" "dragonforge-agent" "-p" "dragonforge-agent-service" "-p" "dragonforge-security-center" "--all-targets"
    Invoke-Checked node "--check" "apps/security-center/ui/app.js"

    foreach ($Path in @(
        "installer/DragonForgeSecuritySuite.iss",
        "scripts/package-windows-installer.ps1",
        "docs/PHASE_12_1_WINDOWS_INSTALLER.md",
        "docs/INSTALLER_TEST_CHECKLIST.md"
    )) {
        if (-not (Test-Path -LiteralPath (Join-Path $RepoRoot $Path) -PathType Leaf)) {
            throw "Required Phase 12.1 artifact is missing: $Path"
        }
        Write-Host "OK  $Path"
    }

    $Iss = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "installer\DragonForgeSecuritySuite.iss")
    foreach ($RequiredText in @(
        "PrivilegesRequired=lowest",
        "DefaultDirName={localappdata}\Programs\DragonForge Security Suite",
        "dragonforge-security-center.exe",
        "dragonforge-agent.exe",
        "WebView2Installed",
        "Stop-DragonForge-Agent.ps1"
    )) {
        if (-not $Iss.Contains($RequiredText)) {
            throw "Installer definition is missing required policy: $RequiredText"
        }
    }

    if (-not $SkipInstallerBuild) {
        & (Join-Path $PSScriptRoot "package-windows-installer.ps1") -Version "0.1.0-alpha.2"
        if ($LASTEXITCODE -ne 0) {
            throw "Phase 12.1 installer build failed with exit code $LASTEXITCODE."
        }

        $Installer = Join-Path $RepoRoot "dist\DragonForge-Security-Suite-v0.1.0-alpha.2-win-x64-setup.exe"
        $Sidecar = "$Installer.sha256"
        if (-not (Test-Path -LiteralPath $Installer -PathType Leaf)) {
            throw "Expected installer artifact was not produced."
        }
        if (-not (Test-Path -LiteralPath $Sidecar -PathType Leaf)) {
            throw "Expected installer SHA-256 sidecar was not produced."
        }
        $Expected = ((Get-Content -LiteralPath $Sidecar -Raw).Trim() -split "\s+")[0].ToUpperInvariant()
        $Actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $Installer).Hash.ToUpperInvariant()
        if ($Expected -ne $Actual) {
            throw "Installer SHA-256 sidecar does not match the installer."
        }
        Write-Host "INSTALLER SHA256 VERIFIED: $Actual"

        foreach ($TemporaryPath in @(
            (Join-Path $RepoRoot "dist\DragonForge-Security-Suite-v0.1.0-alpha.2-win-x64"),
            (Join-Path $RepoRoot "dist\DragonForge-Security-Suite-v0.1.0-alpha.2-win-x64-installer-stage")
        )) {
            if (Test-Path -LiteralPath $TemporaryPath) {
                throw "Temporary packaging directory was not cleaned: $TemporaryPath"
            }
            Write-Host "CLEAN  $TemporaryPath"
        }
    }

    Write-Host ""
    Write-Host "PHASE 12.1 WINDOWS INSTALLER VERIFICATION: PASS"
}
catch {
    Write-Host ""
    Write-Host "PHASE 12.1 WINDOWS INSTALLER VERIFICATION: FAIL"
    Write-Host $_
    throw
}
finally {
    try { Stop-Transcript | Out-Null } catch {}
    Pop-Location
    if (Test-Path -LiteralPath $LogPath) {
        $Hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $LogPath).Hash
        "$Hash  $(Split-Path -Leaf $LogPath)" | Set-Content -LiteralPath $HashPath -Encoding ASCII
        Write-Host "Log: $LogPath"
        Write-Host "SHA256: $Hash"
    }
}
