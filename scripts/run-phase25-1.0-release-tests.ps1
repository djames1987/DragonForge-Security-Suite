param(
    [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot ".."))
)

$ErrorActionPreference = "Stop"
$Failed = $false
$LogDir = Join-Path $RepoRoot "test-logs"
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
$Stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogPath = Join-Path $LogDir "dragonforge-phase25-1.0-release-readiness-$Stamp.log"
$LicenseInventory = Join-Path $LogDir "dragonforge-phase25-license-inventory-$Stamp.json"

Set-Location $RepoRoot
Start-Transcript -Path $LogPath -Force | Out-Null

function Run([string]$Label, [scriptblock]$Command) {
    Write-Host ""
    Write-Host ">>> $Label"
    $Previous = $ErrorActionPreference
    try {
        $ErrorActionPreference = "Continue"
        & $Command 2>&1 | ForEach-Object { Write-Host $_ }
        $ExitCode = $LASTEXITCODE
    }
    finally {
        $ErrorActionPreference = $Previous
    }
    if ($ExitCode -ne 0) {
        throw "Command failed with exit code $ExitCode - $Label"
    }
}

function Require-Text([string]$Path, [string[]]$Needles) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Required Phase 25 artifact is missing: $Path"
    }
    $Text = Get-Content -Raw -LiteralPath $Path
    foreach ($Needle in $Needles) {
        if (-not $Text.Contains($Needle)) {
            throw "$Path is missing Phase 25 invariant: $Needle"
        }
    }
}

try {
    Write-Host "DragonForge Security Suite - Phase 25 1.0 Release Readiness"
    Write-Host "Windows host: $env:COMPUTERNAME"
    Write-Host "Repository: $RepoRoot"

    $Dirty = @(& git status --porcelain)
    if ($LASTEXITCODE -ne 0) { throw "Unable to inspect Git working tree." }
    if ($Dirty.Count -gt 0) {
        $Dirty | ForEach-Object { Write-Host "  $_" }
        throw "Phase 25 readiness requires a clean working tree."
    }

    & (Join-Path $PSScriptRoot "set-release-version.ps1") -Version "1.0.0" -CheckOnly
    if ($LASTEXITCODE -ne 0) { throw "DragonForge release version is not consistently stamped to 1.0.0." }

    Run "cargo fmt --all --check" { cargo fmt --all --check }
    Run "cargo metadata --locked" { cargo metadata --locked --format-version 1 --no-deps | Out-Null }
    Run "cargo check --workspace --all-targets --all-features --locked" {
        cargo check --workspace --all-targets --all-features --locked
    }
    Run "cargo clippy --workspace --all-targets --all-features --locked -- -D warnings" {
        cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
    }
    Run "cargo test --workspace --all-features --locked" {
        cargo test --workspace --all-features --locked
    }

    Write-Host ""
    Write-Host ">>> RustSec dependency advisory audit"
    & (Join-Path $PSScriptRoot "run-dependency-audit.ps1") -InstallIfMissing
    if ($LASTEXITCODE -ne 0) { throw "RustSec dependency advisory audit failed." }

    Write-Host ""
    Write-Host ">>> Locked dependency license audit"
    & (Join-Path $PSScriptRoot "run-license-audit.ps1") -OutputPath $LicenseInventory
    if ($LASTEXITCODE -ne 0) { throw "Dependency license audit failed." }

    Write-Host ""
    Write-Host ">>> Exact-commit source reproducibility"
    & (Join-Path $PSScriptRoot "verify-source-reproducibility.ps1")
    if ($LASTEXITCODE -ne 0) { throw "Source reproducibility audit failed." }

    $Apps = @(
        "security-center",
        "password-manager",
        "file-vault",
        "authenticator",
        "security-scanner",
        "integrity-monitor",
        "network-guard",
        "backup-recovery",
        "secure-share"
    )
    foreach ($App in $Apps) {
        Run "node --check $App app.js" { node --check "apps/$App/ui/app.js" }
        Run "node --check $App phase23.js" { node --check "apps/$App/ui/phase23.js" }

        $Html = Get-Content -Raw -LiteralPath "apps/$App/ui/index.html"
        if (-not $Html.Contains("DragonForge 1.0")) {
            throw "$App does not expose the DragonForge 1.0 release marker."
        }
    }

    foreach ($File in @(
        "docs/PHASE_25_DRAGONFORGE_1_0.md",
        "docs/1_0_COMPATIBILITY_POLICY.md",
        "docs/1_0_MIGRATION_GUIDE.md",
        "docs/SUPPORT_POLICY.md",
        "docs/VULNERABILITY_RESPONSE.md",
        "release/V1_0_0_RELEASE_CHECKLIST.md",
        "release/phase25-release-gate.json"
    )) {
        if (-not (Test-Path -LiteralPath $File -PathType Leaf)) {
            throw "Missing Phase 25 artifact: $File"
        }
        Write-Host "OK  $File"
    }

    $Phase24 = Get-Content -Raw -LiteralPath "release/phase24-audit-gate.json" | ConvertFrom-Json
    if ($Phase24.status -ne "verified_complete" -or $Phase24.release_decision -ne "qualified_to_enter_phase25") {
        throw "Phase 24 machine-readable gate is not finalized as verified."
    }

    $Gate = Get-Content -Raw -LiteralPath "release/phase25-release-gate.json" | ConvertFrom-Json
    if ($Gate.schema_version -ne 1 -or $Gate.phase -ne 25 -or $Gate.version -ne "1.0.0" -or $Gate.tag -ne "v1.0.0") {
        throw "Phase 25 machine-readable release gate is invalid."
    }
    if ($Gate.stable_release_requires_signing -ne $true) {
        throw "Phase 25 release gate must require stable signing."
    }

    Require-Text "docs/1_0_COMPATIBILITY_POLICY.md" @(
        "File Vault",
        "format 1",
        "Agent",
        "1.1",
        "Privileged service",
        "Secure update manifest",
        "schema 1",
        "Password Manager sync API",
        "Browser/native messaging"
    )
    Require-Text "SECURITY.md" @(
        "DragonForge Security Suite 1.0",
        "docs/SUPPORT_POLICY.md",
        "docs/VULNERABILITY_RESPONSE.md"
    )
    Require-Text "apps/security-center/src/diagnostics.rs" @(
        'const PHASE: &str = "25";',
        'const RELEASE_CHANNEL: &str = "stable";',
        'env!("CARGO_PKG_VERSION")'
    )
    Require-Text "scripts/build-tagged-windows-release.ps1" @(
        "Stable releases must be Authenticode-signed",
        'HEAD $Head is not the tagged release commit'
    )
    Require-Text "scripts/publish-windows-release.ps1" @(
        "Phase 14 update-capable releases must be Authenticode-signed",
        "Update manifest signing credentials are required",
        "already exists and will not be mutated"
    )
    Require-Text "scripts/publish-dragonforge-1.0.ps1" @(
        'Tag = "v1.0.0"',
        "DRAGONFORGE SECURITY SUITE 1.0 RELEASE: PASS",
        "publish-windows-release.ps1"
    )

    Write-Host ""
    Write-Host ">>> Stable release fail-closed checks"
    $Previous = $ErrorActionPreference
    try {
        $ErrorActionPreference = "Continue"
        $BuildOutput = & powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-tagged-windows-release.ps1 -Tag v1.0.0 2>&1 | Out-String
        $BuildExit = $LASTEXITCODE
        $PublishOutput = & powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\publish-windows-release.ps1 -Tag v1.0.0 2>&1 | Out-String
        $PublishExit = $LASTEXITCODE
    }
    finally {
        $ErrorActionPreference = $Previous
    }
    if ($BuildExit -eq 0 -or -not $BuildOutput.Contains("Stable releases must be Authenticode-signed")) {
        throw "Stable tagged build did not fail closed without -SignRelease."
    }
    if ($PublishExit -eq 0 -or -not $PublishOutput.Contains("Phase 14 update-capable releases must be Authenticode-signed")) {
        throw "Stable publication did not fail closed without -SignRelease."
    }
    Write-Host "STABLE SIGNING FAIL-CLOSED CHECK: PASS"

    foreach ($Script in @(
        "scripts/run-phase25-1.0-release-tests.ps1",
        "scripts/publish-dragonforge-1.0.ps1",
        "scripts/build-tagged-windows-release.ps1",
        "scripts/publish-windows-release.ps1",
        "scripts/verify-release-artifacts.ps1",
        "scripts/generate-signed-update-manifest.ps1",
        "scripts/sign-windows-files.ps1",
        "scripts/verify-authenticode.ps1",
        "scripts/run-license-audit.ps1",
        "scripts/verify-source-reproducibility.ps1"
    )) {
        $Tokens = $null
        $Errors = $null
        [System.Management.Automation.Language.Parser]::ParseFile(
            (Resolve-Path $Script),
            [ref]$Tokens,
            [ref]$Errors
        ) | Out-Null
        if ($Errors.Count -gt 0) {
            $Errors | ForEach-Object { Write-Host $_.Message }
            throw "PowerShell syntax errors in $Script"
        }
        Write-Host "POWERSHELL SYNTAX PASS  $Script"
    }

    Run "build all Windows suite applications in release profile" {
        powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-all-apps-for-testing.ps1 -Profile release
    }

    Write-Host ""
    Write-Host "PHASE 25 DRAGONFORGE SECURITY SUITE 1.0 RELEASE READINESS: PASS"
    Write-Host "License inventory: $LicenseInventory"
}
catch {
    $Failed = $true
    Write-Host ""
    Write-Host "PHASE 25 DRAGONFORGE SECURITY SUITE 1.0 RELEASE READINESS: FAIL"
    Write-Host $_
}
finally {
    Stop-Transcript | Out-Null
    if (Test-Path -LiteralPath $LogPath) {
        $Hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $LogPath).Hash
        "$Hash  $([IO.Path]::GetFileName($LogPath))" | Set-Content -Encoding ascii "$LogPath.sha256"
        Write-Host "Log: $LogPath"
        Write-Host "SHA256: $Hash"
    }
}

if ($Failed) { exit 1 }
