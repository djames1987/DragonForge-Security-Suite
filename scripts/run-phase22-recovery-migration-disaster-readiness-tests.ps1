param(
    [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot ".."))
)

$ErrorActionPreference = "Stop"
$Failed = $false
$LogDir = Join-Path $RepoRoot "test-logs"
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
$Stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogPath = Join-Path $LogDir "dragonforge-phase22-recovery-migration-disaster-readiness-$Stamp.log"

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

try {
    Write-Host "DragonForge Security Suite - Phase 22 Recovery, Migration & Disaster Readiness verification"
    Write-Host "Windows host: $env:COMPUTERNAME"
    Write-Host "Repository: $RepoRoot"

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
    Run "node --check Security Center UI" {
        node --check apps/security-center/ui/app.js
    }
    Run "node --check Backup & Recovery UI" {
        node --check apps/backup-recovery/ui/app.js
    }

    foreach ($File in @(
        "crates/dragonforge-backup-recovery/src/recovery.rs",
        "crates/dragonforge-backup-recovery/src/format.rs",
        "apps/backup-recovery/src/lib.rs",
        "apps/backup-recovery/ui/app.js",
        "apps/backup-recovery/ui/index.html",
        "docs/PHASE_22_RECOVERY_MIGRATION_DISASTER_READINESS.md",
        "docs/adr/0015-encrypted-suite-recovery-clean-restore.md"
    )) {
        if (-not (Test-Path -LiteralPath $File -PathType Leaf)) {
            throw "Missing Phase 22 artifact: $File"
        }
        Write-Host "OK  $File"
    }

    $Recovery = Get-Content -Raw "crates/dragonforge-backup-recovery/src/recovery.rs"
    foreach ($Needle in @(
        "RecoveryScope",
        "RECOVERY_SCHEMA_VERSION",
        "create_suite_recovery",
        "restore_suite_recovery",
        "repair_recoverable_json_state",
        "legacy-unknown",
        "agent-session.key",
        "agent-runtime.json",
        "MAX_REPAIR_DEPTH",
        "MAX_REPAIR_BACKUPS",
        "recovery destination must be empty",
        "recovery package schema version is unsupported"
    )) {
        if (-not $Recovery.Contains($Needle)) {
            throw "Missing Phase 22 recovery invariant: $Needle"
        }
    }

    foreach ($Forbidden in @(
        "Command::new(",
        "std::process::Command",
        "powershell.exe",
        "cmd.exe",
        "overwrite existing",
        "remove_dir_all(data_root)"
    )) {
        if ($Recovery.Contains($Forbidden)) {
            throw "Phase 22 introduced a forbidden recovery execution/overwrite surface: $Forbidden"
        }
    }

    $Format = Get-Content -Raw "crates/dragonforge-backup-recovery/src/format.rs"
    foreach ($Needle in @("DFRECOVR", "RECOVERY_FORMAT_VERSION", "RECOVERY_SCHEMA_VERSION", "dfrecovery")) {
        if (-not $Format.Contains($Needle)) {
            throw "Missing Phase 22 recovery format invariant: $Needle"
        }
    }

    $BackupUi = Get-Content -Raw "apps/backup-recovery/ui/index.html"
    foreach ($Needle in @(
        'data-view="recovery"',
        "Create recovery package",
        "Clean-machine restore",
        "Repair recoverable state"
    )) {
        if (-not $BackupUi.Contains($Needle)) {
            throw "Missing Phase 22 Backup & Recovery UI invariant: $Needle"
        }
    }

    $CenterUi = Get-Content -Raw "apps/security-center/ui/index.html"
    if (-not $CenterUi.Contains("Suite Phase 22")) {
        throw "Security Center Phase 22 milestone metadata is missing."
    }

    foreach ($File in @(
        "scripts/run-phase22-recovery-migration-disaster-readiness-tests.ps1",
        "scripts/build-all-apps-for-testing.ps1"
    )) {
        $Tokens = $null
        $Errors = $null
        [System.Management.Automation.Language.Parser]::ParseFile(
            (Resolve-Path $File),
            [ref]$Tokens,
            [ref]$Errors
        ) | Out-Null
        if ($Errors.Count -gt 0) {
            $Errors | ForEach-Object { Write-Host $_.Message }
            throw "PowerShell syntax errors in $File"
        }
        Write-Host "POWERSHELL SYNTAX PASS  $File"
    }

    Run "build all Windows suite applications" {
        powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-all-apps-for-testing.ps1
    }

    Write-Host ""
    Write-Host "PHASE 22 RECOVERY, MIGRATION & DISASTER READINESS VERIFICATION: PASS"
}
catch {
    $Failed = $true
    Write-Host ""
    Write-Host "PHASE 22 RECOVERY, MIGRATION & DISASTER READINESS VERIFICATION: FAIL"
    Write-Host $_
}
finally {
    Stop-Transcript | Out-Null
    $Hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $LogPath).Hash
    "$Hash  $([IO.Path]::GetFileName($LogPath))" | Set-Content -Encoding ascii "$LogPath.sha256"
    Write-Host "Log: $LogPath"
    Write-Host "SHA256: $Hash"
}

if ($Failed) { exit 1 }
