param(
    [string]$Version = "0.1.0-alpha.2",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$DistRoot = Join-Path $RepoRoot "dist"
$PackageName = "DragonForge-Security-Suite-v$Version-win-x64"
$StageRoot = Join-Path $DistRoot $PackageName
$InstallerName = "$PackageName-setup.exe"
$InstallerPath = Join-Path $DistRoot $InstallerName
$InstallerHashPath = "$InstallerPath.sha256"
$IssPath = Join-Path $RepoRoot "installer\DragonForgeSecuritySuite.iss"

function Find-InnoCompiler {
    $Command = Get-Command "ISCC.exe" -ErrorAction SilentlyContinue
    if ($Command) {
        return $Command.Source
    }

    $Candidates = @()
    if (${env:ProgramFiles(x86)}) {
        $Candidates += (Join-Path ${env:ProgramFiles(x86)} "Inno Setup 6\ISCC.exe")
    }
    if ($env:ProgramFiles) {
        $Candidates += (Join-Path $env:ProgramFiles "Inno Setup 6\ISCC.exe")
    }

    foreach ($Candidate in $Candidates) {
        if ($Candidate -and (Test-Path -LiteralPath $Candidate -PathType Leaf)) {
            return $Candidate
        }
    }

    throw "Inno Setup 6 was not found. Install it with: winget install --id JRSoftware.InnoSetup -e"
}

if ($env:OS -ne "Windows_NT") {
    throw "The DragonForge Windows installer must be built on Windows."
}

Push-Location $RepoRoot
try {
    if (-not (Test-Path -LiteralPath $IssPath -PathType Leaf)) {
        throw "Installer definition not found: $IssPath"
    }

    $Commit = (& git rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0 -or -not $Commit) {
        throw "Unable to determine the installer source commit."
    }

    & (Join-Path $PSScriptRoot "package-windows-release.ps1") -Version $Version -SkipBuild:$SkipBuild
    if ($LASTEXITCODE -ne 0) {
        throw "Portable staging failed before installer compilation."
    }

    if (-not (Test-Path -LiteralPath $StageRoot -PathType Container)) {
        throw "Portable staging directory is missing: $StageRoot"
    }

    $Iscc = Find-InnoCompiler

    if (Test-Path -LiteralPath $InstallerPath) {
        Remove-Item -LiteralPath $InstallerPath -Force
    }
    if (Test-Path -LiteralPath $InstallerHashPath) {
        Remove-Item -LiteralPath $InstallerHashPath -Force
    }

    $Arguments = @(
        "/Qp",
        "/DMyAppVersion=$Version",
        "/DStageDir=$StageRoot",
        "/DOutputDir=$DistRoot",
        "/DBuildCommit=$Commit",
        $IssPath
    )

    Write-Host ""
    Write-Host ">>> $Iscc $($Arguments -join ' ')"
    & $Iscc @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "Inno Setup compilation failed with exit code $LASTEXITCODE."
    }

    if (-not (Test-Path -LiteralPath $InstallerPath -PathType Leaf)) {
        throw "Installer compiler completed but expected artifact was not found: $InstallerPath"
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
    Pop-Location
}
