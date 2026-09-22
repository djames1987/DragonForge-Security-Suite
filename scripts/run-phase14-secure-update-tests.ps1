$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$LogDirectory = Join-Path $RepoRoot "test-logs"
$Timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogPath = Join-Path $LogDirectory "dragonforge-phase14-secure-update-$Timestamp.log"
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
    }
    finally { $ErrorActionPreference = $Previous }
    if ($ExitCode -ne 0) { throw "Command failed with exit code $ExitCode - $Command $($Arguments -join ' ')" }
}

try {
    Start-Transcript -Path $LogPath -Force | Out-Null
    Write-Host "DragonForge Security Suite - Phase 14 Secure Update System verification"

    Invoke-Checked cargo "fmt" "--all" "--check"
    Invoke-Checked cargo "check" "--workspace" "--all-targets" "--all-features" "--locked"
    Invoke-Checked cargo "clippy" "--workspace" "--all-targets" "--all-features" "--locked" "--" "-D" "warnings"
    Invoke-Checked cargo "test" "--workspace" "--all-features" "--locked"
    Invoke-Checked cargo "metadata" "--locked" "--format-version" "1" "--no-deps"

    foreach ($Path in @(
        "apps/security-center/ui/app.js",
        "apps/password-manager/ui/app.js",
        "apps/file-vault/ui/app.js",
        "apps/authenticator/ui/app.js",
        "apps/security-scanner/ui/app.js",
        "apps/integrity-monitor/ui/app.js",
        "apps/network-guard/ui/app.js",
        "apps/backup-recovery/ui/app.js",
        "apps/secure-share/ui/app.js"
    )) {
        Invoke-Checked node "--check" $Path
    }

    foreach ($Required in @(
        "crates/dragonforge-update/src/lib.rs",
        "crates/dragonforge-update/src/bin/sign_update_manifest.rs",
        "apps/security-center/src/update.rs",
        "scripts/generate-signed-update-manifest.ps1",
        "docs/PHASE_14_SECURE_UPDATE.md",
        "docs/adr/0008-secure-update-trust-boundary.md"
    )) {
        if (-not (Test-Path -LiteralPath $Required -PathType Leaf)) { throw "Missing Phase 14 artifact: $Required" }
        Write-Host "OK  $Required"
    }

    $UpdateSource = Get-Content -Raw "crates/dragonforge-update/src/lib.rs"
    foreach ($Text in @("ML-DSA-65","update candidate would downgrade","windows-installer","authenticode_required","https://")) {
        if (-not $UpdateSource.Contains($Text)) { throw "Update engine invariant missing: $Text" }
    }
    $CenterSource = Get-Content -Raw "apps/security-center/src/update.rs"
    foreach ($Text in @("DRAGONFORGE_UPDATE_PUBLIC_KEY_HEX","Get-AuthenticodeSignature","sha256_hex","install_prepared","Command::new(&prepared.path)")) {
        if (-not $CenterSource.Contains($Text)) { throw "Security Center update invariant missing: $Text" }
    }
    $PublishSource = Get-Content -Raw "scripts/publish-windows-release.ps1"
    foreach ($Text in @("Phase 14 update-capable releases must be Authenticode-signed","generate-signed-update-manifest.ps1","DRAGONFORGE_UPDATE_SIGNING_KEY_HEX")) {
        if (-not $PublishSource.Contains($Text)) { throw "Release pipeline update invariant missing: $Text" }
    }

    foreach ($App in @("security-center","password-manager","file-vault","authenticator","security-scanner","integrity-monitor","network-guard","backup-recovery","secure-share")) {
        $Html = Get-Content -Raw "apps/$App/ui/index.html"
        if (-not $Html.Contains("Suite Phase 14")) { throw "$App has stale suite phase metadata." }
    }
    $Diagnostics = Get-Content -Raw "apps/security-center/src/diagnostics.rs"
    if (-not $Diagnostics.Contains('const PHASE: &str = "14";')) { throw "Security Center diagnostics do not report Phase 14." }

    foreach ($Script in @(
        "generate-signed-update-manifest.ps1",
        "publish-windows-release.ps1",
        "build-tagged-windows-release.ps1",
        "verify-release-artifacts.ps1",
        "run-phase14-secure-update-tests.ps1"
    )) {
        $Path = Join-Path $PSScriptRoot $Script
        $Tokens = $null
        $Errors = $null
        [void][System.Management.Automation.Language.Parser]::ParseFile($Path, [ref]$Tokens, [ref]$Errors)
        if ($Errors.Count -ne 0) { throw "PowerShell syntax validation failed for $Script : $($Errors[0].Message)" }
        Write-Host "POWERSHELL SYNTAX PASS  $Script"
    }

    Write-Host ""
    Write-Host "PHASE 14 SECURE UPDATE SYSTEM VERIFICATION: PASS"
}
catch {
    Write-Host ""
    Write-Host "PHASE 14 SECURE UPDATE SYSTEM VERIFICATION: FAIL"
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
