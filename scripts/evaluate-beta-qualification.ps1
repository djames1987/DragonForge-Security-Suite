[CmdletBinding()]
param(
    [string]$CandidateCommit = "",
    [string]$EvidenceDirectory = ""
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
if (-not $EvidenceDirectory) {
    $EvidenceDirectory = Join-Path $RepoRoot "test-logs\beta-qualification"
}
if (-not $CandidateCommit) {
    Push-Location $RepoRoot
    try {
        $CandidateCommit = (& git rev-parse HEAD).Trim()
        if ($LASTEXITCODE -ne 0 -or -not $CandidateCommit) { throw "Unable to determine candidate commit." }
    }
    finally { Pop-Location }
}

$Required = @("BQ-01","BQ-02","BQ-03","BQ-04","BQ-05","BQ-06")
if (-not (Test-Path -LiteralPath $EvidenceDirectory -PathType Container)) {
    throw "Beta qualification evidence directory does not exist: $EvidenceDirectory"
}

$Records = @()
foreach ($File in Get-ChildItem -LiteralPath $EvidenceDirectory -Filter "qualification-*.json" -File) {
    try {
        $Record = Get-Content -Raw -LiteralPath $File.FullName | ConvertFrom-Json
        if ($Record.source.commit -eq $CandidateCommit) {
            $Records += [pscustomobject]@{ File = $File; Record = $Record }
        }
    }
    catch {
        throw "Malformed beta qualification record: $($File.FullName)"
    }
}

$Failures = @()
foreach ($Scenario in $Required) {
    $Candidates = @($Records | Where-Object { $_.Record.scenario_id -eq $Scenario } | Sort-Object { $_.Record.recorded_utc } -Descending)
    if ($Candidates.Count -eq 0) {
        $Failures += "$Scenario has no qualification record for commit $CandidateCommit"
        continue
    }
    $Record = $Candidates[0].Record
    if ($Record.results.automated_qualification -ne "pass") { $Failures += "$Scenario automated qualification is not pass" }
    if ($Record.results.agent_lifecycle -ne "pass") { $Failures += "$Scenario Agent lifecycle is not pass" }
    if ($Record.results.suite_launch -ne "pass") { $Failures += "$Scenario suite launch is not pass" }
    if ($Record.results.reboot -eq "fail") { $Failures += "$Scenario reboot result failed" }
    if ($Record.results.installer_lifecycle -eq "fail") { $Failures += "$Scenario installer lifecycle failed" }
    if ($Record.disposable_test_data_only -ne $true) { $Failures += "$Scenario does not attest disposable test data only" }
}

if ($Failures.Count -gt 0) {
    Write-Host "BETA QUALIFICATION EVALUATION: FAIL"
    $Failures | ForEach-Object { Write-Host " - $_" }
    exit 1
}

Write-Host "BETA QUALIFICATION EVALUATION: PASS"
Write-Host "Candidate commit: $CandidateCommit"
foreach ($Scenario in $Required) {
    $Latest = @($Records | Where-Object { $_.Record.scenario_id -eq $Scenario } | Sort-Object { $_.Record.recorded_utc } -Descending)[0]
    Write-Host "PASS  $Scenario  $($Latest.File.Name)"
}
