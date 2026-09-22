[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)] [ValidateSet("BQ-01","BQ-02","BQ-03","BQ-04","BQ-05","BQ-06")] [string]$ScenarioId,
    [Parameter(Mandatory = $true)] [ValidateSet("physical","vm")] [string]$MachineType,
    [Parameter(Mandatory = $true)] [ValidateSet("standard","administrator-not-elevated")] [string]$UserContext,
    [Parameter(Mandatory = $true)] [ValidateSet("fresh","established")] [string]$ProfileState,
    [Parameter(Mandatory = $true)] [ValidateSet("present","missing")] [string]$WebView2State,
    [Parameter(Mandatory = $true)] [ValidateSet("installer","portable")] [string]$PackageKind,
    [Parameter(Mandatory = $true)] [ValidateSet("pass","fail","not-required")] [string]$RebootResult,
    [Parameter(Mandatory = $true)] [ValidateSet("pass","fail","not-applicable")] [string]$InstallerLifecycleResult,
    [Parameter(Mandatory = $true)] [ValidateSet("pass","fail")] [string]$AgentLifecycleResult,
    [Parameter(Mandatory = $true)] [ValidateSet("pass","fail")] [string]$SuiteLaunchResult,
    [Parameter(Mandatory = $true)] [string]$QualificationLog,
    [string]$ReleaseTag = "",
    [string]$Notes = ""
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$EvidenceRoot = Join-Path $RepoRoot "test-logs\beta-qualification"
New-Item -ItemType Directory -Force -Path $EvidenceRoot | Out-Null

if (-not (Test-Path -LiteralPath $QualificationLog -PathType Leaf)) {
    throw "Qualification log does not exist: $QualificationLog"
}
$Sidecar = "$QualificationLog.sha256"
if (-not (Test-Path -LiteralPath $Sidecar -PathType Leaf)) {
    throw "Qualification SHA-256 sidecar does not exist: $Sidecar"
}
$Expected = ((Get-Content -Raw -LiteralPath $Sidecar).Trim() -split "\s+")[0].ToUpperInvariant()
$Actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $QualificationLog).Hash.ToUpperInvariant()
if ($Expected -ne $Actual) { throw "Qualification log SHA-256 does not match its sidecar." }
$LogText = Get-Content -Raw -LiteralPath $QualificationLog
if (-not $LogText.Contains("PHASE 13 BETA READINESS & RELEASE QUALIFICATION VERIFICATION: PASS")) {
    throw "Qualification log does not contain the Phase 13 PASS marker."
}

Push-Location $RepoRoot
try {
    $Commit = (& git rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0 -or -not $Commit) { throw "Unable to determine Git commit." }
}
finally { Pop-Location }

$Os = Get-CimInstance Win32_OperatingSystem
$Record = [ordered]@{
    schema_version = 1
    scenario_id = $ScenarioId
    recorded_utc = [DateTime]::UtcNow.ToString("o")
    source = [ordered]@{
        commit = $Commit
        release_tag = $(if ($ReleaseTag) { $ReleaseTag } else { $null })
    }
    platform = [ordered]@{
        windows_caption = $Os.Caption
        windows_version = $Os.Version
        windows_build = $Os.BuildNumber
        architecture = $env:PROCESSOR_ARCHITECTURE
        machine_type = $MachineType
        user_context = $UserContext
        profile_state = $ProfileState
        webview2 = $WebView2State
        package_kind = $PackageKind
    }
    results = [ordered]@{
        automated_qualification = "pass"
        agent_lifecycle = $AgentLifecycleResult
        suite_launch = $SuiteLaunchResult
        reboot = $RebootResult
        installer_lifecycle = $InstallerLifecycleResult
    }
    evidence = [ordered]@{
        qualification_log = (Split-Path -Leaf $QualificationLog)
        qualification_log_sha256 = $Actual
    }
    disposable_test_data_only = $true
    notes = $Notes
}
$SafeId = ($ScenarioId -replace '[^A-Za-z0-9._-]', '_')
$Output = Join-Path $EvidenceRoot ("qualification-{0}-{1}.json" -f $SafeId, (Get-Date -Format "yyyyMMdd-HHmmss"))
$Record | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $Output -Encoding UTF8
Write-Host "BETA QUALIFICATION RECORD: PASS"
Write-Host "Record: $Output"
Write-Host "Log SHA256: $Actual"
