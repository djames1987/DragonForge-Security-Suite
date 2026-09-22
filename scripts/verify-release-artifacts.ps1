param(
    [Parameter(Mandatory = $true)] [string]$Tag,
    [switch]$RequireInstaller,
    [switch]$RequireSigning
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$DistRoot = Join-Path $RepoRoot "dist"

if ($Tag -notmatch '^v(\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?)$') {
    throw "Tag must be v<version>, for example v0.1.0-alpha.3"
}
$Version = $Matches[1]
$PackageName = "DragonForge-Security-Suite-v$Version-win-x64"
$ZipPath = Join-Path $DistRoot "$PackageName.zip"
$ZipSidecar = "$ZipPath.sha256"
$InstallerPath = Join-Path $DistRoot "$PackageName-setup.exe"
$InstallerSidecar = "$InstallerPath.sha256"
$ManifestPath = Join-Path $DistRoot "release-manifest-$Tag.json"
$ManifestSidecar = "$ManifestPath.sha256"
$ExpectedExecutables = @(
    "dragonforge-desktop.exe",
    "dragonforge-security-center.exe",
    "dragonforge-file-vault.exe",
    "dragonforge-authenticator.exe",
    "dragonforge-security-scanner.exe",
    "dragonforge-integrity-monitor.exe",
    "dragonforge-network-guard.exe",
    "dragonforge-backup-recovery.exe",
    "dragonforge-secure-share.exe",
    "dragonforge-agent.exe"
)

function Verify-Sidecar {
    param([string]$Artifact, [string]$Sidecar)
    if (-not (Test-Path -LiteralPath $Artifact -PathType Leaf)) { throw "Artifact missing: $Artifact" }
    if (-not (Test-Path -LiteralPath $Sidecar -PathType Leaf)) { throw "Checksum sidecar missing: $Sidecar" }
    $Expected = ((Get-Content -LiteralPath $Sidecar -Raw).Trim() -split '\s+')[0].ToUpperInvariant()
    $Actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $Artifact).Hash.ToUpperInvariant()
    if ($Expected -ne $Actual) { throw "SHA-256 mismatch for $Artifact" }
    return $Actual
}

Push-Location $RepoRoot
$TempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("dragonforge-release-verify-" + [guid]::NewGuid().ToString("N"))
try {
    $Commit = (& git rev-parse "$Tag^{commit}").Trim()
    if ($LASTEXITCODE -ne 0 -or -not $Commit) { throw "Unable to resolve $Tag." }

    $ZipHash = Verify-Sidecar $ZipPath $ZipSidecar
    $InstallerHash = $null
    if ($RequireInstaller) {
        $InstallerHash = Verify-Sidecar $InstallerPath $InstallerSidecar
    }

    New-Item -ItemType Directory -Force -Path $TempRoot | Out-Null
    Expand-Archive -LiteralPath $ZipPath -DestinationPath $TempRoot -Force
    $PackageRoot = Join-Path $TempRoot $PackageName
    if (-not (Test-Path -LiteralPath $PackageRoot -PathType Container)) {
        throw "Portable ZIP did not contain expected root directory $PackageName."
    }

    foreach ($Name in $ExpectedExecutables) {
        if (-not (Test-Path -LiteralPath (Join-Path $PackageRoot $Name) -PathType Leaf)) {
            throw "Portable release missing expected executable: $Name"
        }
    }

    $PortableExecutables = @($ExpectedExecutables | ForEach-Object { Join-Path $PackageRoot $_ })
    if ($RequireSigning) {
        & (Join-Path $PSScriptRoot "verify-authenticode.ps1") -Path $PortableExecutables -RequireTimestamp
        if ($LASTEXITCODE -ne 0) { throw "Portable Authenticode verification failed." }
        if ($RequireInstaller) {
            & (Join-Path $PSScriptRoot "verify-authenticode.ps1") -Path @($InstallerPath) -RequireTimestamp
            if ($LASTEXITCODE -ne 0) { throw "Installer Authenticode verification failed." }
        }
    }

    $BuildInfoPath = Join-Path $PackageRoot "BUILD-INFO.txt"
    $BuildInfo = Get-Content -Raw -LiteralPath $BuildInfoPath
    foreach ($ExpectedText in @("Version: v$Version", "Git commit: $Commit", "Git tag: $Tag")) {
        if (-not $BuildInfo.Contains($ExpectedText)) { throw "BUILD-INFO.txt missing identity: $ExpectedText" }
    }

    $InternalManifest = Join-Path $PackageRoot "SHA256SUMS.txt"
    if (-not (Test-Path -LiteralPath $InternalManifest -PathType Leaf)) { throw "Portable SHA256SUMS.txt is missing." }
    foreach ($Line in Get-Content -LiteralPath $InternalManifest) {
        if ($Line -notmatch '^([A-Fa-f0-9]{64})  (.+)$') { throw "Malformed internal checksum line: $Line" }
        $ExpectedHash = $Matches[1].ToUpperInvariant()
        $Name = $Matches[2]
        $File = Join-Path $PackageRoot $Name
        if (-not (Test-Path -LiteralPath $File -PathType Leaf)) { throw "Internal manifest references missing file: $Name" }
        $ActualHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $File).Hash.ToUpperInvariant()
        if ($ExpectedHash -ne $ActualHash) { throw "Internal package checksum mismatch: $Name" }
    }

    $Signature = Get-AuthenticodeSignature -LiteralPath (Join-Path $PackageRoot $ExpectedExecutables[0])
    $SigningState = if ($RequireSigning) { "authenticode-sha256-rfc3161" } else { "unsigned-development" }
    $SignerSubject = if ($RequireSigning -and $Signature.SignerCertificate) { $Signature.SignerCertificate.Subject } else { $null }
    $SignerThumbprint = if ($RequireSigning -and $Signature.SignerCertificate) { $Signature.SignerCertificate.Thumbprint } else { $null }

    $Artifacts = @(
        [ordered]@{
            name = Split-Path -Leaf $ZipPath
            sha256 = $ZipHash
            bytes = (Get-Item -LiteralPath $ZipPath).Length
            kind = "portable-zip"
        }
    )
    if ($RequireInstaller) {
        $Artifacts += [ordered]@{
            name = Split-Path -Leaf $InstallerPath
            sha256 = $InstallerHash
            bytes = (Get-Item -LiteralPath $InstallerPath).Length
            kind = "windows-installer"
        }
    }

    $Manifest = [ordered]@{
        schema_version = 1
        tag = $Tag
        version = $Version
        commit = $Commit
        platform = "windows-x64"
        generated_utc = [DateTime]::UtcNow.ToString("o")
        expected_executables = $ExpectedExecutables
        artifacts = $Artifacts
        signing = [ordered]@{
            state = $SigningState
            required = [bool]$RequireSigning
            file_digest = "SHA256"
            timestamp_protocol = if ($RequireSigning) { "RFC3161" } else { $null }
            timestamp_digest = if ($RequireSigning) { "SHA256" } else { $null }
            signer_subject = $SignerSubject
            signer_thumbprint = $SignerThumbprint
        }
    }
    $Manifest | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $ManifestPath -Encoding UTF8
    $ManifestHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $ManifestPath).Hash
    "$ManifestHash  $(Split-Path -Leaf $ManifestPath)" | Set-Content -LiteralPath $ManifestSidecar -Encoding ASCII

    Write-Host "RELEASE ARTIFACT VERIFICATION: PASS"
    Write-Host "Tag: $Tag"
    Write-Host "Commit: $Commit"
    Write-Host "Manifest: $ManifestPath"
    Write-Host "Manifest SHA256: $ManifestHash"
}
finally {
    if (Test-Path -LiteralPath $TempRoot) { Remove-Item -LiteralPath $TempRoot -Recurse -Force -ErrorAction SilentlyContinue }
    Pop-Location
}
