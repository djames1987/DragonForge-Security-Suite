param(
    [Parameter(Mandatory = $true)] [string]$Tag,
    [switch]$SkipBuild,
    [switch]$PortableOnly,
    [switch]$SignRelease
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot

if ($Tag -notmatch '^v(\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?)$') {
    throw "Tag must be v<version>, for example v0.1.0-alpha.3"
}
$Version = $Matches[1]
$IsPrerelease = $Version -match '-'
if (-not $SignRelease) {
    throw "Phase 14 update-capable releases must be Authenticode-signed. Use -SignRelease."
}
$PackageName = "DragonForge-Security-Suite-v$Version-win-x64"
$DistRoot = Join-Path $RepoRoot "dist"
$ZipPath = Join-Path $DistRoot "$PackageName.zip"
$ZipHashPath = "$ZipPath.sha256"
$InstallerPath = Join-Path $DistRoot "$PackageName-setup.exe"
$InstallerHashPath = "$InstallerPath.sha256"
$ManifestPath = Join-Path $DistRoot "release-manifest-$Tag.json"
$ManifestHashPath = "$ManifestPath.sha256"
$NotesPath = Join-Path $DistRoot "release-notes-$Tag.md"
$UpdateChannel = if ($Version -match '-alpha(?:\.|$)') { "alpha" } elseif ($Version -match '-beta(?:\.|$)') { "beta" } elseif (-not $IsPrerelease) { "stable" } else { throw "Only alpha, beta, and stable release channels are supported." }
$UpdateManifestPath = Join-Path $DistRoot "DragonForge-Security-Suite-update-$UpdateChannel.json"
$UpdateManifestHashPath = "$UpdateManifestPath.sha256"

Push-Location $RepoRoot
try {
    if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
        throw "GitHub CLI (gh) is required to publish releases."
    }
    & gh auth status
    if ($LASTEXITCODE -ne 0) { throw "GitHub CLI is not authenticated." }

    $Dirty = (& git status --porcelain)
    if ($Dirty) { throw "Refusing to publish from a dirty working tree." }

    & git fetch origin --tags
    if ($LASTEXITCODE -ne 0) { throw "Unable to refresh Git tags." }

    $Head = (& git rev-parse HEAD).Trim()
    $TaggedCommit = (& git rev-parse "$Tag^{commit}").Trim()
    if ($LASTEXITCODE -ne 0 -or -not $TaggedCommit) { throw "Tag $Tag does not exist." }
    if ($Head -ne $TaggedCommit) { throw "HEAD does not match $Tag." }

    $Previous = $ErrorActionPreference
    try {
        $ErrorActionPreference = "Continue"
        & gh release view $Tag --repo "djames1987/DragonForge-Security-Suite" *> $null
        $Exists = ($LASTEXITCODE -eq 0)
    }
    finally { $ErrorActionPreference = $Previous }
    if ($Exists) { throw "GitHub release $Tag already exists and will not be mutated." }

    if ($PortableOnly) { throw "Phase 14 secure update publication requires the Windows installer." }
    if (-not $env:DRAGONFORGE_UPDATE_SIGNING_KEY_HEX -or -not $env:DRAGONFORGE_UPDATE_KEY_ID) {
        throw "Update manifest signing credentials are required."
    }

    & (Join-Path $PSScriptRoot "build-tagged-windows-release.ps1") -Tag $Tag -SkipBuild:$SkipBuild -NoInstaller:$PortableOnly -SignRelease:$SignRelease
    if ($LASTEXITCODE -ne 0) { throw "Tagged release build/verification failed." }

    & (Join-Path $PSScriptRoot "generate-signed-update-manifest.ps1") -Tag $Tag
    if ($LASTEXITCODE -ne 0) { throw "Signed update manifest generation failed." }

    $Assets = @($ZipPath, $ZipHashPath, $ManifestPath, $ManifestHashPath, $UpdateManifestPath, $UpdateManifestHashPath)
    if (-not $PortableOnly) {
        $Assets += @($InstallerPath, $InstallerHashPath)
    }
    foreach ($Asset in $Assets) {
        if (-not (Test-Path -LiteralPath $Asset -PathType Leaf)) { throw "Verified release asset missing: $Asset" }
    }
    if (-not (Test-Path -LiteralPath $NotesPath -PathType Leaf)) { throw "Generated release notes are missing." }

    $Arguments = @("release", "create", $Tag) + $Assets + @(
        "--repo", "djames1987/DragonForge-Security-Suite",
        "--title", "DragonForge Security Suite $Tag",
        "--notes-file", $NotesPath,
        "--verify-tag"
    )
    if ($Tag -match '-') { $Arguments += "--prerelease" }

    & gh @Arguments
    if ($LASTEXITCODE -ne 0) { throw "GitHub release creation failed." }

    Write-Host ""
    Write-Host "GITHUB RELEASE PUBLISHED: $Tag"
    Write-Host "Commit: $TaggedCommit"
    Write-Host "Artifacts were verified before publication."
}
finally { Pop-Location }
