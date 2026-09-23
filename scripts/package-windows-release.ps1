param(
    [string]$Version = "0.1.0-alpha.1",
    [switch]$SkipBuild,
    [switch]$KeepStage,
    [string]$ExpectedCommit,
    [string]$ReleaseTag,
    [switch]$SignRelease
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$DistRoot = Join-Path $RepoRoot "dist"
$PackageName = "DragonForge-Security-Suite-v$Version-win-x64"
$StageRoot = Join-Path $DistRoot $PackageName
$ZipPath = Join-Path $DistRoot "$PackageName.zip"
$ZipHashPath = "$ZipPath.sha256"
$TargetRelease = Join-Path $RepoRoot "target\release"

$Executables = @(
    "dragonforge-desktop.exe",
    "dragonforge-security-center.exe",
    "dragonforge-file-vault.exe",
    "dragonforge-authenticator.exe",
    "dragonforge-security-scanner.exe",
    "dragonforge-integrity-monitor.exe",
    "dragonforge-network-guard.exe",
    "dragonforge-backup-recovery.exe",
    "dragonforge-secure-share.exe",
    "dragonforge-agent.exe",
    "dragonforge-privileged-service.exe"
)

function Invoke-Checked {
    param(
        [Parameter(Mandatory = $true)] [string]$Command,
        [Parameter(ValueFromRemainingArguments = $true)] [string[]]$Arguments
    )
    Write-Host ""
    Write-Host ">>> $Command $($Arguments -join ' ')"
    $Previous = $ErrorActionPreference
    try {
        $ErrorActionPreference = "Continue"
        & $Command @Arguments 2>&1 | ForEach-Object { Write-Host $_ }
        $ExitCode = $LASTEXITCODE
    }
    finally {
        $ErrorActionPreference = $Previous
    }
    if ($ExitCode -ne 0) {
        throw "Command failed with exit code $ExitCode - $Command $($Arguments -join ' ')"
    }
}

if ($env:OS -ne "Windows_NT") {
    throw "The Windows portable release must be packaged on Windows."
}

Push-Location $RepoRoot
$PreviousBuildCommit = $env:DRAGONFORGE_BUILD_COMMIT
try {
    New-Item -ItemType Directory -Force -Path $DistRoot | Out-Null

    $Commit = (& git rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0 -or -not $Commit) {
        throw "Unable to determine the release commit."
    }
    if ($ExpectedCommit -and $Commit -ne $ExpectedCommit) {
        throw "Release commit mismatch. Expected $ExpectedCommit but HEAD is $Commit."
    }
    if ($ReleaseTag) {
        $TaggedCommit = (& git rev-parse "$ReleaseTag^{commit}").Trim()
        if ($LASTEXITCODE -ne 0 -or $TaggedCommit -ne $Commit) {
            throw "Release tag $ReleaseTag does not resolve to the current commit."
        }
    }
    $env:DRAGONFORGE_BUILD_COMMIT = $Commit
    $ReleaseChannel = if ($Version -match '-') { "pre-release / external testing" } else { "stable" }
    $SigningLabel = if ($SignRelease) { "Authenticode SHA-256 + RFC 3161 timestamp" } else { "unsigned development/test build" }
    $SigningNotice = if ($SignRelease) {
        "This package is Authenticode-signed. Verify signatures before trusting the binaries."
    } else {
        "This package is unsigned. Windows SmartScreen may show an unknown-publisher warning."
    }

    if (-not $SkipBuild) {
        & (Join-Path $PSScriptRoot "build-all-apps-for-testing.ps1") -Profile release
        if ($LASTEXITCODE -ne 0) {
            throw "Release-profile suite build failed with exit code $LASTEXITCODE."
        }
    }

    foreach ($Name in $Executables) {
        $Source = Join-Path $TargetRelease $Name
        if (-not (Test-Path -LiteralPath $Source -PathType Leaf)) {
            throw "Missing release executable: $Source"
        }
    }

    if (Test-Path -LiteralPath $StageRoot) {
        Remove-Item -LiteralPath $StageRoot -Recurse -Force
    }
    if (Test-Path -LiteralPath $ZipPath) {
        Remove-Item -LiteralPath $ZipPath -Force
    }
    if (Test-Path -LiteralPath $ZipHashPath) {
        Remove-Item -LiteralPath $ZipHashPath -Force
    }

    New-Item -ItemType Directory -Force -Path $StageRoot | Out-Null

    foreach ($Name in $Executables) {
        Copy-Item -LiteralPath (Join-Path $TargetRelease $Name) -Destination (Join-Path $StageRoot $Name)
    }

    Copy-Item -LiteralPath (Join-Path $RepoRoot "SECURITY.md") -Destination (Join-Path $StageRoot "SECURITY.md")
    Copy-Item -LiteralPath (Join-Path $RepoRoot "docs\EXTERNAL_TEST_CHECKLIST.md") -Destination (Join-Path $StageRoot "EXTERNAL-TEST-CHECKLIST.md")
    foreach ($ScriptName in @(
        "install-privileged-service.ps1",
        "install-privileged-service.cmd",
        "uninstall-privileged-service.ps1",
        "uninstall-privileged-service.cmd",
        "test-privileged-service.ps1"
    )) {
        Copy-Item -LiteralPath (Join-Path $PSScriptRoot $ScriptName) -Destination (Join-Path $StageRoot $ScriptName)
    }

    $BuildInfo = @"
DragonForge Security Suite
Version: v$Version
Release channel: $ReleaseChannel
Git commit: $Commit
Git tag: $(if ($ReleaseTag) { $ReleaseTag } else { "un-tagged" })
Built (UTC): $([DateTime]::UtcNow.ToString("o"))
Platform: Windows x64 portable
Code signing: $SigningLabel
"@
    Set-Content -LiteralPath (Join-Path $StageRoot "BUILD-INFO.txt") -Value $BuildInfo -Encoding UTF8

    $StartHere = @"
DragonForge Security Suite v$Version - Windows x64 Portable

Release channel: $ReleaseChannel
Do not treat it as production security software yet.

No Rust, Cargo, Node.js, Git, or source checkout is required on the test machine.

FIRST RUN
1. Extract this entire ZIP to a normal writable folder.
2. Keep every EXE in the same folder. Security Center launches suite apps and the Agent by exact sibling path.
3. Run Verify-Package.cmd and confirm PORTABLE PACKAGE INTEGRITY: PASS.
4. Run Check-Prerequisites.cmd.
5. Start the suite with Launch-Security-Center.cmd.
6. Security Center will automatically ensure DragonForge Agent is running when needed.
7. Follow EXTERNAL-TEST-CHECKLIST.md for the structured smoke test.

REQUIREMENTS
- 64-bit Windows 10/11.
- Microsoft Edge WebView2 Runtime for the Tauri desktop applications.
- Normal-user execution is expected. Phase 12.2 Agent lifecycle does not require elevation.

IMPORTANT TESTING NOTES
- $SigningNotice
- Do not enter irreplaceable production passwords, recovery codes, or secrets during early external testing.
- Keep all packaged executables together.
- Use Stop-DragonForge-Agent.cmd before deleting or moving the test folder.
- Network Guard is visibility-only. Security Scanner is read-only.
- Secure Share is offline and does not provide remote revocation.

INTEGRITY
SHA256SUMS.txt contains SHA-256 hashes for every packaged file.
The GitHub release also includes a SHA-256 file for the ZIP itself.
"@
    Set-Content -LiteralPath (Join-Path $StageRoot "START-HERE.txt") -Value $StartHere -Encoding UTF8

    $Prereq = @'
$ErrorActionPreference = "Stop"
Write-Host "DragonForge Security Suite prerequisite check"
Write-Host ""

$Os = Get-CimInstance Win32_OperatingSystem
Write-Host "Windows: $($Os.Caption) $($Os.Version)"
Write-Host "Architecture: $env:PROCESSOR_ARCHITECTURE"

if (-not [Environment]::Is64BitOperatingSystem) {
    Write-Host "FAIL: This package requires 64-bit Windows." -ForegroundColor Red
    exit 1
}

$WebViewFound = $false
$Roots = @(
    "HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients",
    "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients",
    "HKCU:\SOFTWARE\Microsoft\EdgeUpdate\Clients"
)

foreach ($RegistryRoot in $Roots) {
    if (-not (Test-Path $RegistryRoot)) { continue }
    Get-ChildItem $RegistryRoot -ErrorAction SilentlyContinue | ForEach-Object {
        $Item = Get-ItemProperty $_.PSPath -ErrorAction SilentlyContinue
        $Text = "$($Item.name) $($Item.DisplayName)"
        if ($Text -match "WebView2") {
            $WebViewFound = $true
            Write-Host "WebView2 Runtime: FOUND ($($Item.pv))" -ForegroundColor Green
        }
    }
}

if (-not $WebViewFound) {
    Write-Host "WebView2 Runtime: NOT DETECTED" -ForegroundColor Yellow
    Write-Host "Install the Microsoft Edge WebView2 Evergreen Runtime from Microsoft before launching the desktop apps."
}

$Required = @(
    "dragonforge-desktop.exe",
    "dragonforge-security-center.exe",
    "dragonforge-file-vault.exe",
    "dragonforge-authenticator.exe",
    "dragonforge-security-scanner.exe",
    "dragonforge-integrity-monitor.exe",
    "dragonforge-network-guard.exe",
    "dragonforge-backup-recovery.exe",
    "dragonforge-secure-share.exe",
    "dragonforge-agent.exe",
    "dragonforge-privileged-service.exe"
)

$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$Missing = @()
foreach ($Name in $Required) {
    if (-not (Test-Path -LiteralPath (Join-Path $Root $Name) -PathType Leaf)) {
        $Missing += $Name
    }
}

if ($Missing.Count -gt 0) {
    Write-Host "FAIL: Missing package files:" -ForegroundColor Red
    $Missing | ForEach-Object { Write-Host "  $_" }
    exit 1
}

Write-Host ""
if ($WebViewFound) {
    Write-Host "PASS: Required suite executables and WebView2 were detected." -ForegroundColor Green
    exit 0
}

Write-Host "PARTIAL: Suite files are complete, but WebView2 was not detected." -ForegroundColor Yellow
exit 2
'@
    Set-Content -LiteralPath (Join-Path $StageRoot "Check-Prerequisites.ps1") -Value $Prereq -Encoding UTF8

    $PrereqCmd = @'
@echo off
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0Check-Prerequisites.ps1"
pause
'@
    Set-Content -LiteralPath (Join-Path $StageRoot "Check-Prerequisites.cmd") -Value $PrereqCmd -Encoding ASCII

    $Launch = @'
@echo off
cd /d "%~dp0"
start "" "dragonforge-security-center.exe"
'@
    Set-Content -LiteralPath (Join-Path $StageRoot "Launch-Security-Center.cmd") -Value $Launch -Encoding ASCII

    $StopAgentPs = @'
$ErrorActionPreference = "Stop"
$Root = [System.IO.Path]::GetFullPath((Split-Path -Parent $MyInvocation.MyCommand.Path))
$Expected = [System.IO.Path]::GetFullPath((Join-Path $Root "dragonforge-agent.exe"))
$Matches = Get-Process -Name "dragonforge-agent" -ErrorAction SilentlyContinue | Where-Object {
    try { $_.Path -and ([System.IO.Path]::GetFullPath($_.Path) -eq $Expected) } catch { $false }
}
if (-not $Matches) {
    Write-Host "DragonForge Agent from this package is not running."
    exit 0
}
$Matches | Stop-Process
Write-Host "Stopped DragonForge Agent from this package."
'@
    Set-Content -LiteralPath (Join-Path $StageRoot "Stop-DragonForge-Agent.ps1") -Value $StopAgentPs -Encoding UTF8

    $StopAgentCmd = @'
@echo off
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0Stop-DragonForge-Agent.ps1"
'@
    Set-Content -LiteralPath (Join-Path $StageRoot "Stop-DragonForge-Agent.cmd") -Value $StopAgentCmd -Encoding ASCII

    $VerifyPackagePs = @'
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$Manifest = Join-Path $Root "SHA256SUMS.txt"
if (-not (Test-Path -LiteralPath $Manifest -PathType Leaf)) {
    Write-Host "FAIL: SHA256SUMS.txt is missing." -ForegroundColor Red
    exit 1
}
$Failed = $false
Get-Content -LiteralPath $Manifest | ForEach-Object {
    if ($_ -notmatch '^([A-Fa-f0-9]{64})  (.+)$') {
        Write-Host "FAIL: malformed checksum line: $_" -ForegroundColor Red
        $Failed = $true
        return
    }
    $Expected = $Matches[1].ToUpperInvariant()
    $Name = $Matches[2]
    $Path = Join-Path $Root $Name
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        Write-Host "FAIL  $Name (missing)" -ForegroundColor Red
        $Failed = $true
        return
    }
    $Actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $Path).Hash.ToUpperInvariant()
    if ($Actual -ne $Expected) {
        Write-Host "FAIL  $Name (checksum mismatch)" -ForegroundColor Red
        $Failed = $true
    } else {
        Write-Host "OK    $Name"
    }
}
if ($Failed) { exit 1 }
Write-Host ""
Write-Host "PORTABLE PACKAGE INTEGRITY: PASS" -ForegroundColor Green
'@
    Set-Content -LiteralPath (Join-Path $StageRoot "Verify-Package.ps1") -Value $VerifyPackagePs -Encoding UTF8

    $VerifyPackageCmd = @'
@echo off
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0Verify-Package.ps1"
pause
'@
    Set-Content -LiteralPath (Join-Path $StageRoot "Verify-Package.cmd") -Value $VerifyPackageCmd -Encoding ASCII

    if ($SignRelease) {
        $SigningTargets = @($Executables | ForEach-Object { Join-Path $StageRoot $_ })
        & (Join-Path $PSScriptRoot "sign-windows-files.ps1") -Path $SigningTargets -Description "DragonForge Security Suite $Version"
        if ($LASTEXITCODE -ne 0) { throw "Portable executable signing failed." }
    }

    $HashLines = Get-ChildItem -LiteralPath $StageRoot -File |
        Where-Object { $_.Name -ne "SHA256SUMS.txt" } |
        Sort-Object Name |
        ForEach-Object {
            $Hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $_.FullName).Hash
            "$Hash  $($_.Name)"
        }
    Set-Content -LiteralPath (Join-Path $StageRoot "SHA256SUMS.txt") -Value $HashLines -Encoding ASCII

    Compress-Archive -Path $StageRoot -DestinationPath $ZipPath -CompressionLevel Optimal

    $ZipHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $ZipPath).Hash
    "$ZipHash  $(Split-Path -Leaf $ZipPath)" | Set-Content -LiteralPath $ZipHashPath -Encoding ASCII

    if (-not $KeepStage -and (Test-Path -LiteralPath $StageRoot)) {
        Remove-Item -LiteralPath $StageRoot -Recurse -Force
        Write-Host "Removed temporary portable staging directory: $StageRoot"
    }

    Write-Host ""
    Write-Host "WINDOWS PORTABLE RELEASE PACKAGE: PASS"
    Write-Host "Package: $ZipPath"
    Write-Host "SHA256: $ZipHash"
    Write-Host "Commit: $Commit"
}
finally {
    if ($null -eq $PreviousBuildCommit) {
        Remove-Item Env:DRAGONFORGE_BUILD_COMMIT -ErrorAction SilentlyContinue
    } else {
        $env:DRAGONFORGE_BUILD_COMMIT = $PreviousBuildCommit
    }
    Pop-Location
}
