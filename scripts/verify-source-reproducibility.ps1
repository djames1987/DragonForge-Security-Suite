[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
Push-Location $RepoRoot
try {
    $Dirty = @(& git status --porcelain)
    if ($LASTEXITCODE -ne 0) { throw "Unable to inspect Git working tree." }
    if ($Dirty.Count -gt 0) { throw "Source reproducibility verification requires a clean working tree." }

    $Head = (& git rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0 -or -not $Head) { throw "Unable to resolve HEAD." }

    $Temp = Join-Path ([IO.Path]::GetTempPath()) ("dragonforge-phase24-source-" + [Guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Force -Path $Temp | Out-Null
    try {
        $A = Join-Path $Temp "source-a.tar"
        $B = Join-Path $Temp "source-b.tar"
        & git archive --format=tar --output=$A $Head
        if ($LASTEXITCODE -ne 0) { throw "First git archive failed." }
        & git archive --format=tar --output=$B $Head
        if ($LASTEXITCODE -ne 0) { throw "Second git archive failed." }

        $HashA = (Get-FileHash -Algorithm SHA256 -LiteralPath $A).Hash
        $HashB = (Get-FileHash -Algorithm SHA256 -LiteralPath $B).Hash
        if ($HashA -ne $HashB) { throw "Identical source commit produced different source archive hashes." }

        Write-Host "SOURCE REPRODUCIBILITY: PASS"
        Write-Host "Commit: $Head"
        Write-Host "Archive SHA256: $HashA"
    }
    finally {
        if (Test-Path -LiteralPath $Temp) { Remove-Item -LiteralPath $Temp -Recurse -Force -ErrorAction SilentlyContinue }
    }
}
finally {
    Pop-Location
}
