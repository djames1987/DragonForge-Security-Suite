[CmdletBinding()]
param(
    [string]$OutputPath,
    [switch]$Quiet
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
if (-not $OutputPath) {
    $Stamp = Get-Date -Format "yyyyMMdd-HHmmss"
    $OutputPath = Join-Path $RepoRoot "test-logs\phase24-license-inventory-$Stamp.json"
}

$ReviewedComplexExpressions = @(
    "Apache-2.0 WITH LLVM-exception",
    "Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT",
    "(MIT OR Apache-2.0) AND Unicode-3.0"
)

$Allowed = @(
    "0BSD",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "BSL-1.0",
    "CC0-1.0",
    "CDLA-Permissive-2.0",
    "ISC",
    "MIT",
    "MIT-0",
    "MPL-2.0",
    "NCSA",
    "OpenSSL",
    "Python-2.0",
    "Unicode-3.0",
    "Unicode-DFS-2016",
    "Unlicense",
    "Zlib",
    "LLVM-exception"
)

Push-Location $RepoRoot
try {
    $Raw = & cargo metadata --locked --format-version 1
    if ($LASTEXITCODE -ne 0) { throw "cargo metadata failed during license audit." }
    $Metadata = $Raw | ConvertFrom-Json

    $Rows = @()
    $Failures = @()
    foreach ($Package in ($Metadata.packages | Sort-Object name, version)) {
        if (-not $Package.source) { continue }

        $License = [string]$Package.license
        if ([string]::IsNullOrWhiteSpace($License)) {
            $Failures += "$($Package.name) $($Package.version): missing license metadata"
            continue
        }

        $HasAnd = $License -match '\bAND\b'
        $HasOr = $License -match '\bOR\b'
        $IsComplex = $License -match '\bWITH\b|[()]' -or ($HasAnd -and $HasOr)

        if ($IsComplex) {
            if ($License -notin $ReviewedComplexExpressions) {
                $Failures += "$($Package.name) $($Package.version): $License (complex SPDX expression requires explicit review)"
            }
        }
        else {
            $Identifiers = @([regex]::Matches($License, '[A-Za-z0-9.+-]+') | ForEach-Object { $_.Value }) |
                Where-Object { $_ -notin @("AND","OR","WITH") } |
                Select-Object -Unique

            if ($HasAnd) {
                $Rejected = @($Identifiers | Where-Object { $_ -notin $Allowed })
                if ($Rejected.Count -gt 0) {
                    $Failures += "$($Package.name) $($Package.version): $License (unreviewed required identifiers: $($Rejected -join ', '))"
                }
            }
            else {
                # A single license must be reviewed. A pure SPDX OR expression
                # is acceptable when at least one distributable branch is reviewed.
                $Accepted = @($Identifiers | Where-Object { $_ -in $Allowed })
                if ($Identifiers.Count -eq 0 -or $Accepted.Count -eq 0) {
                    $Failures += "$($Package.name) $($Package.version): $License (no reviewed acceptable license option)"
                }
            }
        }

        $Rows += [ordered]@{
            name = $Package.name
            version = $Package.version
            source = $Package.source
            license = $License
        }
    }

    $Inventory = [ordered]@{
        schema_version = 1
        generated_utc = [DateTime]::UtcNow.ToString("o")
        repository = "djames1987/DragonForge-Security-Suite"
        package_count = $Rows.Count
        allowed_license_identifiers = $Allowed
        reviewed_complex_expressions = $ReviewedComplexExpressions
        packages = $Rows
        failures = $Failures
    }

    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $OutputPath) | Out-Null
    $Inventory | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $OutputPath -Encoding UTF8

    if ($Failures.Count -gt 0) {
        $Failures | ForEach-Object { Write-Host "LICENSE REVIEW REQUIRED  $_" }
        throw "Dependency license audit found unreviewed or missing license metadata."
    }

    if (-not $Quiet) {
        Write-Host "DEPENDENCY LICENSE AUDIT: PASS"
        Write-Host "Packages reviewed: $($Rows.Count)"
        Write-Host "Inventory: $OutputPath"
    }
}
finally {
    Pop-Location
}
