param(
    [Parameter(Mandatory = $true)] [string]$Tag
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$DistRoot = Join-Path $RepoRoot "dist"

if ($Tag -notmatch '^v(\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?)$') {
    throw "Tag must be v<version>."
}
$Version = $Matches[1]
$Channel = if ($Version -match '-alpha(?:\.|$)') {
    "alpha"
}
elseif ($Version -match '-beta(?:\.|$)') {
    "beta"
}
elseif ($Version -notmatch '-') {
    "stable"
}
else {
    throw "Phase 14 supports only alpha, beta, and stable release channels."
}

if (-not $env:DRAGONFORGE_UPDATE_SIGNING_KEY_HEX) {
    throw "DRAGONFORGE_UPDATE_SIGNING_KEY_HEX is required to publish a signed update manifest."
}
if (-not $env:DRAGONFORGE_UPDATE_KEY_ID) {
    throw "DRAGONFORGE_UPDATE_KEY_ID is required to publish a signed update manifest."
}

$ReleaseManifestPath = Join-Path $DistRoot "release-manifest-$Tag.json"
if (-not (Test-Path -LiteralPath $ReleaseManifestPath -PathType Leaf)) {
    throw "Verified release manifest is missing: $ReleaseManifestPath"
}
$ReleaseManifest = Get-Content -Raw -LiteralPath $ReleaseManifestPath | ConvertFrom-Json
if ($ReleaseManifest.tag -ne $Tag -or $ReleaseManifest.version -ne $Version) {
    throw "Release manifest identity does not match $Tag."
}
$Installer = @($ReleaseManifest.artifacts | Where-Object { $_.kind -eq "windows-installer" })
if ($Installer.Count -ne 1) {
    throw "Exactly one verified Windows installer is required for secure updates."
}

$PayloadPath = Join-Path $DistRoot "update-payload-$Tag.json"
$OutputPath = Join-Path $DistRoot "DragonForge-Security-Suite-update-$Channel.json"
$OutputHashPath = "$OutputPath.sha256"
$AssetUrl = "https://github.com/djames1987/DragonForge-Security-Suite/releases/download/$Tag/$($Installer[0].name)"

$Payload = [ordered]@{
    schema_version = 1
    channel = $Channel
    version = $Version
    tag = $Tag
    commit = $ReleaseManifest.commit
    published_utc = [DateTime]::UtcNow.ToString("o")
    artifacts = @(
        [ordered]@{
            name = $Installer[0].name
            kind = "windows-installer"
            url = $AssetUrl
            sha256 = $Installer[0].sha256
            bytes = [uint64]$Installer[0].bytes
            authenticode_required = $true
        }
    )
}

$Utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllText(
    $PayloadPath,
    ($Payload | ConvertTo-Json -Depth 6),
    $Utf8NoBom
)

Push-Location $RepoRoot
try {
    & cargo run --locked -p dragonforge-update --bin dragonforge-sign-update-manifest -- $PayloadPath $OutputPath
    if ($LASTEXITCODE -ne 0) { throw "Update manifest signing failed." }

    $Hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $OutputPath).Hash
    "$Hash  $(Split-Path -Leaf $OutputPath)" | Set-Content -LiteralPath $OutputHashPath -Encoding ASCII

    Write-Host "SIGNED UPDATE MANIFEST: PASS"
    Write-Host "Channel: $Channel"
    Write-Host "Manifest: $OutputPath"
    Write-Host "SHA256: $Hash"
}
finally {
    Pop-Location
    Remove-Item -LiteralPath $PayloadPath -Force -ErrorAction SilentlyContinue
}
