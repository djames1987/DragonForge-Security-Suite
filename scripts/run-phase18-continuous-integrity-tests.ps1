param(
    [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot ".."))
)

$ErrorActionPreference = "Stop"
$Failed = $false
$LogDir = Join-Path $RepoRoot "test-logs"
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
$Stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogPath = Join-Path $LogDir "dragonforge-phase18-continuous-integrity-$Stamp.log"

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
    Write-Host "DragonForge Security Suite - Phase 18 Continuous Integrity Monitoring verification"
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
    Run "node --check Integrity Monitor UI" {
        node --check apps/integrity-monitor/ui/app.js
    }

    foreach ($File in @(
        "crates/dragonforge-integrity-monitor/src/continuous.rs",
        "crates/dragonforge-agent/src/integrity.rs",
        "apps/integrity-monitor/src/lib.rs",
        "apps/integrity-monitor/ui/app.js",
        "apps/security-center/src/state.rs",
        "docs/PHASE_18_CONTINUOUS_INTEGRITY_MONITORING.md",
        "docs/adr/0011-continuous-integrity-runtime.md"
    )) {
        if (-not (Test-Path -LiteralPath $File -PathType Leaf)) {
            throw "Missing Phase 18 artifact: $File"
        }
        Write-Host "OK  $File"
    }

    $Continuous = Get-Content -Raw "crates/dragonforge-integrity-monitor/src/continuous.rs"
    foreach ($Needle in @(
        "CONTINUOUS_STATE_VERSION",
        "MAX_CONTINUOUS_EVENTS",
        "MAX_SUPPRESSION_RULES",
        "baseline_sha256",
        "baseline_changed",
        "suppressed",
        "run_continuous_check_if_due",
        "continuous_events",
        "reseal_continuous_baseline"
    )) {
        if (-not $Continuous.Contains($Needle)) {
            throw "Missing Phase 18 continuous-monitor invariant: $Needle"
        }
    }

    $Agent = Get-Content -Raw "crates/dragonforge-agent/src/server.rs"
    foreach ($Needle in @(
        "continuous-integrity-monitoring",
        "next_integrity_poll",
        "self.integrity.tick()",
        "set_nonblocking(true)"
    )) {
        if (-not $Agent.Contains($Needle)) {
            throw "Missing Phase 18 Agent scheduler invariant: $Needle"
        }
    }

    $SecurityCenter = Get-Content -Raw "apps/security-center/src/state.rs"
    foreach ($Needle in @(
        "sync_integrity_events",
        "integrity_last_event_id",
        "integrity-monitor.change-detected",
        "EventKind::Security",
        "Severity::Warning"
    )) {
        if (-not $SecurityCenter.Contains($Needle)) {
            throw "Missing Phase 18 Security Center alert invariant: $Needle"
        }
    }

    $Privileged = Get-Content -Raw "services/dragonforge-privileged-service/src/windows.rs"
    foreach ($Forbidden in @(
        'Command::new(',
        '"powershell.exe"',
        '"cmd.exe"',
        'FirewallPolicyMutation => true',
        'ProtectedProcessControl => true',
        'ProtectedFileQuarantine => true',
        'ProtectedRegistryRemediation => true',
        'SystemIntegrityRemediation => true'
    )) {
        if ($Privileged.Contains($Forbidden)) {
            throw "Phase 18 expanded forbidden privileged execution surface: $Forbidden"
        }
    }

    foreach ($File in @(
        "scripts/run-phase18-continuous-integrity-tests.ps1",
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
    Write-Host "PHASE 18 CONTINUOUS INTEGRITY MONITORING VERIFICATION: PASS"
}
catch {
    $Failed = $true
    Write-Host ""
    Write-Host "PHASE 18 CONTINUOUS INTEGRITY MONITORING VERIFICATION: FAIL"
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
