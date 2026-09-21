param(
    [string]$Version = "0.1.0-alpha.1",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$Tag = "v$Version"
$PackageName = "DragonForge-Security-Suite-v$Version-win-x64"
$ZipPath = Join-Path $RepoRoot "dist\$PackageName.zip"
$HashPath = "$ZipPath.sha256"
$NotesPath = Join-Path $RepoRoot "docs\releases\v0.1.0-alpha.1.md"

Push-Location $RepoRoot
try {
    if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
        throw "GitHub CLI (gh) is required to publish the release. Install gh and authenticate with gh auth login."
    }

    & gh auth status
    if ($LASTEXITCODE -ne 0) {
        throw "GitHub CLI is not authenticated."
    }

    $Dirty = (& git status --porcelain)
    if ($Dirty) {
        throw "Refusing to publish from a dirty working tree. Commit or stash local changes first."
    }

    $Branch = (& git branch --show-current).Trim()
    if ($Branch -ne "main") {
        throw "Publish releases from main. Current branch: $Branch"
    }

    & git fetch origin main --tags
    if ($LASTEXITCODE -ne 0) {
        throw "Unable to refresh origin/main and tags."
    }

    $Local = (& git rev-parse HEAD).Trim()
    $Remote = (& git rev-parse origin/main).Trim()
    if ($Local -ne $Remote) {
        throw "Local main is not exactly origin/main. Pull before publishing."
    }

    & (Join-Path $PSScriptRoot "package-windows-release.ps1") -Version $Version -SkipBuild:$SkipBuild
    if ($LASTEXITCODE -ne 0) {
        throw "Portable release packaging failed."
    }

    if (-not (Test-Path -LiteralPath $NotesPath -PathType Leaf)) {
        throw "Release notes not found: $NotesPath"
    }

    & gh release view $Tag --repo "djames1987/DragonForge-Security-Suite" 2>$null | Out-Null
    if ($LASTEXITCODE -eq 0) {
        throw "GitHub release $Tag already exists."
    }

    $Arguments = @(
        "release", "create", $Tag,
        $ZipPath,
        $HashPath,
        "--repo", "djames1987/DragonForge-Security-Suite",
        "--target", $Local,
        "--title", "DragonForge Security Suite $Tag",
        "--notes-file", $NotesPath,
        "--prerelease"
    )
    & gh @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "GitHub release creation failed."
    }

    Write-Host ""
    Write-Host "GITHUB PRE-RELEASE PUBLISHED: $Tag"
    Write-Host "Commit: $Local"
    & gh release view $Tag --repo "djames1987/DragonForge-Security-Suite"
}
finally {
    Pop-Location
}
