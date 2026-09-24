param(
    [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot ".."))
)

$ErrorActionPreference = "Stop"
$Failed = $false
$LogDir = Join-Path $RepoRoot "test-logs"
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
$Stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogPath = Join-Path $LogDir "dragonforge-phase24-security-release-audit-$Stamp.log"
$LicenseInventory = Join-Path $LogDir "dragonforge-phase24-license-inventory-$Stamp.json"

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
        throw "Required Phase 24 artifact is missing: $Path"
    }
    $Text = Get-Content -Raw -LiteralPath $Path
    foreach ($Needle in $Needles) {
        if (-not $Text.Contains($Needle)) {
            throw "$Path is missing release-audit invariant: $Needle"
        }
    }
}

try {
    Write-Host "DragonForge Security Suite - Phase 24 1.0 Security & Release Audit"
    Write-Host "Windows host: $env:COMPUTERNAME"
    Write-Host "Repository: $RepoRoot"

    $Dirty = @(& git status --porcelain)
    if ($LASTEXITCODE -ne 0) { throw "Unable to inspect Git working tree." }
    if ($Dirty.Count -gt 0) {
        Write-Host "Dirty paths:"
        $Dirty | ForEach-Object { Write-Host "  $_" }
        throw "Phase 24 audit requires a clean working tree."
    }

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
    }

    Write-Host ""
    Write-Host ">>> Tracked-source secret material scan"
    $TextExtensions = @(
        ".rs",".toml",".json",".yml",".yaml",".ps1",".cmd",".md",".txt",
        ".html",".css",".js",".mjs",".iss",".sql"
    )
    $Tracked = @(& git ls-files)
    if ($LASTEXITCODE -ne 0) { throw "Unable to enumerate tracked files for secret scan." }
    foreach ($Relative in $Tracked) {
        $Extension = [IO.Path]::GetExtension($Relative).ToLowerInvariant()
        if ($Extension -notin $TextExtensions) { continue }
        $Full = Join-Path $RepoRoot $Relative
        if (-not (Test-Path -LiteralPath $Full -PathType Leaf)) { continue }
        $Text = Get-Content -Raw -LiteralPath $Full -ErrorAction SilentlyContinue
        if ($null -eq $Text) { continue }
        if ($Text -match '-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----') {
            throw "Tracked private-key material detected in $Relative"
        }
        if ($Text -match '(?im)^\s*DRAGONFORGE_UPDATE_SIGNING_KEY_HEX\s*=\s*["''][0-9A-Fa-f]{64,}["'']') {
            throw "Literal update signing key detected in tracked file $Relative"
        }
    }
    Write-Host "TRACKED SOURCE SECRET SCAN: PASS"

    foreach ($File in @(
        "release/PHASE24_FEATURE_FREEZE.md",
        "release/phase24-audit-gate.json",
        "docs/PHASE_24_1_0_SECURITY_RELEASE_AUDIT.md",
        "docs/PHASE_24_THREAT_MODEL_REVIEW.md",
        "docs/PHASE_24_RESIDUAL_RISK_REGISTER.md"
    )) {
        if (-not (Test-Path -LiteralPath $File -PathType Leaf)) {
            throw "Missing Phase 24 release-audit artifact: $File"
        }
        Write-Host "OK  $File"
    }

    $Gate = Get-Content -Raw "release/phase24-audit-gate.json" | ConvertFrom-Json
    if ($Gate.schema_version -ne 1 -or $Gate.phase -ne 24) {
        throw "Phase 24 machine-readable audit gate has an invalid schema or phase."
    }
    if ($Gate.binary_reproducibility_claim -ne $false -or
        $Gate.source_archive_reproducibility_required -ne $true) {
        throw "Phase 24 reproducibility claims do not match the reviewed release boundary."
    }

    Require-Text "release/PHASE24_FEATURE_FREEZE.md" @(
        "State: ACTIVE",
        "encrypted vault/container/package formats",
        "privileged-service protocol and capability set",
        "New features and broader privileged capabilities are deferred until after 1.0"
    )

    Require-Text "crates/dragonforge-file-vault/src/format.rs" @(
        "phase24_mutation_corpus_never_panics_header_parser",
        "header_rejects_unsupported_version_and_truncation"
    )
    Require-Text "crates/dragonforge-backup-recovery/src/container.rs" @(
        "truncated_and_unsupported_version_backups_are_rejected",
        "modified_ciphertext_is_rejected"
    )
    Require-Text "crates/dragonforge-secure-share/src/package.rs" @(
        "truncated_and_unsupported_version_packages_are_rejected",
        "wrong_password_and_tampering_are_rejected"
    )
    Require-Text "crates/dragonforge-update/src/lib.rs" @(
        "phase24_mutation_corpus_never_panics_update_verifier",
        "ML-DSA-65",
        "update candidate would downgrade the installed version",
        "update manifest channel does not match the selected channel",
        "Windows update installer must require Authenticode"
    )

    Require-Text "crates/dragonforge-windows-boundary/src/lib.rs" @(
        "operating_system_peer_verified",
        "authenticode_required: true",
        "publisher_pin_required: true",
        "arbitrary_command_execution_prohibited: true",
        "generic_shell_execution_prohibited: true",
        "CommandDenied"
    )
    Require-Text "services/dragonforge-privileged-service/src/lib.rs" @(
        "replay_and_rate_abuse_fail_closed",
        "malformed_and_stale_requests_are_rejected"
    )

    Require-Text "scripts/build-tagged-windows-release.ps1" @(
        "Stable releases must be Authenticode-signed",
        "Refusing to build a tagged release from a dirty working tree",
        'HEAD $Head is not the tagged release commit',
        "Cargo.lock must be tracked for release builds"
    )
    Require-Text "scripts/verify-release-artifacts.ps1" @(
        "SHA-256 mismatch for",
        "RequireTimestamp",
        "BUILD-INFO.txt missing identity",
        "Internal package checksum mismatch"
    )
    Require-Text "scripts/publish-windows-release.ps1" @(
        "Refusing to publish from a dirty working tree",
        "HEAD does not match $Tag",
        "already exists and will not be mutated",
        "Update manifest signing credentials are required",
        "secure update publication requires the Windows installer"
    )
    Require-Text "installer/DragonForgeSecuritySuite.iss" @(
        "PrivilegesRequired=lowest",
        "Install DragonForge Privileged Service (requires signed build and Administrator approval)",
        "InfoBeforeFile=PHASE23-PRIVACY.txt"
    )

    foreach ($Script in @(
        "scripts/run-phase24-security-release-audit.ps1",
        "scripts/run-license-audit.ps1",
        "scripts/verify-source-reproducibility.ps1",
        "scripts/run-dependency-audit.ps1",
        "scripts/build-tagged-windows-release.ps1",
        "scripts/verify-release-artifacts.ps1",
        "scripts/publish-windows-release.ps1",
        "scripts/package-windows-installer.ps1",
        "scripts/build-all-apps-for-testing.ps1"
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

    Run "build all Windows suite applications" {
        powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-all-apps-for-testing.ps1
    }

    Write-Host ""
    Write-Host "PHASE 24 1.0 SECURITY & RELEASE AUDIT VERIFICATION: PASS"
    Write-Host "License inventory: $LicenseInventory"
}
catch {
    $Failed = $true
    Write-Host ""
    Write-Host "PHASE 24 1.0 SECURITY & RELEASE AUDIT VERIFICATION: FAIL"
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
