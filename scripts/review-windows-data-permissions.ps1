[CmdletBinding()]
param(
    [switch]$Strict
)

$ErrorActionPreference = "Stop"
$Local = $env:LOCALAPPDATA
if ([string]::IsNullOrWhiteSpace($Local)) { throw "LOCALAPPDATA is unavailable." }
$Root = Join-Path $Local "DragonForge"
$Targets = @(
    $Root,
    (Join-Path $Root "agent"),
    (Join-Path $Root "agent\agent-session.key"),
    (Join-Path $Root "agent\agent-runtime.json"),
    (Join-Path $Root "agent\agent.lock")
)
$BroadPrincipals = @("Everyone", "BUILTIN\Users", "Authenticated Users", "NT AUTHORITY\Authenticated Users")
$BroadMask = [System.Security.AccessControl.FileSystemRights]::Write -bor
    [System.Security.AccessControl.FileSystemRights]::Modify -bor
    [System.Security.AccessControl.FileSystemRights]::FullControl -bor
    [System.Security.AccessControl.FileSystemRights]::CreateFiles -bor
    [System.Security.AccessControl.FileSystemRights]::Delete
$Findings = 0

foreach ($Target in $Targets) {
    if (-not (Test-Path -LiteralPath $Target)) {
        Write-Host "NOT PRESENT  $Target"
        continue
    }
    $Acl = Get-Acl -LiteralPath $Target
    Write-Host ""
    Write-Host "TARGET  $Target"
    Write-Host "OWNER   $($Acl.Owner)"
    foreach ($Ace in $Acl.Access) {
        $Identity = $Ace.IdentityReference.Value
        $Broad = $BroadPrincipals -contains $Identity
        $Writable = (($Ace.FileSystemRights -band $BroadMask) -ne 0)
        if ($Ace.AccessControlType -eq "Allow" -and $Broad -and $Writable) {
            $Findings++
            Write-Warning "Broad write-capable ACE: $Identity [$($Ace.FileSystemRights)]"
        }
    }
}

if ($Findings -gt 0) {
    if ($Strict) { throw "$Findings broad write-capable ACL finding(s) detected." }
    Write-Warning "$Findings broad write-capable ACL finding(s) detected. Review before stable release."
} else {
    Write-Host "WINDOWS DATA PERMISSION REVIEW: PASS"
}
