param(
    [Parameter(Mandatory = $true)] [string]$Tag,
    [switch]$SkipBuild,
    [switch]$NoInstaller
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot

if ($Tag -notmatch '^v(\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?)$') {
    throw "Tag must be v<version>, for example v0.1.0-alpha.3"
}
$Version = $Matches[1]

Push-Location $RepoRoot
try {
    if ($env:OS -ne "Windows_NT") { throw "Tagged Windows releases must be built on Windows." }

    $Dirty = (& git status --porcelain)
    if ($Dirty) { throw "Refusing to build a tagged release from a dirty working tree." }

    & git fetch origin --tags
    if ($LASTEXITCODE -ne 0) { throw "Unable to refresh Git tags." }

    $Head = (& git rev-parse HEAD).Trim()
    $TaggedCommit = (& git rev-parse "$Tag^{commit}").Trim()
    if ($LASTEXITCODE -ne 0 -or -not $TaggedCommit) { throw "Tag $Tag does not exist." }
    if ($Head -ne $TaggedCommit) { throw "HEAD $Head is not the tagged release commit $TaggedCommit." }

    & git ls-files --error-unmatch Cargo.lock *> $null
    if ($LASTEXITCODE -ne 0) { throw "Cargo.lock must be tracked for release builds." }

    & (Join-Path $PSScriptRoot "set-release-version.ps1") -Version $Version -CheckOnly
    if ($LASTEXITCODE -ne 0) { throw "Suite version stamps do not match $Version." }

    & cargo metadata --locked --format-version 1 --no-deps *> $null
    if ($LASTEXITCODE -ne 0) { throw "Cargo.lock does not match the tagged manifests." }

    if ($NoInstaller) {
        & (Join-Path $PSScriptRoot "package-windows-release.ps1") -Version $Version -SkipBuild:$SkipBuild -ExpectedCommit $TaggedCommit -ReleaseTag $Tag
    } else {
        & (Join-Path $PSScriptRoot "package-windows-installer.ps1") -Version $Version -SkipBuild:$SkipBuild -ExpectedCommit $TaggedCommit -ReleaseTag $Tag
    }
    if ($LASTEXITCODE -ne 0) { throw "Tagged Windows packaging failed." }

    & (Join-Path $PSScriptRoot "generate-release-notes.ps1") -Tag $Tag
    if ($LASTEXITCODE -ne 0) { throw "Release notes generation failed." }

    & (Join-Path $PSScriptRoot "verify-release-artifacts.ps1") -Tag $Tag -RequireInstaller:(-not $NoInstaller)
    if ($LASTEXITCODE -ne 0) { throw "Release artifact verification failed." }

    Write-Host ""
    Write-Host "TAGGED WINDOWS RELEASE BUILD: PASS"
    Write-Host "Tag: $Tag"
    Write-Host "Commit: $TaggedCommit"
}
finally { Pop-Location }
