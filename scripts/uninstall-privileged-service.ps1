param(
    [switch]$RemoveData
)
$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
if ($env:OS -ne "Windows_NT") { throw "DragonForge Privileged Service is Windows-only." }
$Principal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $Principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw "Administrator elevation is required to remove DragonForge Privileged Service."
}
$ServiceName = "DragonForgePrivilegedService"
$Service = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
if ($Service) {
    if ($Service.Status -ne "Stopped") {
        Stop-Service -Name $ServiceName -Force
        $Service.WaitForStatus("Stopped", [TimeSpan]::FromSeconds(15))
    }
    & sc.exe delete $ServiceName | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "Failed to delete DragonForge Privileged Service." }
}
$ProtectedRoot = Join-Path $env:ProgramFiles "DragonForge Security Suite\Privileged Service"
if (Test-Path -LiteralPath $ProtectedRoot) {
    Remove-Item -LiteralPath $ProtectedRoot -Recurse -Force
}

if ($RemoveData) {
    $DataRoot = Join-Path $env:ProgramData "DragonForge\Security\privileged-service"
    if (Test-Path -LiteralPath $DataRoot) {
        Remove-Item -LiteralPath $DataRoot -Recurse -Force
    }
}
Write-Host "DRAGONFORGE PRIVILEGED SERVICE REMOVE: PASS" -ForegroundColor Green
