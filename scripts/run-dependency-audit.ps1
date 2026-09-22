[CmdletBinding()]
param(
    [switch]$InstallIfMissing
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
Push-Location $RepoRoot
try {
    $Audit = Get-Command cargo-audit -ErrorAction SilentlyContinue
    if (-not $Audit) {
        if (-not $InstallIfMissing) {
            throw "cargo-audit is not installed. Install with: cargo install cargo-audit --locked, or rerun this script with -InstallIfMissing."
        }
        & cargo install cargo-audit --locked
        if ($LASTEXITCODE -ne 0) { throw "cargo-audit installation failed." }
    }

    & cargo audit
    if ($LASTEXITCODE -ne 0) { throw "Dependency advisory audit failed." }
    Write-Host "DEPENDENCY SECURITY AUDIT: PASS"
}
finally {
    Pop-Location
}
