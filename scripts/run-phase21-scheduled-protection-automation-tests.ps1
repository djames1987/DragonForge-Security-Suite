param(
    [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot ".."))
)

$ErrorActionPreference = "Stop"
$Failed = $false
$LogDir = Join-Path $RepoRoot "test-logs"
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
$Stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogPath = Join-Path $LogDir "dragonforge-phase21-scheduled-protection-automation-$Stamp.log"

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
    Write-Host "DragonForge Security Suite - Phase 21 Scheduled Protection & Automation verification"
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

    foreach ($File in @(
        "crates/dragonforge-agent/src/automation.rs",
        "crates/dragonforge-agent/src/integrity.rs",
        "crates/dragonforge-agent/src/server.rs",
        "services/dragonforge-agent/src/main.rs",
        "apps/security-center/src/state.rs",
        "apps/security-center/src/settings.rs",
        "apps/security-center/ui/app.js",
        "apps/security-center/ui/index.html",
        "docs/PHASE_21_SCHEDULED_PROTECTION_AUTOMATION.md",
        "docs/adr/0014-capability-scoped-scheduled-protection.md"
    )) {
        if (-not (Test-Path -LiteralPath $File -PathType Leaf)) {
            throw "Missing Phase 21 artifact: $File"
        }
        Write-Host "OK  $File"
    }

    $Automation = Get-Content -Raw "crates/dragonforge-agent/src/automation.rs"
    foreach ($Needle in @(
        "AUTOMATION_STATE_VERSION",
        "MIN_INTERVAL_MINUTES",
        "MAX_INTERVAL_MINUTES",
        "MAX_AUTOMATION_EVENTS",
        "SecurityScan",
        "IntegrityCheck",
        "BackupReminder",
        "missed_recovered",
        "MAX_CONSECUTIVE_RETRIES",
        "quarantine_invalid",
        ".invalid",
        ".bak",
        "scheduled automation state is busy"
    )) {
        if (-not $Automation.Contains($Needle)) {
            throw "Missing Phase 21 automation invariant: $Needle"
        }
    }

    foreach ($Forbidden in @(
        "Command::new(",
        "std::process::Command",
        "command: String",
        "arguments: Vec",
        "script: String",
        "ProtectedProcessControl => true",
        "ProtectedFileQuarantine => true",
        "ProtectedRegistryRemediation => true",
        "SystemIntegrityRemediation => true"
    )) {
        if ($Automation.Contains($Forbidden)) {
            throw "Phase 21 introduced a forbidden generic/privileged scheduler surface: $Forbidden"
        }
    }

    $Server = Get-Content -Raw "crates/dragonforge-agent/src/server.rs"
    foreach ($Needle in @(
        "scheduled-protection-automation",
        "dragonforge-automation",
        "automation.tick()"
    )) {
        if (-not $Server.Contains($Needle)) {
            throw "Missing Phase 21 Agent scheduler invariant: $Needle"
        }
    }

    $Integrity = Get-Content -Raw "crates/dragonforge-integrity-monitor/src/continuous.rs"
    if (-not $Integrity.Contains("run_continuous_check_now")) {
        throw "Phase 21 explicit integrity check entry point is missing."
    }

    $State = Get-Content -Raw "apps/security-center/src/state.rs"
    foreach ($Needle in @(
        "automation_status",
        "configure_automation_job",
        "run_automation_job",
        "sync_automation_events",
        "automation_event_cursor"
    )) {
        if (-not $State.Contains($Needle)) {
            throw "Missing Phase 21 Security Center integration invariant: $Needle"
        }
    }

    $Ui = Get-Content -Raw "apps/security-center/ui/index.html"
    if (-not $Ui.Contains('data-view="automation"') -or -not $Ui.Contains("Suite Phase 21")) {
        throw "Security Center Phase 21 automation UI/milestone is missing."
    }

    foreach ($File in @(
        "scripts/run-phase21-scheduled-protection-automation-tests.ps1",
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
    Write-Host "PHASE 21 SCHEDULED PROTECTION & AUTOMATION VERIFICATION: PASS"
}
catch {
    $Failed = $true
    Write-Host ""
    Write-Host "PHASE 21 SCHEDULED PROTECTION & AUTOMATION VERIFICATION: FAIL"
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
