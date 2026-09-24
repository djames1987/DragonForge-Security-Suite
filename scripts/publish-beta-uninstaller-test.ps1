param(
    [string]$Version = "1.0.0-beta.1",
    [string]$Tag = "v1.0.0-beta.1"
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$LocalTagCreated = $false

if ($Version -ne "1.0.0-beta.1" -or $Tag -ne "v1.0.0-beta.1") {
    throw "This publisher is intentionally pinned to DragonForge v1.0.0-beta.1."
}
if ($env:OS -ne "Windows_NT") { throw "The beta installer test release must be produced on Windows." }

Push-Location $RepoRoot
try {
    if (-not (Get-Command gh -ErrorAction SilentlyContinue)) { throw "GitHub CLI (gh) is required." }
    & gh auth status
    if ($LASTEXITCODE -ne 0) { throw "GitHub CLI is not authenticated." }

    $Dirty = @(& git status --porcelain)
    if ($LASTEXITCODE -ne 0) { throw "Unable to inspect Git working tree." }
    if ($Dirty.Count -gt 0) {
        $Dirty | ForEach-Object { Write-Host "  $_" }
        throw "Refusing beta publication from a dirty working tree."
    }

    & (Join-Path $PSScriptRoot "set-release-version.ps1") -Version $Version -CheckOnly
    if ($LASTEXITCODE -ne 0) { throw "Beta release branch is not fully stamped to $Version." }

    $Branch = (& git branch --show-current).Trim()
    if ($Branch -ne "release/v1.0.0-beta.1-uninstaller") {
        throw "Run this publisher only from release/v1.0.0-beta.1-uninstaller."
    }

    & git fetch origin --tags
    if ($LASTEXITCODE -ne 0) { throw "Unable to refresh Git tags." }
    $RemoteTagOutput = @(& git ls-remote --tags origin "refs/tags/$Tag")
    if ($LASTEXITCODE -ne 0) { throw "Unable to inspect remote tag $Tag." }
    $RemoteTag = ($RemoteTagOutput -join "").Trim()
    if ($RemoteTag) { throw "Remote tag $Tag already exists; refusing to mutate a published candidate identity." }

    $LocalTagOutput = @(& git tag --list $Tag)
    if ($LASTEXITCODE -ne 0) { throw "Unable to inspect local tag $Tag." }
    $LocalTag = ($LocalTagOutput -join "").Trim()
    if ($LocalTag) {
        Write-Host "Removing stale local-only tag $Tag from a previous failed publication attempt."
        & git tag -d $Tag
        if ($LASTEXITCODE -ne 0) { throw "Unable to remove stale local tag $Tag." }
    }

    $Previous = $ErrorActionPreference
    try {
        $ErrorActionPreference = "Continue"
        & gh release view $Tag --repo "djames1987/DragonForge-Security-Suite" *> $null
        $ReleaseExists = ($LASTEXITCODE -eq 0)
    }
    finally { $ErrorActionPreference = $Previous }
    if ($ReleaseExists) { throw "GitHub release $Tag already exists." }

    Write-Host ""
    Write-Host ">>> Verify beta workspace"
    & cargo fmt --all --check
    if ($LASTEXITCODE -ne 0) { throw "cargo fmt failed." }
    & cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw "cargo clippy failed." }
    & cargo test --workspace --all-features --locked
    if ($LASTEXITCODE -ne 0) { throw "cargo test failed." }

    Write-Host ""
    Write-Host ">>> Build release-profile suite once"
    & (Join-Path $PSScriptRoot "build-all-apps-for-testing.ps1") -Profile release
    if ($LASTEXITCODE -ne 0) { throw "Release-profile suite build failed." }

    $Token = [guid]::NewGuid().ToString("N").Substring(0, 8)
    $TestAppId = "DragonForge-Uninstaller-Test-$Token"
    $TestGroup = "DragonForge Uninstaller Test $Token"
    $TestStartup = "DragonForge Agent Uninstaller Test $Token"

    Write-Host ""
    Write-Host ">>> Build isolated installer for lifecycle test"
    & (Join-Path $PSScriptRoot "package-windows-installer.ps1") `
        -Version $Version `
        -SkipBuild `
        -InstallerAppId $TestAppId `
        -InstallerGroupName $TestGroup `
        -InstallerStartupName $TestStartup
    if ($LASTEXITCODE -ne 0) { throw "Isolated lifecycle-test installer build failed." }

    $DistRoot = Join-Path $RepoRoot "dist"
    $Base = "DragonForge-Security-Suite-v$Version-win-x64"
    $Installer = Join-Path $DistRoot "$Base-setup.exe"

    Write-Host ""
    Write-Host ">>> Exercise isolated real installer/uninstaller lifecycle"
    & (Join-Path $PSScriptRoot "test-windows-installer-uninstall.ps1") `
        -Version $Version `
        -InstallerPath $Installer `
        -AllowLocal `
        -StartMenuGroupName $TestGroup `
        -StartupShortcutName $TestStartup
    if ($LASTEXITCODE -ne 0) { throw "Installer/uninstaller lifecycle test failed." }

    $LifecycleLog = Get-ChildItem (Join-Path $RepoRoot "test-logs\dragonforge-installer-uninstall-$Version-*.log") |
        Sort-Object LastWriteTime -Descending |
        Select-Object -First 1
    if (-not $LifecycleLog) { throw "Lifecycle evidence log was not produced." }

    $PostLifecycleDirty = @(& git status --porcelain)
    if ($LASTEXITCODE -ne 0) { throw "Unable to inspect working tree after lifecycle verification." }
    if ($PostLifecycleDirty.Count -gt 0) {
        Write-Host ""
        Write-Host "Tracked/untracked working-tree changes detected after beta build/lifecycle:"
        $PostLifecycleDirty | ForEach-Object { Write-Host "  $_" }
        Write-Host ""
        & git diff --stat
        & git diff -- Cargo.lock
        throw "Beta build/lifecycle modified the working tree; refusing to create a release tag."
    }

    $Head = (& git rev-parse HEAD).Trim()
    & git tag -a $Tag -m "DragonForge Security Suite $Tag uninstaller test release" $Head
    if ($LASTEXITCODE -ne 0) { throw "Unable to create local annotated tag $Tag." }
    $LocalTagCreated = $true

    Write-Host ""
    Write-Host ">>> Rebuild normal-AppId exact-tag beta artifacts"
    & (Join-Path $PSScriptRoot "build-tagged-windows-release.ps1") -Tag $Tag -SkipBuild
    if ($LASTEXITCODE -ne 0) { throw "Exact-tag beta release build/verification failed." }

    $Notes = Join-Path $DistRoot "release-notes-$Tag.md"
    @(
        "",
        "## Testing focus",
        "",
        "This unsigned prerelease validates installation and complete uninstall behavior before DragonForge Security Suite 1.0.",
        "",
        "Before publication, the release host performed a real isolated install, started the exact installed DragonForge Agent, ran the generated uninstaller, verified Agent shutdown, verified installer-managed directory and shortcut removal, and verified data outside the installation directory was preserved.",
        "",
        "**This is not the signed stable release and is not eligible for the stable update channel.**"
    ) | Add-Content -LiteralPath $Notes

    Write-Host ""
    Write-Host ">>> Push verified tag and publish GitHub prerelease"
    & git push origin "refs/tags/$Tag"
    if ($LASTEXITCODE -ne 0) { throw "Unable to push verified beta tag." }

    $Assets = @(
        (Join-Path $DistRoot "$Base.zip"),
        (Join-Path $DistRoot "$Base.zip.sha256"),
        (Join-Path $DistRoot "$Base-setup.exe"),
        (Join-Path $DistRoot "$Base-setup.exe.sha256"),
        (Join-Path $DistRoot "release-manifest-$Tag.json"),
        (Join-Path $DistRoot "release-manifest-$Tag.json.sha256"),
        $LifecycleLog.FullName,
        ($LifecycleLog.FullName + ".sha256")
    )
    foreach ($Asset in $Assets) {
        if (-not (Test-Path -LiteralPath $Asset -PathType Leaf)) { throw "Release asset missing: $Asset" }
    }

    & gh release create $Tag @Assets `
        --repo "djames1987/DragonForge-Security-Suite" `
        --title "DragonForge Security Suite $Tag - Uninstaller Test" `
        --notes-file $Notes `
        --verify-tag `
        --prerelease
    if ($LASTEXITCODE -ne 0) { throw "GitHub prerelease creation failed." }

    Write-Host ""
    Write-Host "DRAGONFORGE BETA UNINSTALLER TEST RELEASE: PASS"
    Write-Host "Tag: $Tag"
    Write-Host "Commit: $Head"
    Write-Host "Lifecycle log: $($LifecycleLog.FullName)"
}
catch {
    $OriginalError = $_
    if ($LocalTagCreated) {
        $PreviousCleanupPreference = $ErrorActionPreference
        try {
            $ErrorActionPreference = "Continue"
            $RemoteCleanupOutput = @(& git ls-remote --tags origin "refs/tags/$Tag")
            $RemoteCleanupExit = $LASTEXITCODE
            $RemoteCleanup = ($RemoteCleanupOutput -join "").Trim()
            if ($RemoteCleanupExit -eq 0 -and -not $RemoteCleanup) {
                & git tag -d $Tag *> $null
            }
        }
        finally {
            $ErrorActionPreference = $PreviousCleanupPreference
        }
    }
    throw $OriginalError
}
finally {
    Pop-Location
}
