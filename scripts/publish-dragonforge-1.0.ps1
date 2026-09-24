param(
    [string]$Tag = "v1.0.0"
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
if ($Tag -ne "v1.0.0") {
    throw "Phase 25 stable publisher is intentionally pinned to v1.0.0."
}

Push-Location $RepoRoot
try {
    if ($env:OS -ne "Windows_NT") { throw "DragonForge 1.0 stable publication must run on Windows." }

    $Dirty = @(& git status --porcelain)
    if ($LASTEXITCODE -ne 0) { throw "Unable to inspect Git working tree." }
    if ($Dirty.Count -gt 0) { throw "Refusing DragonForge 1.0 publication from a dirty working tree." }

    & (Join-Path $PSScriptRoot "set-release-version.ps1") -Version "1.0.0" -CheckOnly
    if ($LASTEXITCODE -ne 0) { throw "Release source is not stamped consistently to 1.0.0." }

    $HasStoreIdentity = [bool]$env:DRAGONFORGE_SIGN_CERT_THUMBPRINT
    $HasPfxIdentity = [bool]$env:DRAGONFORGE_SIGN_PFX_PATH -and [bool]$env:DRAGONFORGE_SIGN_PFX_PASSWORD
    if ($HasStoreIdentity -eq $HasPfxIdentity) {
        throw "Configure exactly one Authenticode signing identity: certificate-store thumbprint or PFX path/password."
    }
    if (-not $env:DRAGONFORGE_SIGN_TIMESTAMP_URL) {
        throw "RFC 3161 timestamp service configuration is required."
    }
    foreach ($Name in @(
        "DRAGONFORGE_UPDATE_PUBLIC_KEY_HEX",
        "DRAGONFORGE_UPDATE_KEY_ID",
        "DRAGONFORGE_UPDATE_SIGNING_KEY_HEX"
    )) {
        if (-not [Environment]::GetEnvironmentVariable($Name)) {
            throw "$Name is required for the stable release."
        }
    }

    if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
        throw "GitHub CLI (gh) is required."
    }
    & gh auth status
    if ($LASTEXITCODE -ne 0) { throw "GitHub CLI is not authenticated." }

    & git fetch origin --tags
    if ($LASTEXITCODE -ne 0) { throw "Unable to refresh Git tags." }
    $Head = (& git rev-parse HEAD).Trim()
    $TaggedCommit = (& git rev-parse "$Tag^{commit}").Trim()
    if ($LASTEXITCODE -ne 0 -or -not $TaggedCommit) { throw "Tag $Tag does not exist." }
    if ($Head -ne $TaggedCommit) { throw "HEAD $Head is not the v1.0.0 tagged commit $TaggedCommit." }

    & (Join-Path $PSScriptRoot "run-phase25-1.0-release-tests.ps1")
    if ($LASTEXITCODE -ne 0) { throw "Phase 25 release-readiness verification failed." }

    & (Join-Path $PSScriptRoot "publish-windows-release.ps1") -Tag $Tag -SignRelease
    if ($LASTEXITCODE -ne 0) { throw "Signed DragonForge 1.0 publication failed." }

    $ReleaseJson = & gh release view $Tag --repo "djames1987/DragonForge-Security-Suite" --json tagName,isPrerelease,url,publishedAt,assets
    if ($LASTEXITCODE -ne 0) { throw "Unable to inspect the published GitHub release." }
    $Release = $ReleaseJson | ConvertFrom-Json
    if ($Release.tagName -ne $Tag -or $Release.isPrerelease) {
        throw "Published v1.0.0 release identity/channel is invalid."
    }

    $RequiredAssets = @(
        "DragonForge-Security-Suite-v1.0.0-win-x64.zip",
        "DragonForge-Security-Suite-v1.0.0-win-x64.zip.sha256",
        "DragonForge-Security-Suite-v1.0.0-win-x64-setup.exe",
        "DragonForge-Security-Suite-v1.0.0-win-x64-setup.exe.sha256",
        "release-manifest-v1.0.0.json",
        "release-manifest-v1.0.0.json.sha256",
        "DragonForge-Security-Suite-update-stable.json",
        "DragonForge-Security-Suite-update-stable.json.sha256"
    )
    $PublishedAssets = @($Release.assets | ForEach-Object { $_.name })
    foreach ($Name in $RequiredAssets) {
        if ($Name -notin $PublishedAssets) { throw "Published DragonForge 1.0 release is missing $Name." }
    }

    $Receipt = [ordered]@{
        schema_version = 1
        release = "DragonForge Security Suite 1.0"
        version = "1.0.0"
        tag = $Tag
        commit = $TaggedCommit
        release_url = $Release.url
        published_at = $Release.publishedAt
        prerelease = [bool]$Release.isPrerelease
        required_assets = $RequiredAssets
        verified_utc = [DateTime]::UtcNow.ToString("o")
    }
    $LogDir = Join-Path $RepoRoot "test-logs"
    New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
    $ReceiptPath = Join-Path $LogDir "dragonforge-v1.0.0-release-receipt.json"
    $Receipt | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $ReceiptPath -Encoding UTF8
    $ReceiptHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $ReceiptPath).Hash
    "$ReceiptHash  $([IO.Path]::GetFileName($ReceiptPath))" | Set-Content -Encoding ascii "$ReceiptPath.sha256"

    Write-Host ""
    Write-Host "DRAGONFORGE SECURITY SUITE 1.0 RELEASE: PASS"
    Write-Host "Tag: $Tag"
    Write-Host "Commit: $TaggedCommit"
    Write-Host "Release: $($Release.url)"
    Write-Host "Receipt: $ReceiptPath"
    Write-Host "Receipt SHA256: $ReceiptHash"
}
finally {
    Pop-Location
}
