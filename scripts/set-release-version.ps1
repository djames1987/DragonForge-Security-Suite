param(
    [Parameter(Mandatory = $true)] [string]$Version,
    [switch]$CheckOnly
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot

if ($Version -notmatch '^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$') {
    throw "Version must be SemVer-like, for example 0.1.0-alpha.3"
}

$NumericParts = ($Version -split '-')[0].Split('.')
$NumericVersion = "$($NumericParts[0]).$($NumericParts[1]).$($NumericParts[2]).0"

$TauriConfigs = @(
    "apps/security-center/tauri.conf.json",
    "apps/password-manager/tauri.conf.json",
    "apps/file-vault/tauri.conf.json",
    "apps/authenticator/tauri.conf.json",
    "apps/security-scanner/tauri.conf.json",
    "apps/integrity-monitor/tauri.conf.json",
    "apps/network-guard/tauri.conf.json",
    "apps/backup-recovery/tauri.conf.json",
    "apps/secure-share/tauri.conf.json"
)

function Assert-Or-SetText {
    param(
        [Parameter(Mandatory = $true)] [string]$Path,
        [Parameter(Mandatory = $true)] [string]$Pattern,
        [Parameter(Mandatory = $true)] [string]$Replacement,
        [Parameter(Mandatory = $true)] [string]$Expected
    )
    $Full = Join-Path $RepoRoot $Path
    $Text = Get-Content -Raw -LiteralPath $Full
    if ($CheckOnly) {
        if ($Text -notmatch $Expected) { throw "Version mismatch in $Path" }
        return
    }
    $Updated = [regex]::Replace($Text, $Pattern, $Replacement, 1)
    if ($Updated -eq $Text -and $Text -notmatch $Expected) { throw "Unable to stamp $Path" }
    Set-Content -LiteralPath $Full -Value $Updated -Encoding UTF8
}

Push-Location $RepoRoot
try {
    Assert-Or-SetText -Path "Cargo.toml" -Pattern '(?m)^version = "[^"]+"$' -Replacement ('version = "' + $Version + '"') -Expected ('(?m)^version = "' + [regex]::Escape($Version) + '"$')
    foreach ($Config in $TauriConfigs) {
        Assert-Or-SetText -Path $Config -Pattern '"version"\s*:\s*"[^"]+"' -Replacement ('"version": "' + $Version + '"') -Expected ('"version"\s*:\s*"' + [regex]::Escape($Version) + '"')
    }
    Assert-Or-SetText -Path "apps/security-center/ui/index.html" -Pattern 'Version [0-9A-Za-z.-]+' -Replacement ('Version ' + $Version) -Expected ('Version ' + [regex]::Escape($Version))
    Assert-Or-SetText -Path "installer/DragonForgeSecuritySuite.iss" -Pattern '(?m)^VersionInfoVersion=[0-9.]+$' -Replacement ('VersionInfoVersion=' + $NumericVersion) -Expected ('(?m)^VersionInfoVersion=' + [regex]::Escape($NumericVersion) + '$')
    Assert-Or-SetText -Path "installer/DragonForgeSecuritySuite.iss" -Pattern '(?m)^VersionInfoProductVersion=[0-9.]+$' -Replacement ('VersionInfoProductVersion=' + $NumericVersion) -Expected ('(?m)^VersionInfoProductVersion=' + [regex]::Escape($NumericVersion) + '$')

    if (-not $CheckOnly) {
        & cargo metadata --format-version 1 --no-deps *> $null
        if ($LASTEXITCODE -ne 0) { throw "Cargo metadata failed while refreshing Cargo.lock." }
    }
    if (-not (Test-Path -LiteralPath (Join-Path $RepoRoot "Cargo.lock") -PathType Leaf)) { throw "Cargo.lock is missing." }

    Write-Host "RELEASE VERSION STAMP: PASS"
    Write-Host "Version: $Version"
    Write-Host "Numeric version: $NumericVersion"
    Write-Host "Mode: $(if ($CheckOnly) { 'check-only' } else { 'updated' })"
}
finally {
    Pop-Location
}
