$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$LogDirectory = Join-Path $RepoRoot "test-logs"
$Timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogPath = Join-Path $LogDirectory "dragonforge-phase12.6-ux-consistency-$Timestamp.log"
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

$Apps = @("security-center","password-manager","file-vault","authenticator","security-scanner","integrity-monitor","network-guard","backup-recovery","secure-share")

try {
    Start-Transcript -Path $LogPath -Force | Out-Null
    Write-Host "DragonForge Security Suite - Phase 12.6 UX Consistency verification"
    Write-Host "Repository: $RepoRoot"

    Invoke-Checked cargo "fmt" "--all" "--check"
    Invoke-Checked cargo "check" "-p" "dragonforge-core" "-p" "dragonforge-agent" "-p" "dragonforge-security-center" "--all-targets"

    foreach ($App in $Apps) {
        $Js = Join-Path $RepoRoot "apps\$App\ui\app.js"
        Invoke-Checked node "--check" $Js
    }

    $Workspace = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "Cargo.toml")
    if (-not $Workspace.Contains('version = "0.1.0"')) { throw "Workspace version is not the expected Phase 12.6 visible version." }

    foreach ($App in $Apps) {
        $HtmlPath = Join-Path $RepoRoot "apps\$App\ui\index.html"
        $CssPath = Join-Path $RepoRoot "apps\$App\ui\app.css"
        if (-not (Test-Path -LiteralPath $HtmlPath -PathType Leaf)) { throw "Missing UI HTML for $App" }
        if (-not (Test-Path -LiteralPath $CssPath -PathType Leaf)) { throw "Missing UI CSS for $App" }
        $Html = Get-Content -Raw -LiteralPath $HtmlPath
        $Css = Get-Content -Raw -LiteralPath $CssPath
        if (-not $Html.Contains("Version 0.1.0")) { throw "$App does not expose Version 0.1.0" }
        if (-not $Html.Contains("Suite Phase 12.6")) { throw "$App does not expose Suite Phase 12.6" }
        if (-not $Html.Contains("DragonForge")) { throw "$App does not expose DragonForge product identity" }
        if (-not $Css.Contains("Phase 12.6 suite UX baseline")) { throw "$App is missing the shared Phase 12.6 CSS baseline" }
        if (-not $Css.Contains(":focus-visible")) { throw "$App is missing visible keyboard focus styling" }
        if (-not $Css.Contains("button:disabled")) { throw "$App is missing disabled-control styling" }
        Write-Host "UX-METADATA PASS  $App"
    }

    $FileVaultJs = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "apps\file-vault\ui\app.js")
    $BackupJs = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "apps\backup-recovery\ui\app.js")
    $ShareJs = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "apps\secure-share\ui\app.js")
    $AuthJs = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "apps\authenticator\ui\app.js")
    foreach ($Check in @(
        @($FileVaultJs, "Extract this authenticated vault"),
        @($BackupJs, "Verify and restore this backup"),
        @($ShareJs, "Verify and extract these attachments"),
        @($AuthJs, "Replace the encrypted recovery codes")
    )) {
        if (-not $Check[0].Contains($Check[1])) { throw "Sensitive-action confirmation missing: $($Check[1])" }
    }

    foreach ($App in @("integrity-monitor","network-guard")) {
        $Html = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "apps\$App\ui\index.html")
        if ($Html.Contains("future Agent")) { throw "$App still contains stale pre-Phase-11 Agent wording" }
    }

    $Doc = Join-Path $RepoRoot "docs\PHASE_12_6_UX_CONSISTENCY.md"
    if (-not (Test-Path -LiteralPath $Doc -PathType Leaf)) { throw "Phase 12.6 documentation is missing" }
    Write-Host "OK  docs/PHASE_12_6_UX_CONSISTENCY.md"
    Write-Host ""
    Write-Host "PHASE 12.6 UX CONSISTENCY VERIFICATION: PASS"
}
catch {
    Write-Host ""
    Write-Host "PHASE 12.6 UX CONSISTENCY VERIFICATION: FAIL"
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
