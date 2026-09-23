param(
    [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot ".."))
)

$ErrorActionPreference = "Stop"
$Failed = $false
$LogDir = Join-Path $RepoRoot "test-logs"
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
$Stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogPath = Join-Path $LogDir "dragonforge-phase20-security-center-policy-event-hub-$Stamp.log"

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
    Write-Host "DragonForge Security Suite - Phase 20 Security Center Policy & Event Hub verification"
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
        "apps/security-center/src/events.rs",
        "apps/security-center/src/health_history.rs",
        "apps/security-center/src/settings.rs",
        "apps/security-center/src/state.rs",
        "apps/security-center/ui/app.js",
        "apps/security-center/ui/index.html",
        "docs/PHASE_20_SECURITY_CENTER_POLICY_EVENT_HUB.md",
        "docs/adr/0013-security-center-policy-event-hub.md"
    )) {
        if (-not (Test-Path -LiteralPath $File -PathType Leaf)) {
            throw "Missing Phase 20 artifact: $File"
        }
        Write-Host "OK  $File"
    }

    $Events = Get-Content -Raw "apps/security-center/src/events.rs"
    foreach ($Needle in @(
        "EVENT_HUB_VERSION",
        "MAX_EVENT_CAPACITY",
        "MAX_EVENT_HUB_BYTES",
        "acknowledged_at_ms",
        "acknowledge_all",
        "notifications",
        "quarantine_invalid",
        ".invalid",
        ".bak"
    )) {
        if (-not $Events.Contains($Needle)) {
            throw "Missing Phase 20 event-hub invariant: $Needle"
        }
    }

    $Health = Get-Content -Raw "apps/security-center/src/health_history.rs"
    foreach ($Needle in @(
        "HEALTH_HISTORY_VERSION",
        "MAX_HEALTH_HISTORY",
        "HealthHistoryEntry",
        "equivalent",
        "quarantine_invalid",
        ".invalid",
        ".bak"
    )) {
        if (-not $Health.Contains($Needle)) {
            throw "Missing Phase 20 health-history invariant: $Needle"
        }
    }

    $Settings = Get-Content -Raw "apps/security-center/src/settings.rs"
    foreach ($Needle in @(
        "SuitePolicy",
        "notification_min_severity",
        "health_history_limit",
        "10..=500",
        "50..=2_000"
    )) {
        if (-not $Settings.Contains($Needle)) {
            throw "Missing Phase 20 suite-policy invariant: $Needle"
        }
    }

    $State = Get-Content -Raw "apps/security-center/src/state.rs"
    foreach ($Needle in @(
        "record_event",
        "acknowledge_event",
        "acknowledge_all_notifications",
        "health_history",
        "sync_integrity_events",
        "integrity_alert_cursor"
    )) {
        if (-not $State.Contains($Needle)) {
            throw "Missing Phase 20 state integration invariant: $Needle"
        }
    }

    foreach ($Forbidden in @(
        "Command::new(",
        "powershell.exe",
        "cmd.exe",
        "netsh.exe",
        "ProtectedProcessControl => true",
        "ProtectedFileQuarantine => true",
        "ProtectedRegistryRemediation => true",
        "SystemIntegrityRemediation => true"
    )) {
        if ($Events.Contains($Forbidden) -or $Health.Contains($Forbidden) -or $Settings.Contains($Forbidden)) {
            throw "Phase 20 introduced forbidden generic/privileged execution surface: $Forbidden"
        }
    }

    $Ui = Get-Content -Raw "apps/security-center/ui/app.js"
    foreach ($Needle in @(
        "acknowledge_event",
        "acknowledge_all_notifications",
        "health_history",
        "setting-notification-severity",
        "setting-health-history"
    )) {
        if (-not $Ui.Contains($Needle)) {
            throw "Missing Phase 20 UI invariant: $Needle"
        }
    }

    foreach ($File in @(
        "scripts/run-phase20-security-center-policy-event-hub-tests.ps1",
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
    Write-Host "PHASE 20 SECURITY CENTER POLICY & EVENT HUB VERIFICATION: PASS"
}
catch {
    $Failed = $true
    Write-Host ""
    Write-Host "PHASE 20 SECURITY CENTER POLICY & EVENT HUB VERIFICATION: FAIL"
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
