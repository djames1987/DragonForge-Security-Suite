$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$LogDirectory = Join-Path $RepoRoot "test-logs"
$Timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogPath = Join-Path $LogDirectory "dragonforge-phase12.7-security-hardening-$Timestamp.log"
$HashPath = "$LogPath.sha256"

New-Item -ItemType Directory -Force -Path $LogDirectory | Out-Null
Push-Location $RepoRoot

function Invoke-Checked {
    param([Parameter(Mandatory = $true)] [string]$Command, [Parameter(ValueFromRemainingArguments = $true)] [string[]]$Arguments)
    Write-Host ""
    Write-Host ">>> $Command $($Arguments -join ' ')"
    $Previous = $ErrorActionPreference
    try {
        $ErrorActionPreference = "Continue"
        & $Command @Arguments 2>&1 | ForEach-Object { Write-Host $_ }
        $ExitCode = $LASTEXITCODE
    } finally { $ErrorActionPreference = $Previous }
    if ($ExitCode -ne 0) { throw "Command failed with exit code $ExitCode - $Command $($Arguments -join ' ')" }
}

try {
    Start-Transcript -Path $LogPath -Force | Out-Null
    Write-Host "DragonForge Security Suite - Phase 12.7 Security Hardening Review verification"
    Write-Host "Repository: $RepoRoot"

    Invoke-Checked cargo "fmt" "--all" "--check"
    Invoke-Checked cargo "check" "-p" "dragonforge-core" "-p" "dragonforge-agent" "-p" "dragonforge-file-vault" "-p" "dragonforge-backup-recovery" "-p" "dragonforge-secure-share" "-p" "dragonforge-security-center" "--all-targets"
    Invoke-Checked cargo "clippy" "-p" "dragonforge-core" "-p" "dragonforge-agent" "-p" "dragonforge-file-vault" "-p" "dragonforge-backup-recovery" "-p" "dragonforge-secure-share" "-p" "dragonforge-security-center" "--all-targets" "--" "-D" "warnings"
    Invoke-Checked cargo "test" "-p" "dragonforge-core" "-p" "dragonforge-agent" "-p" "dragonforge-file-vault" "-p" "dragonforge-backup-recovery" "-p" "dragonforge-secure-share" "-p" "dragonforge-security-center" "--all-targets"
    Invoke-Checked cargo "metadata" "--locked" "--format-version" "1" "--no-deps"

    Write-Host ""
    Write-Host ">>> RustSec dependency advisory audit"
    & (Join-Path $PSScriptRoot "run-dependency-audit.ps1") -InstallIfMissing
    if ($LASTEXITCODE -ne 0) { throw "Dependency security audit failed." }

    foreach ($Script in @("run-dependency-audit.ps1","review-windows-data-permissions.ps1","run-phase12.7-security-hardening-tests.ps1")) {
        $Path = Join-Path $PSScriptRoot $Script
        $Tokens = $null
        $Errors = $null
        [void][System.Management.Automation.Language.Parser]::ParseFile($Path, [ref]$Tokens, [ref]$Errors)
        if ($Errors.Count -ne 0) { throw "PowerShell syntax validation failed for $Script : $($Errors[0].Message)" }
        Write-Host "POWERSHELL SYNTAX PASS  $Script"
    }

    $CoreDiagnostics = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "crates\dragonforge-core\src\diagnostics.rs")
    foreach ($Needle in @("diagnostic_sanitizer_redacts_secret_markers","failure_record_never_persists_sensitive_summary")) {
        if (-not $CoreDiagnostics.Contains($Needle)) { throw "Diagnostics redaction regression test missing: $Needle" }
    }

    $Agent = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "crates\dragonforge-agent\src\server.rs")
    if (-not $Agent.Contains("active_runtime_lock_refuses_second_owner")) { throw "Agent contention regression test missing." }

    $Vault = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "crates\dragonforge-file-vault\src\container.rs")
    foreach ($Needle in @("decoded_entries_reject_duplicate_and_malformed_paths","decoded_entries_reject_truncated_payload")) {
        if (-not $Vault.Contains($Needle)) { throw "File Vault hardening test missing: $Needle" }
    }
    $VaultFormat = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "crates\dragonforge-file-vault\src\format.rs")
    if (-not $VaultFormat.Contains("header_rejects_unsupported_version_and_truncation")) { throw "File Vault header hardening test missing." }

    $Backup = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "crates\dragonforge-backup-recovery\src\container.rs")
    if (-not $Backup.Contains("truncated_and_unsupported_version_backups_are_rejected")) { throw "Backup parser hardening test missing." }
    $Share = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "crates\dragonforge-secure-share\src\package.rs")
    if (-not $Share.Contains("truncated_and_unsupported_version_packages_are_rejected")) { throw "Secure Share parser hardening test missing." }

    $Workflow = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot ".github\workflows\security-audit.yml")
    if (-not $Workflow.Contains("actions-rust-lang/audit@v1")) { throw "Scheduled RustSec workflow missing." }
    $Security = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "SECURITY.md")
    if (-not $Security.Contains("Local Agent IPC is authenticated and fail-closed")) { throw "Security policy IPC hardening statement missing." }
    $Architecture = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "docs\ARCHITECTURE.md")
    if (-not $Architecture.Contains("Windows currently relies on inherited per-user profile ACLs")) { throw "Windows ACL residual risk is not documented." }
    $Doc = Join-Path $RepoRoot "docs\PHASE_12_7_SECURITY_HARDENING.md"
    if (-not (Test-Path -LiteralPath $Doc -PathType Leaf)) { throw "Phase 12.7 hardening documentation is missing." }

    Write-Host ""
    Write-Host "PHASE 12.7 SECURITY HARDENING REVIEW VERIFICATION: PASS"
}
catch {
    Write-Host ""
    Write-Host "PHASE 12.7 SECURITY HARDENING REVIEW VERIFICATION: FAIL"
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
