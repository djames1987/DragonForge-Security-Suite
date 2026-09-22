param()

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$LogDir = Join-Path $RepoRoot "test-logs"
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
$Stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogPath = Join-Path $LogDir "dragonforge-phase12-external-test-baseline-$Stamp.log"
$HashPath = "$LogPath.sha256"

function Invoke-Checked {
    param(
        [Parameter(Mandatory = $true)] [string]$Command,
        [Parameter(ValueFromRemainingArguments = $true)] [string[]]$Arguments
    )
    Write-Host ""
    Write-Host ">>> $Command $($Arguments -join ' ')"
    $PreviousErrorActionPreference = $ErrorActionPreference
    try {
        $ErrorActionPreference = "Continue"
        & $Command @Arguments 2>&1 | ForEach-Object { Write-Host $_ }
        $CommandExitCode = $LASTEXITCODE
    }
    finally {
        $ErrorActionPreference = $PreviousErrorActionPreference
    }
    if ($CommandExitCode -ne 0) {
        throw "Command failed with exit code $CommandExitCode - $Command $($Arguments -join ' ')"
    }
}

Push-Location $RepoRoot
$Passed = $false
try {
    Start-Transcript -Path $LogPath -Force
    Write-Host "DragonForge Security Suite - Phase 12.0 External Test Baseline verification"
    Write-Host "Repository: $RepoRoot"

    Invoke-Checked cargo "fmt" "--all" "--check"
    Invoke-Checked cargo "check" "-p" "dragonforge-core" "-p" "dragonforge-agent" "-p" "dragonforge-agent-service" "-p" "dragonforge-security-center" "--all-targets"
    Invoke-Checked cargo "clippy" "-p" "dragonforge-core" "-p" "dragonforge-agent" "-p" "dragonforge-agent-service" "-p" "dragonforge-security-center" "--all-targets" "--" "-D" "warnings"
    Invoke-Checked cargo "test" "-p" "dragonforge-agent" "-p" "dragonforge-agent-service" "-p" "dragonforge-security-center" "--all-targets"
    Invoke-Checked node "--check" "apps/security-center/ui/app.js"

    foreach ($Path in @(
        "docs/PHASE_12_0_EXTERNAL_TEST_BASELINE.md",
        "docs/EXTERNAL_TEST_CHECKLIST.md",
        "docs/EXTERNAL_TEST_MATRIX.md",
        ".github/ISSUE_TEMPLATE/external-test-bug.yml",
        "scripts/package-windows-release.ps1"
    )) {
        if (-not (Test-Path -LiteralPath (Join-Path $RepoRoot $Path) -PathType Leaf)) {
            throw "Required Phase 12.0 artifact is missing: $Path"
        }
        Write-Host "OK  $Path"
    }

    $Passed = $true
    Write-Host ""
    Write-Host "PHASE 12.0 EXTERNAL TEST BASELINE VERIFICATION: PASS"
}
catch {
    Write-Host ""
    Write-Host "PHASE 12.0 EXTERNAL TEST BASELINE VERIFICATION: FAIL"
    Write-Host $_
    throw
}
finally {
    try { Stop-Transcript | Out-Null } catch {}
    if (Test-Path -LiteralPath $LogPath) {
        $Hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $LogPath).Hash
        "$Hash  $(Split-Path -Leaf $LogPath)" | Set-Content -LiteralPath $HashPath -Encoding ASCII
        Write-Host "Log: $LogPath"
        Write-Host "SHA256: $Hash"
    }
    Pop-Location
}

if (-not $Passed) { exit 1 }
