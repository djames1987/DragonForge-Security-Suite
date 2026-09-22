param(
    [Parameter(Mandatory = $true)] [string]$Version,
    [switch]$CheckOnly
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot

if ($Version -notmatch '^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$') {
    throw "Version must be SemVer-like, for example 0.1.0-alpha.3"
}

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
    param([string]$Path, [string]$Pattern, [string]$Replacement, [string]$Expected)
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
    Assert-Or-SetText "Cargo.toml" '(?m)^version = "[^"]+"$' ('version = "' + $Version + '"') ('(?m)^version = "' + [regex]::Escape($Version) + '"$')

    foreach ($Config in $TauriConfigs) {
        Assert-Or-SetText $Config '"version"\s*:\s*"[^"]+"' ('"version": "' + $Version + '"') ('"version"\s*:\s*"' + [regex]::Escape($Version) + '"')
    }

    Assert-Or-SetText "apps/security-center/ui/index.html" 'Version [0-9A-Za-z.-]+' ('Version ' + $Version) ('Version ' + [regex]::Escape($Version))

    if (-not $CheckOnly) {
        & cargo metadata --format-version 1 --no-deps *> $null
        if ($LASTEXITCODE -ne 0) { throw "Cargo metadata failed while refreshing Cargo.lock." }
    }

    if (-not (Test-Path -LiteralPath (Join-Path $RepoRoot "Cargo.lock") -PathType Leaf)) {
        throw "Cargo.lock is missing."
    }

    Write-Host "RELEASE VERSION STAMP: PASS"
    Write-Host "Version: $Version"
    Write-Host "Mode: $(if ($CheckOnly) { 'check-only' } else { 'updated' })"
}
finally {
    Pop-Location
}
