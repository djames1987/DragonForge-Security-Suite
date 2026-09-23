$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
if ($env:OS -ne "Windows_NT") { throw "This acceptance test is Windows-only." }
$ServiceName = "DragonForgePrivilegedService"
$Service = Get-CimInstance Win32_Service -Filter "Name='$ServiceName'" -ErrorAction Stop
if ($Service.State -ne "Running") { throw "Privileged service is not running." }
if ($Service.StartName -ne "NT SERVICE\$ServiceName") { throw "Unexpected privileged service account: $($Service.StartName)" }

$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$Agent = Join-Path $Root "dragonforge-agent.exe"
if (-not (Test-Path -LiteralPath $Agent)) {
    $RepoRoot = Split-Path -Parent $PSScriptRoot
    $Agent = Join-Path $RepoRoot "target\release\dragonforge-agent.exe"
}
if (-not (Test-Path -LiteralPath $Agent)) { throw "dragonforge-agent.exe was not found." }

$HealthJson = & $Agent --privileged-health
if ($LASTEXITCODE -ne 0) { throw "Agent could not authenticate to privileged service health." }
$Health = ($HealthJson -join [Environment]::NewLine) | ConvertFrom-Json
if ($Health.state -ne "healthy" -or $Health.privilegedCapabilitiesEnabled -ne $true) {
    throw "Phase 19 privileged service health contract is invalid."
}

$PolicyJson = & $Agent --privileged-policy
if ($LASTEXITCODE -ne 0) { throw "Agent could not retrieve privileged service policy." }
$Policy = ($PolicyJson -join [Environment]::NewLine) | ConvertFrom-Json
if ($Policy.privilegedCapabilitiesEnabled -ne $true) { throw "Phase 19 firewall mutation capability is not enabled." }
if ($Policy.arbitraryCommandExecutionProhibited -ne $true -or $Policy.genericShellExecutionProhibited -ne $true) {
    throw "Privileged service generic execution protections are missing."
}
$Commands = @($Policy.allowedCommands)
$ExpectedCommands = @(
    "health",
    "describe-policy",
    "firewall-status",
    "firewall-apply",
    "firewall-remove",
    "firewall-rollback"
)
if ($Commands.Count -ne $ExpectedCommands.Count) {
    throw "Unexpected privileged service command count."
}
foreach ($Command in $ExpectedCommands) {
    if ($Commands -notcontains $Command) {
        throw "Missing privileged service command: $Command"
    }
}
Write-Host "DRAGONFORGE PRIVILEGED SERVICE PHASE 19 ACCEPTANCE: PASS" -ForegroundColor Green
