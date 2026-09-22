param(
    [Parameter(Mandatory = $true)] [string]$Version
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot

Push-Location $RepoRoot
try {
    $Dirty = (& git status --porcelain)
    if ($Dirty) { throw "Start release preparation from a clean working tree." }

    & (Join-Path $PSScriptRoot "set-release-version.ps1") -Version $Version
    if ($LASTEXITCODE -ne 0) { throw "Version stamping failed." }

    & cargo check --workspace
    if ($LASTEXITCODE -ne 0) { throw "Workspace check failed while refreshing Cargo.lock." }

    & (Join-Path $PSScriptRoot "set-release-version.ps1") -Version $Version -CheckOnly
    if ($LASTEXITCODE -ne 0) { throw "Stamped release version failed consistency check." }

    $Changed = @(& git status --short)
    Write-Host ""
    Write-Host "RELEASE PREPARATION: PASS"
    Write-Host "Version: $Version"
    Write-Host "Next:"
    Write-Host "  1. Review and commit the stamped files and Cargo.lock."
    Write-Host ("  2. Tag that exact commit: git tag -a v{0} -m ""DragonForge Security Suite v{0}""" -f $Version)
    Write-Host "  3. Push the commit and tag."
    Write-Host ("  4. Run build-tagged-windows-release.ps1 -Tag v{0}" -f $Version)
    Write-Host ""
    Write-Host "Changed files:"
    $Changed | ForEach-Object { Write-Host "  $_" }
}
finally { Pop-Location }
