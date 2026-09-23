param(
    [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot ".."))
)

$ErrorActionPreference = "Stop"
$Failed = $false
$LogDir = Join-Path $RepoRoot "test-logs"
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
$Stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogPath = Join-Path $LogDir "dragonforge-phase19-network-policy-firewall-$Stamp.log"

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
    Write-Host "DragonForge Security Suite - Phase 19 Network Policy & Firewall Integration verification"
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
    Run "node --check Network Guard UI" {
        node --check apps/network-guard/ui/app.js
    }

    foreach ($File in @(
        "services/dragonforge-privileged-service/src/firewall.rs",
        "crates/dragonforge-windows-boundary/src/lib.rs",
        "crates/dragonforge-agent/src/privileged.rs",
        "apps/network-guard/src/lib.rs",
        "apps/network-guard/ui/app.js",
        "docs/PHASE_19_NETWORK_POLICY_FIREWALL.md",
        "docs/adr/0012-network-policy-firewall-boundary.md"
    )) {
        if (-not (Test-Path -LiteralPath $File -PathType Leaf)) {
            throw "Missing Phase 19 artifact: $File"
        }
        Write-Host "OK  $File"
    }

    $Boundary = Get-Content -Raw "crates/dragonforge-windows-boundary/src/lib.rs"
    foreach ($Needle in @(
        "FirewallPolicyMutation",
        "phase19_allows",
        "FirewallApplicationIdentity",
        "FirewallMutationRequest",
        "firewall-status",
        "firewall-apply",
        "firewall-remove",
        "firewall-rollback"
    )) {
        if (-not $Boundary.Contains($Needle)) {
            throw "Missing Phase 19 boundary invariant: $Needle"
        }
    }
    foreach ($Forbidden in @(
        "ProtectedProcessControl => true",
        "ProtectedFileQuarantine => true",
        "ProtectedRegistryRemediation => true",
        "SystemIntegrityRemediation => true"
    )) {
        if ($Boundary.Contains($Forbidden)) {
            throw "Phase 19 enabled an unrelated privileged capability: $Forbidden"
        }
    }

    $Firewall = Get-Content -Raw "services/dragonforge-privileged-service/src/firewall.rs"
    foreach ($Needle in @(
        "INetFwPolicy2",
        "INetFwRule",
        "NET_FW_RULE_DIR_OUT",
        "NET_FW_PROFILE2_ALL",
        "FIREWALL_RULE_PREFIX",
        "FIREWALL_RULE_GROUP",
        "MAX_MANAGED_POLICIES",
        "MAX_ROLLBACKS",
        "verify_application_identity",
        "sha256_file",
        "json.bak",
        "restore_backend",
        "dragonforge-agent.exe",
        "dragonforge-privileged-service.exe",
        "Grouping()",
        "LocalPolicyModifyState",
        "NET_FW_MODIFY_STATE_OK",
        "require_local_modification",
        "SetApplicationName",
        "SetAction"
    )) {
        if (-not $Firewall.Contains($Needle)) {
            throw "Missing Phase 19 firewall invariant: $Needle"
        }
    }
    foreach ($Forbidden in @(
        "powershell.exe",
        "cmd.exe",
        "netsh.exe",
        "Command::new("
    )) {
        if ($Firewall.Contains($Forbidden)) {
            throw "Privileged firewall backend exposes a forbidden shell path: $Forbidden"
        }
    }

    $BoundaryLower = $Boundary.ToLowerInvariant()
    foreach ($ProtectedExe in @("dragonforge-agent.exe", "dragonforge-privileged-service.exe")) {
        if (-not $BoundaryLower.Contains($ProtectedExe)) {
            throw "Missing Phase 19 control-plane exclusion: $ProtectedExe"
        }
    }

    $ServiceHost = Get-Content -Raw "services/dragonforge-privileged-service/src/windows.rs"
    foreach ($Needle in @(
        "FirewallManager",
        "FirewallStatus",
        "FirewallApply",
        "FirewallRemove",
        "FirewallRollback",
        "authorize_caller",
        "abuse.authorize"
    )) {
        if (-not $ServiceHost.Contains($Needle)) {
            throw "Missing Phase 19 privileged-service routing invariant: $Needle"
        }
    }

    $Agent = Get-Content -Raw "crates/dragonforge-agent/src/privileged.rs"
    foreach ($Needle in @(
        "firewall_status",
        "firewall_apply",
        "firewall_remove",
        "firewall_rollback",
        "FirewallMutationRequest"
    )) {
        if (-not $Agent.Contains($Needle)) {
            throw "Missing Phase 19 Agent firewall invariant: $Needle"
        }
    }

    $NetworkGuard = Get-Content -Raw "apps/network-guard/src/lib.rs"
    foreach ($Needle in @(
        "inspect_process_application",
        "--firewall-status",
        "--firewall-allow",
        "--firewall-block",
        "--firewall-remove",
        "--firewall-rollback",
        "dragonforge-agent.exe"
    )) {
        if (-not $NetworkGuard.Contains($Needle)) {
            throw "Missing Phase 19 Network Guard routing invariant: $Needle"
        }
    }
    if ($NetworkGuard.Contains("PIPE_NAME")) {
        throw "Network Guard must not access the privileged named pipe directly."
    }

    foreach ($File in @(
        "scripts/run-phase19-network-policy-firewall-tests.ps1",
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
    Write-Host "PHASE 19 NETWORK POLICY & FIREWALL INTEGRATION VERIFICATION: PASS"
}
catch {
    $Failed = $true
    Write-Host ""
    Write-Host "PHASE 19 NETWORK POLICY & FIREWALL INTEGRATION VERIFICATION: FAIL"
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
