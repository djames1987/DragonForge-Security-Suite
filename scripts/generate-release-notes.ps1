param(
    [Parameter(Mandatory = $true)] [string]$Tag,
    [string]$OutputPath
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$TemplatePath = Join-Path $RepoRoot "release\RELEASE_NOTES_TEMPLATE.md"
if (-not $OutputPath) {
    $OutputPath = Join-Path $RepoRoot "dist\release-notes-$Tag.md"
}

Push-Location $RepoRoot
try {
    if (-not (Test-Path -LiteralPath $TemplatePath -PathType Leaf)) { throw "Release notes template is missing." }
    $Commit = (& git rev-parse "$Tag^{commit}").Trim()
    if ($LASTEXITCODE -ne 0 -or -not $Commit) { throw "Tag $Tag could not be resolved." }

    $PreviousErrorActionPreference = $ErrorActionPreference
    try {
        $ErrorActionPreference = "Continue"
        $PreviousTag = (& git describe --tags --abbrev=0 "$Tag^" 2>$null).Trim()
        $HasPrevious = ($LASTEXITCODE -eq 0 -and $PreviousTag)
    }
    finally { $ErrorActionPreference = $PreviousErrorActionPreference }

    if ($HasPrevious) {
        $LogRange = "$PreviousTag..$Tag"
    } else {
        $LogRange = $Tag
    }

    $Changes = @(& git log $LogRange --pretty=format:"- %h %s" --no-merges)
    if ($LASTEXITCODE -ne 0) { throw "Unable to generate release change list." }
    if (-not $Changes) { $Changes = @("- No non-merge changes recorded.") }

    $Channel = if ($Tag -match '-') { "pre-release / external testing" } else { "stable" }
    $Text = Get-Content -Raw -LiteralPath $TemplatePath
    $Text = $Text.Replace("{{TAG}}", $Tag)
    $Text = $Text.Replace("{{COMMIT}}", $Commit)
    $Text = $Text.Replace("{{CHANNEL}}", $Channel)
    $Text = $Text.Replace("{{CHANGES}}", ($Changes -join [Environment]::NewLine))

    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $OutputPath) | Out-Null
    Set-Content -LiteralPath $OutputPath -Value $Text -Encoding UTF8
    Write-Host "RELEASE NOTES GENERATED: $OutputPath"
}
finally { Pop-Location }
