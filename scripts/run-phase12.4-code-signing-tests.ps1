$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$LogDirectory = Join-Path $RepoRoot "test-logs"
$Timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogPath = Join-Path $LogDirectory "dragonforge-phase12.4-code-signing-$Timestamp.log"
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
    Write-Host "DragonForge Security Suite - Phase 12.4 Code Signing verification"
    Write-Host "Repository: $RepoRoot"

    Invoke-Checked cargo "fmt" "--all" "--check"
    Invoke-Checked cargo "check" "-p" "dragonforge-core" "-p" "dragonforge-agent" "-p" "dragonforge-security-center" "--all-targets"
    Invoke-Checked cargo "clippy" "-p" "dragonforge-core" "-p" "dragonforge-agent" "-p" "dragonforge-security-center" "--all-targets" "--" "-D" "warnings"
    Invoke-Checked cargo "test" "-p" "dragonforge-agent" "-p" "dragonforge-security-center" "--all-targets"
    Invoke-Checked node "--check" "apps/security-center/ui/app.js"

    $Scripts = @(
        "scripts/sign-windows-files.ps1",
        "scripts/verify-authenticode.ps1",
        "scripts/package-windows-release.ps1",
        "scripts/package-windows-installer.ps1",
        "scripts/build-tagged-windows-release.ps1",
        "scripts/verify-release-artifacts.ps1",
        "scripts/publish-windows-release.ps1"
    )

    foreach ($Path in $Scripts) {
        $Full = Join-Path $RepoRoot $Path
        if (-not (Test-Path -LiteralPath $Full -PathType Leaf)) { throw "Missing Phase 12.4 script: $Path" }
        try { [void][scriptblock]::Create((Get-Content -Raw -LiteralPath $Full)) }
        catch { throw "PowerShell syntax invalid in $Path : $($_.Exception.Message)" }
        Write-Host "PS-SYNTAX  $Path"
    }

    $Sign = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "scripts\sign-windows-files.ps1")
    foreach ($Required in @(
        'DRAGONFORGE_SIGN_CERT_THUMBPRINT',
        'DRAGONFORGE_SIGN_PFX_PATH',
        'DRAGONFORGE_SIGN_PFX_PASSWORD',
        'DRAGONFORGE_SIGN_TIMESTAMP_URL',
        '"/fd", "SHA256"',
        '"/td", "SHA256"',
        '"/tr"',
        'verify /pa /all /tw',
        '<redacted>'
    )) {
        if (-not $Sign.Contains($Required)) { throw "Signing invariant missing: $Required" }
    }

    $Portable = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "scripts\package-windows-release.ps1")
    $SignIndex = $Portable.IndexOf('sign-windows-files.ps1')
    $HashIndex = $Portable.IndexOf('$HashLines = Get-ChildItem')
    if ($SignIndex -lt 0 -or $HashIndex -lt 0 -or $SignIndex -gt $HashIndex) {
        throw "Portable signing must occur before internal hashing."
    }

    $Installer = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "scripts\package-windows-installer.ps1")
    $CompileIndex = $Installer.IndexOf('& $Iscc @Arguments')
    $InstallerSignIndex = $Installer.IndexOf('sign-windows-files.ps1')
    $InstallerHashIndex = $Installer.IndexOf('$InstallerHash =')
    if ($CompileIndex -lt 0 -or $InstallerSignIndex -lt $CompileIndex -or $InstallerHashIndex -lt $InstallerSignIndex) {
        throw "Installer signing order is invalid."
    }

    $Tagged = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "scripts\build-tagged-windows-release.ps1")
    foreach ($Required in @(
        'Stable releases must be Authenticode-signed',
        '-SignRelease:$SignRelease',
        '-RequireSigning:$SignRelease'
    )) {
        if (-not $Tagged.Contains($Required)) { throw "Tagged release signing invariant missing: $Required" }
    }

    $Artifacts = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "scripts\verify-release-artifacts.ps1")
    foreach ($Required in @(
        'verify-authenticode.ps1',
        '-RequireTimestamp',
        'authenticode-sha256-rfc3161',
        'signer_subject',
        'signer_thumbprint',
        'unsigned-development'
    )) {
        if (-not $Artifacts.Contains($Required)) { throw "Artifact signature invariant missing: $Required" }
    }

    $Ignore = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot ".gitignore")
    foreach ($Pattern in @("*.pfx", "*.p12", "*.pvk", "*.snk")) {
        if (-not $Ignore.Contains($Pattern)) { throw "Signing secret file pattern is not ignored: $Pattern" }
    }

    foreach ($Path in @("docs/PHASE_12_4_CODE_SIGNING.md")) {
        if (-not (Test-Path -LiteralPath (Join-Path $RepoRoot $Path) -PathType Leaf)) { throw "Required Phase 12.4 artifact missing: $Path" }
        Write-Host "OK  $Path"
    }

    Write-Host ""
    Write-Host "PHASE 12.4 CODE SIGNING VERIFICATION: PASS"
}
catch {
    Write-Host ""
    Write-Host "PHASE 12.4 CODE SIGNING VERIFICATION: FAIL"
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
