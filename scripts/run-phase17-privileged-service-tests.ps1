$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

if ($env:OS -ne "Windows_NT") {
    throw "Phase 17 verification is Windows-only."
}

$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot
$LogDir = Join-Path $RepoRoot "test-logs"
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
$Stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogPath = Join-Path $LogDir "dragonforge-phase17-privileged-service-$Stamp.log"
$Failed = $false

Start-Transcript -Path $LogPath -Force | Out-Null
try {
    function Run([string]$Label, [scriptblock]$Command) {
        Write-Host ""
        Write-Host ">>> $Label"
        & $Command
        if ($LASTEXITCODE -ne 0) {
            throw "Command failed with exit code $LASTEXITCODE - $Label"
        }
    }

    Write-Host "DragonForge Security Suite - Phase 17 DragonForge Privileged Service verification"
    Write-Host "Windows host: $env:COMPUTERNAME"
    Write-Host "Repository: $RepoRoot"

    Run "cargo fmt --all --check" {
        cargo fmt --all --check
    }
    Run "cargo metadata --locked" {
        cargo metadata --locked --format-version 1 --no-deps | Out-Null
    }
    Run "cargo check --workspace --all-targets --all-features --locked" {
        cargo check --workspace --all-targets --all-features --locked
    }
    Run "cargo clippy --workspace --all-targets --all-features --locked -- -D warnings" {
        cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
    }
    Run "cargo test --workspace --all-features --locked" {
        cargo test --workspace --all-features --locked
    }

    foreach ($File in @(
        "services/dragonforge-privileged-service/Cargo.toml",
        "services/dragonforge-privileged-service/src/lib.rs",
        "services/dragonforge-privileged-service/src/main.rs",
        "services/dragonforge-privileged-service/src/windows.rs",
        "crates/dragonforge-windows-boundary/src/lib.rs",
        "scripts/install-privileged-service.ps1",
        "scripts/uninstall-privileged-service.ps1",
        "scripts/test-privileged-service.ps1",
        "installer/DragonForgeSecuritySuite.iss",
        "docs/PHASE_17_DRAGONFORGE_PRIVILEGED_SERVICE.md",
        "docs/adr/0010-privileged-service-runtime.md"
    )) {
        if (-not (Test-Path -LiteralPath $File -PathType Leaf)) {
            throw "Missing Phase 17 artifact: $File"
        }
        Write-Host "OK  $File"
    }

    $Boundary = Get-Content -Raw "crates/dragonforge-windows-boundary/src/lib.rs"
    foreach ($Needle in @(
        "phase17_allows",
        "false",
        '"health"',
        '"describe-policy"',
        "CommandDenied",
        "arbitrary_command_execution_prohibited",
        "generic_shell_execution_prohibited"
    )) {
        if (-not $Boundary.Contains($Needle)) {
            throw "Missing Phase 17 boundary invariant: $Needle"
        }
    }

    $WindowsHost = Get-Content -Raw "services/dragonforge-privileged-service/src/windows.rs"
    foreach ($Needle in @(
        "service_dispatcher::start",
        "service_control_handler::register",
        "GetNamedPipeClientProcessId",
        "QueryFullProcessImageNameW",
        "WinVerifyTrust",
        "CertNameToStrW",
        "ConvertStringSecurityDescriptorToSecurityDescriptorW",
        "reject_remote_clients(true)",
        "PIPE_DACL_SDDL",
        "BoundaryPolicy",
        "privileged_capabilities_enabled: false"
    )) {
        if (-not $WindowsHost.Contains($Needle)) {
            throw "Missing privileged-service Windows trust invariant: $Needle"
        }
    }

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
        if ($WindowsHost.Contains($Forbidden)) {
            throw "Forbidden Phase 17 privileged execution surface detected: $Forbidden"
        }
    }

    $ServiceLib = Get-Content -Raw "services/dragonforge-privileged-service/src/lib.rs"
    foreach ($Needle in @(
        "MAX_REPLAY_NONCES",
        "MAX_QUOTA_IDENTITIES",
        "max_requests_per_minute",
        "audit_max_bytes",
        "rotate_if_needed",
        "RequestRejected"
    )) {
        if (-not $ServiceLib.Contains($Needle)) {
            throw "Missing Phase 17 abuse/audit invariant: $Needle"
        }
    }

    $NonFfiRust = Get-ChildItem "services/dragonforge-privileged-service/src" -Filter "*.rs" |
        Where-Object { $_.Name -ne "windows.rs" }
    foreach ($RustFile in $NonFfiRust) {
        $Text = Get-Content -Raw -LiteralPath $RustFile.FullName
        if ($Text -match '\bunsafe\s*\{') {
            throw "Unsafe Rust escaped the reviewed Windows FFI module: $($RustFile.FullName)"
        }
    }

    $Install = Get-Content -Raw "scripts/install-privileged-service.ps1"
    foreach ($Needle in @(
        "Get-AuthenticodeSignature",
        "SignatureStatus]::Valid",
        "SignerCertificate.Subject",
        "NT SERVICE\",
        "icacls.exe",
        "sc.exe sidtype",
        "restricted",
        "sc.exe sdset"
    )) {
        if (-not $Install.Contains($Needle)) {
            throw "Missing privileged-service installation invariant: $Needle"
        }
    }

    $Iss = Get-Content -Raw "installer/DragonForgeSecuritySuite.iss"
    if (-not $Iss.Contains("PrivilegesRequired=lowest")) {
        throw "Normal DragonForge installer must remain non-elevated by default."
    }
    foreach ($Needle in @(
        "privilegedservice",
        "install-privileged-service.cmd",
        "uninstall-privileged-service.cmd"
    )) {
        if (-not $Iss.Contains($Needle)) {
            throw "Installer is missing optional Phase 17 lifecycle integration: $Needle"
        }
    }

    foreach ($File in @(
        "scripts/install-privileged-service.ps1",
        "scripts/uninstall-privileged-service.ps1",
        "scripts/test-privileged-service.ps1",
        "scripts/package-windows-release.ps1",
        "scripts/package-windows-installer.ps1",
        "scripts/build-all-apps-for-testing.ps1",
        "scripts/run-phase17-privileged-service-tests.ps1"
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

    Run "build all Windows suite applications including privileged service" {
        powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-all-apps-for-testing.ps1
    }

    $ExpectedServiceExe = Join-Path $RepoRoot "target\debug\dragonforge-privileged-service.exe"
    if (-not (Test-Path -LiteralPath $ExpectedServiceExe -PathType Leaf)) {
        throw "Privileged service executable was not produced: $ExpectedServiceExe"
    }

    Write-Host ""
    Write-Host "PHASE 17 DRAGONFORGE PRIVILEGED SERVICE VERIFICATION: PASS"
}
catch {
    $Failed = $true
    Write-Host ""
    Write-Host "PHASE 17 DRAGONFORGE PRIVILEGED SERVICE VERIFICATION: FAIL"
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
