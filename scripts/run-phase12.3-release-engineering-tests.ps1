$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$LogDirectory = Join-Path $RepoRoot "test-logs"
$Timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogPath = Join-Path $LogDirectory "dragonforge-phase12.3-release-engineering-$Timestamp.log"
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
    finally {
        $ErrorActionPreference = $Previous
    }

    if ($ExitCode -ne 0) {
        throw "Command failed with exit code $ExitCode - $Command $($Arguments -join ' ')"
    }
}

try {
    Start-Transcript -Path $LogPath -Force | Out-Null

    Write-Host "DragonForge Security Suite - Phase 12.3 Release Engineering verification"
    Write-Host "Repository: $RepoRoot"

    Invoke-Checked cargo "fmt" "--all" "--check"
    Invoke-Checked cargo "check" "-p" "dragonforge-core" "-p" "dragonforge-agent" "-p" "dragonforge-security-center" "--all-targets"
    Invoke-Checked cargo "clippy" "-p" "dragonforge-core" "-p" "dragonforge-agent" "-p" "dragonforge-security-center" "--all-targets" "--" "-D" "warnings"
    Invoke-Checked cargo "test" "-p" "dragonforge-agent" "-p" "dragonforge-security-center" "--all-targets"
    Invoke-Checked node "--check" "apps/security-center/ui/app.js"

    & git ls-files --error-unmatch Cargo.lock *> $null
    if ($LASTEXITCODE -ne 0) {
        throw "Cargo.lock is not tracked."
    }
    Write-Host "OK  Cargo.lock tracked"

    Invoke-Checked cargo "metadata" "--locked" "--format-version" "1" "--no-deps"

    $WorkspaceManifest = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "Cargo.toml")
    if ($WorkspaceManifest -notmatch '(?m)^version\s*=\s*"([^"]+)"\s*$') {
        throw "Workspace version could not be resolved."
    }

    $CurrentVersion = $Matches[1]
    & (Join-Path $PSScriptRoot "set-release-version.ps1") -Version $CurrentVersion -CheckOnly
    if ($LASTEXITCODE -ne 0) {
        throw "Suite-wide version consistency check failed."
    }

    $ReleaseScripts = @(
        "scripts/set-release-version.ps1",
        "scripts/prepare-release.ps1",
        "scripts/generate-release-notes.ps1",
        "scripts/build-tagged-windows-release.ps1",
        "scripts/verify-release-artifacts.ps1",
        "scripts/package-windows-release.ps1",
        "scripts/package-windows-installer.ps1",
        "scripts/publish-windows-release.ps1"
    )

    foreach ($Path in $ReleaseScripts) {
        $Full = Join-Path $RepoRoot $Path
        if (-not (Test-Path -LiteralPath $Full -PathType Leaf)) {
            throw "Missing release script: $Path"
        }

        $Source = Get-Content -Raw -LiteralPath $Full
        try {
            [void][scriptblock]::Create($Source)
        }
        catch {
            throw "PowerShell syntax invalid in $Path : $($_.Exception.Message)"
        }

        Write-Host "PS-SYNTAX  $Path"
    }

    foreach ($Path in @(
        "release/RELEASE_NOTES_TEMPLATE.md",
        "docs/PHASE_12_3_RELEASE_ENGINEERING.md"
    )) {
        if (-not (Test-Path -LiteralPath (Join-Path $RepoRoot $Path) -PathType Leaf)) {
            throw "Required Phase 12.3 artifact missing: $Path"
        }
        Write-Host "OK  $Path"
    }

    $TaggedBuild = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "scripts\build-tagged-windows-release.ps1")
    foreach ($Required in @(
        'git status --porcelain',
        'git rev-parse "$Tag^{commit}"',
        'git ls-files --error-unmatch Cargo.lock',
        'cargo metadata --locked',
        'verify-release-artifacts.ps1'
    )) {
        if (-not $TaggedBuild.Contains($Required)) {
            throw "Tagged release build invariant missing: $Required"
        }
    }

    $Publisher = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "scripts\publish-windows-release.ps1")
    foreach ($Required in @(
        'release view $Tag',
        '--verify-tag',
        'build-tagged-windows-release.ps1',
        'release-manifest-$Tag.json'
    )) {
        if (-not $Publisher.Contains($Required)) {
            throw "Publisher invariant missing: $Required"
        }
    }

    $ArtifactVerifier = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "scripts\verify-release-artifacts.ps1")
    foreach ($Required in @(
        'SHA256SUMS.txt',
        'BUILD-INFO.txt',
        'expected_executables',
        'unsigned-phase-12.3'
    )) {
        if (-not $ArtifactVerifier.Contains($Required)) {
            throw "Artifact verifier invariant missing: $Required"
        }
    }

    Write-Host ""
    Write-Host "PHASE 12.3 RELEASE ENGINEERING VERIFICATION: PASS"
}
catch {
    Write-Host ""
    Write-Host "PHASE 12.3 RELEASE ENGINEERING VERIFICATION: FAIL"
    Write-Host $_
    throw
}
finally {
    try {
        Stop-Transcript | Out-Null
    }
    catch {}

    Pop-Location

    if (Test-Path -LiteralPath $LogPath) {
        $Hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $LogPath).Hash
        "$Hash  $(Split-Path -Leaf $LogPath)" | Set-Content -LiteralPath $HashPath -Encoding ASCII
        Write-Host "Log: $LogPath"
        Write-Host "SHA256: $Hash"
    }
}
