$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$LogDirectory = Join-Path $RepoRoot "test-logs"
$Timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogPath = Join-Path $LogDirectory "dragonforge-phase12.5-crash-diagnostics-$Timestamp.log"
$HashPath = "$LogPath.sha256"

New-Item -ItemType Directory -Force -Path $LogDirectory | Out-Null
Push-Location $RepoRoot

function Invoke-Checked {
    param(
        [Parameter(Mandatory = $true)] [string]$Command,
        [Parameter(ValueFromRemainingArguments = $true)] [string[]]$Arguments
    )
    Write-Host ""
    Write-Host ">>> $Command $($Arguments -join ' ')"
    $Previous = $ErrorActionPreference
    try {
        $ErrorActionPreference = "Continue"
        & $Command @Arguments 2>&1 | ForEach-Object { Write-Host $_ }
        $ExitCode = $LASTEXITCODE
    }
    finally { $ErrorActionPreference = $Previous }
    if ($ExitCode -ne 0) { throw "Command failed with exit code $ExitCode - $Command $($Arguments -join ' ')" }
}

try {
    Start-Transcript -Path $LogPath -Force | Out-Null
    Write-Host "DragonForge Security Suite - Phase 12.5 Crash Handling & Diagnostics verification"
    Write-Host "Repository: $RepoRoot"

    Invoke-Checked cargo "fmt" "--all" "--check"
    Invoke-Checked cargo "check" "-p" "dragonforge-core" "-p" "dragonforge-agent" "-p" "dragonforge-agent-service" "-p" "dragonforge-security-center" "--all-targets"
    Invoke-Checked cargo "clippy" "-p" "dragonforge-core" "-p" "dragonforge-agent" "-p" "dragonforge-agent-service" "-p" "dragonforge-security-center" "--all-targets" "--" "-D" "warnings"
    Invoke-Checked cargo "test" "-p" "dragonforge-core" "-p" "dragonforge-agent" "-p" "dragonforge-security-center" "--all-targets"
    Invoke-Checked node "--check" "apps/security-center/ui/app.js"

    foreach ($Path in @(
        "crates/dragonforge-core/src/diagnostics.rs",
        "apps/security-center/src/diagnostics.rs",
        "docs/PHASE_12_5_CRASH_DIAGNOSTICS.md"
    )) {
        if (-not (Test-Path -LiteralPath (Join-Path $RepoRoot $Path) -PathType Leaf)) { throw "Required Phase 12.5 artifact missing: $Path" }
        Write-Host "OK  $Path"
    }

    $Core = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "crates\dragonforge-core\src\diagnostics.rs")
    foreach ($Required in @("DEFAULT_MAX_BYTES","DEFAULT_BACKUPS","install_safe_panic_hook","last-failure.txt","password","token","vault","private key","authorization:","component_logger_rotates_bounded_files","failure_record_never_persists_sensitive_summary")) {
        if (-not $Core.Contains($Required)) { throw "Core diagnostics invariant missing: $Required" }
    }

    $Diagnostics = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "apps\security-center\src\diagnostics.rs")
    foreach ($Required in @('const PHASE: &str = "12.5"',"webview2_version","version: env!","dragonforge-support-bundle","component_logs","arbitrary-user-paths","sanitize_diagnostic_text")) {
        if (-not $Diagnostics.Contains($Required)) { throw "Security Center diagnostics invariant missing: $Required" }
    }

    $State = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "apps\security-center\src\state.rs")
    foreach ($Required in @("component_failures","Component::ALL","read_last_failure","create_support_bundle")) {
        if (-not $State.Contains($Required)) { throw "Dashboard diagnostics invariant missing: $Required" }
    }

    $Agent = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "services\dragonforge-agent\src\main.rs")
    if (-not $Agent.Contains("install_safe_panic_hook")) { throw "Agent safe panic hook is not installed." }

    $Ui = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "apps\security-center\ui\app.js")
    foreach ($Required in @("create_support_bundle","component_failures","last-failure")) {
        if (-not $Ui.Contains($Required)) { throw "Security Center diagnostics UI invariant missing: $Required" }
    }

    Write-Host ""
    Write-Host "PHASE 12.5 CRASH HANDLING & DIAGNOSTICS VERIFICATION: PASS"
}
catch {
    Write-Host ""
    Write-Host "PHASE 12.5 CRASH HANDLING & DIAGNOSTICS VERIFICATION: FAIL"
    Write-Host $_
    throw
}
finally {
    try { Stop-Transcript | Out-Null } catch {}
    Pop-Location
    if (Test-Path -LiteralPath $LogPath) {
        $Hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $LogPath).Hash
        "$Hash  $(Split-Path -Leaf $LogPath)" | Set-Content -LiteralPath $HashPath -Encoding ASCII
        Write-Host "Log: $LogPath"
        Write-Host "SHA256: $Hash"
    }
}
