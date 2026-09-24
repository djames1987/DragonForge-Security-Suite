param(
    [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot ".."))
)

$ErrorActionPreference = "Stop"
$Failed = $false
$LogDir = Join-Path $RepoRoot "test-logs"
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
$Stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogPath = Join-Path $LogDir "dragonforge-phase23-privacy-accessibility-product-polish-$Stamp.log"

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
    Write-Host "DragonForge Security Suite - Phase 23 Privacy, Accessibility & Product Polish verification"
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

    $Apps = @(
        "security-center",
        "password-manager",
        "file-vault",
        "authenticator",
        "security-scanner",
        "integrity-monitor",
        "network-guard",
        "backup-recovery",
        "secure-share"
    )

    foreach ($App in $Apps) {
        Run "node --check $App app.js" { node --check "apps/$App/ui/app.js" }
        Run "node --check $App phase23.js" { node --check "apps/$App/ui/phase23.js" }

        $HtmlPath = "apps/$App/ui/index.html"
        $CssPath = "apps/$App/ui/phase23.css"
        $JsPath = "apps/$App/ui/phase23.js"
        foreach ($File in @($HtmlPath, $CssPath, $JsPath)) {
            if (-not (Test-Path -LiteralPath $File -PathType Leaf)) {
                throw "Missing Phase 23 UI artifact: $File"
            }
        }

        $Html = Get-Content -Raw $HtmlPath
        foreach ($Needle in @(
            'href="phase23.css"',
            'src="phase23.js"',
            'Suite Phase 23'
        )) {
            if (-not $Html.Contains($Needle)) {
                throw "$App is missing Phase 23 HTML invariant: $Needle"
            }
        }
        if ($Html -match '(?i)(?:src|href)\s*=\s*["'']https?://') {
            throw "$App introduced a remote script, stylesheet, or image dependency."
        }

        $Css = Get-Content -Raw $CssPath
        foreach ($Needle in @(
            'focus-visible',
            'prefers-reduced-motion',
            'forced-colors',
            'phase23-skip-link',
            'phase23-sr-only',
            'phase23-state-error'
        )) {
            if (-not $Css.Contains($Needle)) {
                throw "$App is missing Phase 23 accessibility CSS invariant: $Needle"
            }
        }

        $Js = Get-Content -Raw $JsPath
        foreach ($Needle in @(
            'DragonForgeUX',
            'MutationObserver',
            'aria-current',
            'aria-live',
            'data-l10n-ready',
            'renderState',
            'setBusy',
            'ArrowDown',
            'firstRunSeen'
        )) {
            if (-not $Js.Contains($Needle)) {
                throw "$App is missing Phase 23 accessibility runtime invariant: $Needle"
            }
        }
        foreach ($Forbidden in @('fetch(', 'XMLHttpRequest', 'WebSocket(', 'navigator.sendBeacon')) {
            if ($Js.Contains($Forbidden)) {
                throw "$App Phase 23 runtime introduced forbidden network behavior: $Forbidden"
            }
        }
        Write-Host "PHASE 23 UI PASS  $App"
    }

    $CenterHtml = Get-Content -Raw "apps/security-center/ui/index.html"
    foreach ($Needle in @(
        'Privacy & accessibility',
        'Local-first by default',
        'Diagnostics privacy',
        'Accessibility contract',
        'Language readiness'
    )) {
        if (-not $CenterHtml.Contains($Needle)) {
            throw "Security Center privacy/accessibility guidance is missing: $Needle"
        }
    }

    $Branding = @{
        "security-center" = "../../assets/branding/logos/dragonforge-security-center.png"
        "password-manager" = "icons/icon.png"
        "file-vault" = "../../assets/branding/logos/dragonforge-file-vault.png"
        "authenticator" = "../../assets/branding/logos/dragonforge-authenticator.png"
        "security-scanner" = "../../assets/branding/logos/dragonforge-security-scanner.png"
        "integrity-monitor" = "../../assets/branding/logos/dragonforge-integrity-monitor.png"
        "network-guard" = "../../assets/branding/logos/dragonforge-network-guard.png"
        "backup-recovery" = "../../assets/branding/logos/dragonforge-backup-and-recovery.png"
        "secure-share" = "../../assets/branding/logos/dragonforge-secure-share.png"
    }

    foreach ($App in $Apps) {
        $ConfigPath = "apps/$App/tauri.conf.json"
        $Config = Get-Content -Raw $ConfigPath | ConvertFrom-Json
        $Icons = @($Config.bundle.icon)
        $Expected = $Branding[$App]
        if ($Icons.Count -ne 1 -or $Icons[0] -ne $Expected) {
            throw "$App Tauri branding path is not the approved Phase 23 value."
        }
        $Resolved = [IO.Path]::GetFullPath((Join-Path (Split-Path $ConfigPath -Parent) $Expected))
        if (-not (Test-Path -LiteralPath $Resolved -PathType Leaf)) {
            throw "$App branding asset does not exist: $Resolved"
        }
        Write-Host "BRANDING PASS  $App -> $Expected"
    }

    foreach ($File in @(
        "assets/branding/logos/dragonforge-security-suite.png",
        "assets/branding/logos/dragonforge-agent.png",
        "docs/PHASE_23_PRIVACY_ACCESSIBILITY_PRODUCT_POLISH.md",
        "docs/adr/0016-suite-privacy-accessibility-ui-contract.md"
    )) {
        if (-not (Test-Path -LiteralPath $File -PathType Leaf)) {
            throw "Missing Phase 23 branding/documentation artifact: $File"
        }
        Write-Host "OK  $File"
    }

    foreach ($File in @(
        "scripts/run-phase23-privacy-accessibility-product-polish-tests.ps1",
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
    Write-Host "PHASE 23 PRIVACY, ACCESSIBILITY & PRODUCT POLISH VERIFICATION: PASS"
}
catch {
    $Failed = $true
    Write-Host ""
    Write-Host "PHASE 23 PRIVACY, ACCESSIBILITY & PRODUCT POLISH VERIFICATION: FAIL"
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
