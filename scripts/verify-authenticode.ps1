param(
    [Parameter(Mandatory = $true)] [string[]]$Path,
    [switch]$RequireTimestamp
)

$ErrorActionPreference = "Stop"

function Find-SignTool {
    $Command = Get-Command "signtool.exe" -ErrorAction SilentlyContinue
    if ($Command) { return $Command.Source }
    $Roots = @()
    $ProgramFilesX86 = [Environment]::GetEnvironmentVariable("ProgramFiles(x86)")
    if ($ProgramFilesX86) { $Roots += (Join-Path $ProgramFilesX86 "Windows Kits\10\bin") }
    if ($env:ProgramFiles) { $Roots += (Join-Path $env:ProgramFiles "Windows Kits\10\bin") }
    foreach ($Root in $Roots) {
        if (-not (Test-Path -LiteralPath $Root -PathType Container)) { continue }
        $Candidates = Get-ChildItem -LiteralPath $Root -Directory -ErrorAction SilentlyContinue |
            Sort-Object Name -Descending |
            ForEach-Object { Join-Path $_.FullName "x64\signtool.exe" }
        foreach ($Candidate in $Candidates) { if (Test-Path -LiteralPath $Candidate -PathType Leaf) { return $Candidate } }
    }
    throw "SignTool.exe was not found. Install a Windows SDK that includes SignTool."
}

$SignTool = Find-SignTool
foreach ($Item in $Path) {
    $FullPath = [System.IO.Path]::GetFullPath($Item)
    if (-not (Test-Path -LiteralPath $FullPath -PathType Leaf)) { throw "Signature verification target does not exist: $FullPath" }
    $Arguments = @("verify", "/pa", "/all")
    if ($RequireTimestamp) { $Arguments += "/tw" }
    $Arguments += $FullPath
    Write-Host ">>> $SignTool $($Arguments -join ' ')"
    & $SignTool @Arguments
    if ($LASTEXITCODE -ne 0) { throw "Authenticode verification failed for $FullPath with exit code $LASTEXITCODE." }
    $Signature = Get-AuthenticodeSignature -LiteralPath $FullPath
    if ($Signature.Status -ne "Valid") { throw "PowerShell Authenticode validation is not Valid for $FullPath: $($Signature.Status)" }
    Write-Host "VALID  $FullPath"
}
Write-Host "AUTHENTICODE VERIFICATION: PASS"
