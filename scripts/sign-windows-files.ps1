param(
    [Parameter(Mandatory = $true)] [string[]]$Path,
    [string]$Description = "DragonForge Security Suite",
    [string]$TimestampUrl = $env:DRAGONFORGE_SIGN_TIMESTAMP_URL
)

$ErrorActionPreference = "Stop"

function Find-SignTool {
    $Command = Get-Command "signtool.exe" -ErrorAction SilentlyContinue
    if ($Command) { return $Command.Source }

    $Roots = @()
    $ProgramFilesX86 = [Environment]::GetEnvironmentVariable("ProgramFiles(x86)")
    if ($ProgramFilesX86) { $Roots += (Join-Path $ProgramFilesX86 "Windows Kits\10\bin") }
    if ($env:ProgramFiles) { $Roots += (Join-Path $env:ProgramFiles "Windows Kits\10\bin") }

    foreach ($Root in $Roots) {
        if (-not (Test-Path -LiteralPath $Root -PathType Container)) { continue }
        $Candidates = Get-ChildItem -LiteralPath $Root -Directory -ErrorAction SilentlyContinue |
            Sort-Object Name -Descending |
            ForEach-Object { Join-Path $_.FullName "x64\signtool.exe" }
        foreach ($Candidate in $Candidates) {
            if (Test-Path -LiteralPath $Candidate -PathType Leaf) { return $Candidate }
        }
    }

    throw "SignTool.exe was not found. Install a Windows SDK that includes SignTool."
}

if ($env:OS -ne "Windows_NT") { throw "Authenticode signing is supported only on Windows." }
if (-not $TimestampUrl) { throw "DRAGONFORGE_SIGN_TIMESTAMP_URL must be set to an RFC 3161 timestamp server URL." }

$Thumbprint = (($env:DRAGONFORGE_SIGN_CERT_THUMBPRINT + "") -replace "\s", "").ToUpperInvariant()
$PfxPath = $env:DRAGONFORGE_SIGN_PFX_PATH
$PfxPassword = $env:DRAGONFORGE_SIGN_PFX_PASSWORD

if ($Thumbprint -and $PfxPath) { throw "Configure either DRAGONFORGE_SIGN_CERT_THUMBPRINT or DRAGONFORGE_SIGN_PFX_PATH, not both." }
if (-not $Thumbprint -and -not $PfxPath) { throw "No signing identity configured. Set DRAGONFORGE_SIGN_CERT_THUMBPRINT or DRAGONFORGE_SIGN_PFX_PATH." }
if ($PfxPath -and -not (Test-Path -LiteralPath $PfxPath -PathType Leaf)) { throw "Configured PFX file does not exist." }

$SignTool = Find-SignTool

foreach ($Item in $Path) {
    $FullPath = [System.IO.Path]::GetFullPath($Item)
    if (-not (Test-Path -LiteralPath $FullPath -PathType Leaf)) { throw "Signing target does not exist: $FullPath" }

    $Arguments = @("sign", "/fd", "SHA256", "/td", "SHA256", "/tr", $TimestampUrl, "/d", $Description)
    if ($Thumbprint) {
        $Arguments += @("/sha1", $Thumbprint, "/s", "My")
        if ($env:DRAGONFORGE_SIGN_CERT_STORE_LOCATION -eq "LocalMachine") { $Arguments += "/sm" }
    }
    else {
        $Arguments += @("/f", $PfxPath)
        if ($PfxPassword) { $Arguments += @("/p", $PfxPassword) }
    }
    $Arguments += $FullPath

    $DisplayArguments = @($Arguments)
    for ($Index = 0; $Index -lt $DisplayArguments.Count; $Index++) {
        if ($DisplayArguments[$Index] -eq "/p" -and ($Index + 1) -lt $DisplayArguments.Count) { $DisplayArguments[$Index + 1] = "<redacted>" }
    }

    Write-Host ""
    Write-Host ">>> $SignTool $($DisplayArguments -join ' ')"
    & $SignTool @Arguments
    if ($LASTEXITCODE -ne 0) { throw "Authenticode signing failed for $FullPath with exit code $LASTEXITCODE." }

    & $SignTool verify /pa /all /tw $FullPath
    if ($LASTEXITCODE -ne 0) { throw "Authenticode verification failed for $FullPath with exit code $LASTEXITCODE." }
    Write-Host "SIGNED  $FullPath"
}

Write-Host ""
Write-Host "AUTHENTICODE SIGNING: PASS"
