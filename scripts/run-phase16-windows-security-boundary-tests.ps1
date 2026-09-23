$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot
$LogDir = Join-Path $RepoRoot "test-logs"
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
$Stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogPath = Join-Path $LogDir "dragonforge-phase16-windows-security-boundary-$Stamp.log"
$Failed = $false

Start-Transcript -Path $LogPath -Force | Out-Null
try {
    function Run([string]$Label, [scriptblock]$Command) {
        Write-Host ""
        Write-Host ">>> $Label"
        & $Command
        if ($LASTEXITCODE -ne 0) { throw "Command failed with exit code $LASTEXITCODE - $Label" }
    }

    Write-Host "DragonForge Security Suite - Phase 16 Windows Security Boundary Foundation verification"

    Run "cargo fmt --all --check" { cargo fmt --all --check }
    Run "cargo check --workspace --all-targets --all-features --locked" { cargo check --workspace --all-targets --all-features --locked }
    Run "cargo clippy --workspace --all-targets --all-features --locked -- -D warnings" { cargo clippy --workspace --all-targets --all-features --locked -- -D warnings }
    Run "cargo test --workspace --all-features --locked" { cargo test --workspace --all-features --locked }

    foreach ($File in @(
        "crates/dragonforge-windows-boundary/Cargo.toml",
        "crates/dragonforge-windows-boundary/src/lib.rs",
        "docs/PHASE_16_WINDOWS_SECURITY_BOUNDARY.md",
        "docs/adr/0009-windows-privileged-service-boundary.md"
    )) {
        if (-not (Test-Path -LiteralPath $File)) { throw "Missing Phase 16 artifact: $File" }
    }

    $Boundary = Get-Content -Raw "crates/dragonforge-windows-boundary/src/lib.rs"
    foreach ($Needle in @(
        "DragonForgePrivilegedService",
        "NT SERVICE\DragonForgePrivilegedService",
        "dragonforge-agent.exe",
        "authenticode_valid",
        "publisher_subject",
        "operating_system_peer_verified",
        "CommandDenied",
        "arbitrary_command_execution_prohibited",
        "generic_shell_execution_prohibited"
    )) {
        if (-not $Boundary.Contains($Needle)) { throw "Missing Phase 16 boundary invariant: $Needle" }
    }

    foreach ($Denied in @('"exec"', '"shell"', '"powershell"', '"cmd"', '"run"')) {
        if (-not $Boundary.Contains($Denied)) { throw "Missing denied generic command regression: $Denied" }
    }

    $Docs = Get-Content -Raw "docs/PHASE_16_WINDOWS_SECURITY_BOUNDARY.md"
    if (-not $Docs.Contains("Phase 16 authorizes no privileged capability") -and
        -not $Docs.Contains("authorizes none")) {
        throw "Phase 16 documentation must make the deny-by-default capability boundary explicit."
    }

    $Workflow = Get-Content -Raw ".github/workflows/ci.yml"
    if (-not $Workflow.Contains("dragonforge-windows-boundary")) { throw "CI does not cover the Phase 16 boundary crate." }
    if (-not $Workflow.Contains("run-phase16-windows-security-boundary-tests.ps1")) { throw "CI does not track the Phase 16 verifier." }

    $Tokens = $null
    $Errors = $null
    [System.Management.Automation.Language.Parser]::ParseFile(
        (Resolve-Path "scripts/run-phase16-windows-security-boundary-tests.ps1"),
        [ref]$Tokens,
        [ref]$Errors
    ) | Out-Null
    if ($Errors.Count -gt 0) { throw "PowerShell syntax errors in Phase 16 verifier." }

    Write-Host ""
    Write-Host "PHASE 16 WINDOWS SECURITY BOUNDARY FOUNDATION VERIFICATION: PASS"
}
catch {
    $Failed = $true
    Write-Host ""
    Write-Host "PHASE 16 WINDOWS SECURITY BOUNDARY FOUNDATION VERIFICATION: FAIL"
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
