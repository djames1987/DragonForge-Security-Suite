$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$LogDirectory = Join-Path $RepoRoot "test-logs"
$Timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogPath = Join-Path $LogDirectory "dragonforge-phase13-beta-readiness-$Timestamp.log"
$HashPath = "$LogPath.sha256"
$QualificationVersion = "0.1.0-beta-qualification"
$QualificationPackage = "DragonForge-Security-Suite-v$QualificationVersion-win-x64"
$DistRoot = Join-Path $RepoRoot "dist"
$QualificationArtifacts = @(
    (Join-Path $DistRoot "$QualificationPackage.zip"),
    (Join-Path $DistRoot "$QualificationPackage.zip.sha256"),
    (Join-Path $DistRoot "$QualificationPackage-setup.exe"),
    (Join-Path $DistRoot "$QualificationPackage-setup.exe.sha256"),
    (Join-Path $DistRoot $QualificationPackage),
    (Join-Path $DistRoot "$QualificationPackage-installer-stage")
)

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

function Verify-Sidecar {
    param([Parameter(Mandatory = $true)] [string]$Artifact)
    $Sidecar = "$Artifact.sha256"
    if (-not (Test-Path -LiteralPath $Artifact -PathType Leaf)) { throw "Qualification artifact missing: $Artifact" }
    if (-not (Test-Path -LiteralPath $Sidecar -PathType Leaf)) { throw "Qualification sidecar missing: $Sidecar" }
    $Expected = ((Get-Content -Raw -LiteralPath $Sidecar).Trim() -split "\s+")[0].ToUpperInvariant()
    $Actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $Artifact).Hash.ToUpperInvariant()
    if ($Expected -ne $Actual) { throw "Qualification artifact checksum mismatch: $Artifact" }
    Write-Host "SHA256 PASS  $(Split-Path -Leaf $Artifact)  $Actual"
}

try {
    Start-Transcript -Path $LogPath -Force | Out-Null
    Write-Host "DragonForge Security Suite - Phase 13 Beta Readiness & Release Qualification verification"
    Write-Host "Repository: $RepoRoot"

    Invoke-Checked cargo "fmt" "--all" "--check"
    Invoke-Checked cargo "check" "--workspace" "--all-targets" "--all-features"
    Invoke-Checked cargo "clippy" "--workspace" "--all-targets" "--all-features" "--" "-D" "warnings"
    Invoke-Checked cargo "test" "--workspace" "--all-features"
    Invoke-Checked cargo "metadata" "--locked" "--format-version" "1" "--no-deps"

    $JavaScript = @(
        "apps/security-center/ui/app.js",
        "apps/password-manager/ui/app.js",
        "apps/file-vault/ui/app.js",
        "apps/authenticator/ui/app.js",
        "apps/security-scanner/ui/app.js",
        "apps/integrity-monitor/ui/app.js",
        "apps/network-guard/ui/app.js",
        "apps/backup-recovery/ui/app.js",
        "apps/secure-share/ui/app.js",
        "extensions/password-manager-browser/src/core.js",
        "extensions/password-manager-browser/src/background.js",
        "extensions/password-manager-browser/ui/popup.js"
    )
    foreach ($Path in $JavaScript) { Invoke-Checked node "--check" $Path }
    Invoke-Checked node "--test" "extensions/password-manager-browser/tests/core.test.mjs" "extensions/password-manager-browser/tests/manifest.test.mjs"

    Write-Host ""
    Write-Host ">>> RustSec dependency advisory audit"
    & (Join-Path $PSScriptRoot "run-dependency-audit.ps1") -InstallIfMissing
    if ($LASTEXITCODE -ne 0) { throw "Dependency security audit failed." }

    & (Join-Path $PSScriptRoot "build-all-apps-for-testing.ps1") -Profile release
    if ($LASTEXITCODE -ne 0) { throw "Release-profile suite build failed." }

    & (Join-Path $PSScriptRoot "package-windows-release.ps1") -Version $QualificationVersion -SkipBuild
    if ($LASTEXITCODE -ne 0) { throw "Qualification portable package failed." }
    $ZipPath = Join-Path $DistRoot "$QualificationPackage.zip"
    Verify-Sidecar -Artifact $ZipPath

    $ExpandRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("dragonforge-phase13-" + [guid]::NewGuid().ToString("N"))
    try {
        Expand-Archive -LiteralPath $ZipPath -DestinationPath $ExpandRoot -Force
        $PackageRoot = Join-Path $ExpandRoot $QualificationPackage
        $Verifier = Join-Path $PackageRoot "Verify-Package.ps1"
        if (-not (Test-Path -LiteralPath $Verifier -PathType Leaf)) { throw "Portable internal verifier missing." }
        & powershell.exe -NoProfile -ExecutionPolicy Bypass -File $Verifier
        if ($LASTEXITCODE -ne 0) { throw "Portable internal package verification failed." }
    }
    finally {
        if ($ExpandRoot -and (Test-Path -LiteralPath $ExpandRoot)) {
            Remove-Item -LiteralPath $ExpandRoot -Recurse -Force -ErrorAction SilentlyContinue
        }
    }

    & (Join-Path $PSScriptRoot "package-windows-installer.ps1") -Version $QualificationVersion -SkipBuild
    if ($LASTEXITCODE -ne 0) { throw "Qualification installer package failed." }
    $InstallerPath = Join-Path $DistRoot "$QualificationPackage-setup.exe"
    Verify-Sidecar -Artifact $InstallerPath

    foreach ($Required in @(
        "docs/PHASE_13_BETA_READINESS.md",
        "docs/BETA_QUALIFICATION_MATRIX.md",
        "docs/BETA_RELEASE_GATE.md",
        "docs/EXTERNAL_TEST_CHECKLIST.md",
        "docs/EXTERNAL_TEST_MATRIX.md",
        "docs/INSTALLER_TEST_CHECKLIST.md",
        "scripts/new-beta-qualification-record.ps1",
        "scripts/evaluate-beta-qualification.ps1",
        "scripts/run-phase13-beta-readiness-tests.ps1",
        "scripts/package-windows-release.ps1",
        "scripts/package-windows-installer.ps1",
        "scripts/verify-release-artifacts.ps1"
    )) {
        if (-not (Test-Path -LiteralPath (Join-Path $RepoRoot $Required) -PathType Leaf)) {
            throw "Required Phase 13 artifact is missing: $Required"
        }
        Write-Host "OK  $Required"
    }

    foreach ($App in @("security-center","password-manager","file-vault","authenticator","security-scanner","integrity-monitor","network-guard","backup-recovery","secure-share")) {
        $Html = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "apps\$App\ui\index.html")
        if (-not $Html.Contains("Suite Phase 13")) { throw "$App does not expose current Suite Phase 13 metadata." }
    }
    $DiagnosticsSource = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "apps\security-center\src\diagnostics.rs")
    if (-not $DiagnosticsSource.Contains('const PHASE: &str = "13";')) { throw "Security Center diagnostics do not report Phase 13." }

    $Matrix = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "docs\BETA_QUALIFICATION_MATRIX.md")
    foreach ($Id in @("BQ-01","BQ-02","BQ-03","BQ-04","BQ-05","BQ-06")) {
        if (-not $Matrix.Contains($Id)) { throw "Beta qualification scenario missing: $Id" }
    }
    foreach ($RequiredText in @("Windows 10 22H2","Windows 11","Physical","VM","Standard user","WebView2")) {
        if (-not $Matrix.Contains($RequiredText)) { throw "Beta matrix coverage missing: $RequiredText" }
    }

    $Gate = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "docs\BETA_RELEASE_GATE.md")
    foreach ($RequiredText in @("data loss","secret leakage","Agent","all ten expected executables","known limitations")) {
        if (-not $Gate.Contains($RequiredText)) { throw "Beta release gate missing requirement: $RequiredText" }
    }

    $Evaluator = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "scripts\evaluate-beta-qualification.ps1")
    foreach ($RequiredText in @("BQ-01","BQ-06","CandidateCommit","BETA QUALIFICATION EVALUATION: PASS")) {
        if (-not $Evaluator.Contains($RequiredText)) { throw "Qualification evaluator invariant missing: $RequiredText" }
    }

    $RecordScript = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "scripts\new-beta-qualification-record.ps1")
    foreach ($RequiredText in @("qualification_log_sha256","disposable_test_data_only","scenario_id","Get-FileHash","PHASE 13 BETA READINESS & RELEASE QUALIFICATION VERIFICATION: PASS","ValidateSet(\"BQ-01\"")) {
        if (-not $RecordScript.Contains($RequiredText)) { throw "Qualification record invariant missing: $RequiredText" }
    }

    foreach ($Script in @(
        "new-beta-qualification-record.ps1",
        "evaluate-beta-qualification.ps1",
        "run-phase13-beta-readiness-tests.ps1",
        "run-dependency-audit.ps1",
        "review-windows-data-permissions.ps1",
        "package-windows-release.ps1",
        "package-windows-installer.ps1",
        "verify-release-artifacts.ps1"
    )) {
        $Path = Join-Path $PSScriptRoot $Script
        $Tokens = $null
        $Errors = $null
        [void][System.Management.Automation.Language.Parser]::ParseFile($Path, [ref]$Tokens, [ref]$Errors)
        if ($Errors.Count -ne 0) { throw "PowerShell syntax validation failed for $Script : $($Errors[0].Message)" }
        Write-Host "POWERSHELL SYNTAX PASS  $Script"
    }

    Write-Host ""
    Write-Host "PHASE 13 BETA READINESS & RELEASE QUALIFICATION VERIFICATION: PASS"
}
catch {
    Write-Host ""
    Write-Host "PHASE 13 BETA READINESS & RELEASE QUALIFICATION VERIFICATION: FAIL"
    Write-Host $_
    throw
}
finally {
    try { Stop-Transcript | Out-Null } catch {}
    foreach ($Path in $QualificationArtifacts) {
        if ($Path -and (Test-Path -LiteralPath $Path)) {
            Remove-Item -LiteralPath $Path -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
    Pop-Location
    if (Test-Path -LiteralPath $LogPath) {
        $Hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $LogPath).Hash
        "$Hash  $(Split-Path -Leaf $LogPath)" | Set-Content -LiteralPath $HashPath -Encoding ASCII
        Write-Host "Log: $LogPath"
        Write-Host "SHA256: $Hash"
    }
}
