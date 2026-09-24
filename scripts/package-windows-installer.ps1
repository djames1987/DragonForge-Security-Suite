param(
    [string]$Version = "0.1.0-alpha.2",
    [switch]$SkipBuild,
    [string]$ExpectedCommit,
    [string]$ReleaseTag,
    [switch]$SignRelease,
    [string]$InstallerAppId,
    [string]$InstallerGroupName,
    [string]$InstallerStartupName
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$DistRoot = Join-Path $RepoRoot "dist"
$PackageName = "DragonForge-Security-Suite-v$Version-win-x64"
$PortableStage = Join-Path $DistRoot $PackageName
$InstallerStage = Join-Path $DistRoot "$PackageName-installer-stage"
$InstallerName = "$PackageName-setup.exe"
$InstallerPath = Join-Path $DistRoot $InstallerName
$InstallerHashPath = "$InstallerPath.sha256"
$IssPath = Join-Path $RepoRoot "installer\DragonForgeSecuritySuite.iss"

$InstallerFiles = @(
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
    "dragonforge-privileged-service.exe",
    "install-privileged-service.ps1",
    "install-privileged-service.cmd",
    "uninstall-privileged-service.ps1",
    "uninstall-privileged-service.cmd",
    "test-privileged-service.ps1",
    "SECURITY.md",
    "EXTERNAL-TEST-CHECKLIST.md",
    "Check-Prerequisites.ps1",
    "Check-Prerequisites.cmd",
    "Stop-DragonForge-Agent.ps1",
    "Stop-DragonForge-Agent.cmd",
    "Verify-Package.ps1",
    "Verify-Package.cmd"
)

function Find-InnoCompiler {
    $Command = Get-Command "ISCC.exe" -ErrorAction SilentlyContinue
    if ($Command) { return $Command.Source }
    $Candidates = @()
    if ($env:LOCALAPPDATA) { $Candidates += (Join-Path $env:LOCALAPPDATA "Programs\Inno Setup 6\ISCC.exe") }
    if ($env:LOCALAPPDATA) { $Candidates += (Join-Path $env:LOCALAPPDATA "Inno Setup 6\ISCC.exe") }
    if (${env:ProgramFiles(x86)}) { $Candidates += (Join-Path ${env:ProgramFiles(x86)} "Inno Setup 6\ISCC.exe") }
    if ($env:ProgramFiles) { $Candidates += (Join-Path $env:ProgramFiles "Inno Setup 6\ISCC.exe") }
    foreach ($Candidate in $Candidates) {
        if ($Candidate -and (Test-Path -LiteralPath $Candidate -PathType Leaf)) { return $Candidate }
    }
    throw "Inno Setup 6 was not found. Install it with: winget install --id JRSoftware.InnoSetup -e"
}

if ($env:OS -ne "Windows_NT") { throw "The DragonForge Windows installer must be built on Windows." }

Push-Location $RepoRoot
try {
    if (-not (Test-Path -LiteralPath $IssPath -PathType Leaf)) { throw "Installer definition not found: $IssPath" }
    $Commit = (& git rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0 -or -not $Commit) { throw "Unable to determine the installer source commit." }
    if ($ExpectedCommit -and $Commit -ne $ExpectedCommit) { throw "Installer source commit mismatch." }
    if ($ReleaseTag) {
        $TaggedCommit = (& git rev-parse "$ReleaseTag^{commit}").Trim()
        if ($LASTEXITCODE -ne 0 -or $TaggedCommit -ne $Commit) { throw "Release tag $ReleaseTag does not resolve to the current commit." }
    }
    $ReleaseChannel = if ($Version -match '-') { "pre-release / external testing" } else { "stable" }

    & (Join-Path $PSScriptRoot "package-windows-release.ps1") -Version $Version -SkipBuild:$SkipBuild -KeepStage -ExpectedCommit $Commit -ReleaseTag $ReleaseTag -SignRelease:$SignRelease
    if ($LASTEXITCODE -ne 0) { throw "Portable staging failed before installer compilation." }
    if (-not (Test-Path -LiteralPath $PortableStage -PathType Container)) { throw "Portable staging directory is missing: $PortableStage" }

    if (Test-Path -LiteralPath $InstallerStage) { Remove-Item -LiteralPath $InstallerStage -Recurse -Force }
    New-Item -ItemType Directory -Force -Path $InstallerStage | Out-Null
    foreach ($Name in $InstallerFiles) {
        $Source = Join-Path $PortableStage $Name
        if (-not (Test-Path -LiteralPath $Source -PathType Leaf)) { throw "Required installer payload file is missing: $Source" }
        Copy-Item -LiteralPath $Source -Destination (Join-Path $InstallerStage $Name)
    }

    $BuildInfo = @"
DragonForge Security Suite
Version: v$Version
Release channel: $ReleaseChannel
Git commit: $Commit
Git tag: $(if ($ReleaseTag) { $ReleaseTag } else { "un-tagged" })
Built (UTC): $([DateTime]::UtcNow.ToString("o"))
Platform: Windows x64 installer
Code signing: $(if ($SignRelease) { "Authenticode SHA-256 + RFC 3161 timestamp" } else { "unsigned development/test build" })
"@
    Set-Content -LiteralPath (Join-Path $InstallerStage "BUILD-INFO.txt") -Value $BuildInfo -Encoding UTF8

    $HashLines = Get-ChildItem -LiteralPath $InstallerStage -File |
        Where-Object { $_.Name -ne "SHA256SUMS.txt" } |
        Sort-Object Name |
        ForEach-Object {
            $Hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $_.FullName).Hash
            "$Hash  $($_.Name)"
        }
    Set-Content -LiteralPath (Join-Path $InstallerStage "SHA256SUMS.txt") -Value $HashLines -Encoding ASCII

    $Iscc = Find-InnoCompiler
    if (Test-Path -LiteralPath $InstallerPath) { Remove-Item -LiteralPath $InstallerPath -Force }
    if (Test-Path -LiteralPath $InstallerHashPath) { Remove-Item -LiteralPath $InstallerHashPath -Force }

    $Arguments = @(
        "/DMyAppVersion=$Version",
        "/DStageDir=$InstallerStage",
        "/DOutputDir=$DistRoot",
        "/DBuildCommit=$Commit",
        $IssPath
    )
    if ($InstallerAppId) {
        $Arguments = @("/DMyAppId=$InstallerAppId") + $Arguments
    }
    if ($InstallerGroupName) {
        $Arguments = @("/DMyGroupName=$InstallerGroupName") + $Arguments
    }
    if ($InstallerStartupName) {
        $Arguments = @("/DMyStartupName=$InstallerStartupName") + $Arguments
    }
    Write-Host ""
    Write-Host ">>> $Iscc $($Arguments -join ' ')"
    & $Iscc @Arguments
    if ($LASTEXITCODE -ne 0) { throw "Inno Setup compilation failed with exit code $LASTEXITCODE." }
    if (-not (Test-Path -LiteralPath $InstallerPath -PathType Leaf)) { throw "Expected installer artifact was not found: $InstallerPath" }

    if ($SignRelease) {
        & (Join-Path $PSScriptRoot "sign-windows-files.ps1") -Path @($InstallerPath) -Description "DragonForge Security Suite Installer $Version"
        if ($LASTEXITCODE -ne 0) { throw "Installer Authenticode signing failed." }
    }

    $InstallerHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $InstallerPath).Hash
    "$InstallerHash  $InstallerName" | Set-Content -LiteralPath $InstallerHashPath -Encoding ASCII
    Write-Host ""
    Write-Host "WINDOWS INSTALLER PACKAGE: PASS"
    Write-Host "Installer: $InstallerPath"
    Write-Host "SHA256: $InstallerHash"
    Write-Host "Commit: $Commit"
}
finally {
    foreach ($TemporaryPath in @($InstallerStage, $PortableStage)) {
        if ($TemporaryPath -and (Test-Path -LiteralPath $TemporaryPath)) {
            Remove-Item -LiteralPath $TemporaryPath -Recurse -Force -ErrorAction SilentlyContinue
            Write-Host "Removed temporary staging directory: $TemporaryPath"
        }
    }
    Pop-Location
}
