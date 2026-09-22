$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$LogDirectory = Join-Path $RepoRoot "test-logs"
$Timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogPath = Join-Path $LogDirectory "dragonforge-phase12.2-agent-lifecycle-$Timestamp.log"
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
    Write-Host "DragonForge Security Suite - Phase 12.2 Agent Lifecycle verification"
    Write-Host "Repository: $RepoRoot"

    Invoke-Checked cargo "fmt" "--all" "--check"
    Invoke-Checked cargo "check" "-p" "dragonforge-agent" "-p" "dragonforge-agent-service" "-p" "dragonforge-security-center" "--all-targets"
    Invoke-Checked cargo "clippy" "-p" "dragonforge-agent" "-p" "dragonforge-agent-service" "-p" "dragonforge-security-center" "--all-targets" "--" "-D" "warnings"
    Invoke-Checked cargo "test" "-p" "dragonforge-agent" "-p" "dragonforge-agent-service" "-p" "dragonforge-security-center" "--all-targets"
    Invoke-Checked node "--check" "apps/security-center/ui/app.js"

    foreach ($Path in @(
        "docs/PHASE_12_2_AGENT_LIFECYCLE.md",
        "installer/DragonForgeSecuritySuite.iss",
        "scripts/package-windows-installer.ps1"
    )) {
        if (-not (Test-Path -LiteralPath (Join-Path $RepoRoot $Path) -PathType Leaf)) {
            throw "Required Phase 12.2 artifact is missing: $Path"
        }
        Write-Host "OK  $Path"
    }

    $AgentSource = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "crates\dragonforge-agent\src\server.rs")
    foreach ($Required in @("shutdown", "graceful-shutdown", "restartable-session")) {
        if (-not $AgentSource.Contains($Required)) { throw "Agent lifecycle source missing: $Required" }
    }

    $Installer = Get-Content -Raw -LiteralPath (Join-Path $RepoRoot "installer\DragonForgeSecuritySuite.iss")
    if (-not $Installer.Contains('{userstartup}\DragonForge Agent')) {
        throw "Installer does not define current-user Agent login startup."
    }

    Write-Host ""
    Write-Host "PHASE 12.2 AGENT LIFECYCLE VERIFICATION: PASS"
}
catch {
    Write-Host ""
    Write-Host "PHASE 12.2 AGENT LIFECYCLE VERIFICATION: FAIL"
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
