param(
    [string]$InstallDirectory,
    [int]$MaxRequestsPerMinute = 120,
    [int64]$AuditMaxBytes = 1048576
)
$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
if ($env:OS -ne "Windows_NT") { throw "DragonForge Privileged Service is Windows-only." }
$Principal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $Principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw "Administrator elevation is required to install DragonForge Privileged Service."
}
$RepoRoot = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($InstallDirectory)) {
    if (Test-Path -LiteralPath (Join-Path $PSScriptRoot "dragonforge-privileged-service.exe")) {
        $InstallDirectory = $PSScriptRoot
    } else {
        $InstallDirectory = Join-Path $RepoRoot "target\\release"
    }
}
$InstallDirectory = [IO.Path]::GetFullPath($InstallDirectory)
$ServiceExe = Join-Path $InstallDirectory "dragonforge-privileged-service.exe"
$AgentExe = Join-Path $InstallDirectory "dragonforge-agent.exe"
foreach ($Path in @($ServiceExe, $AgentExe)) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) { throw "Required signed DragonForge executable is missing: $Path" }
}
$ServiceSignature = Get-AuthenticodeSignature -LiteralPath $ServiceExe
$AgentSignature = Get-AuthenticodeSignature -LiteralPath $AgentExe
foreach ($Entry in @(
    @{ Name = "privileged service"; Signature = $ServiceSignature },
    @{ Name = "Agent"; Signature = $AgentSignature }
)) {
    if ($Entry.Signature.Status -ne [System.Management.Automation.SignatureStatus]::Valid -or $null -eq $Entry.Signature.SignerCertificate) {
        throw "DragonForge $($Entry.Name) must have a valid Authenticode signature before privileged-service installation."
    }
}
$ServiceSubject = $ServiceSignature.SignerCertificate.Subject
$AgentSubject = $AgentSignature.SignerCertificate.Subject
if ($ServiceSubject -ne $AgentSubject) { throw "DragonForge Agent and privileged service must be signed by the same publisher." }
if ([string]::IsNullOrWhiteSpace($ServiceSubject) -or $ServiceSubject.Length -gt 512) { throw "DragonForge signing publisher subject is invalid." }
if ($MaxRequestsPerMinute -lt 10 -or $MaxRequestsPerMinute -gt 10000) { throw "MaxRequestsPerMinute must be between 10 and 10000." }
if ($AuditMaxBytes -lt 65536 -or $AuditMaxBytes -gt 16777216) { throw "AuditMaxBytes must be between 65536 and 16777216." }

$ServiceName = "DragonForgePrivilegedService"
$ServiceDisplayName = "DragonForge Privileged Service"
$ServiceAccount = "NT SERVICE\\$ServiceName"
$DataRoot = Join-Path $env:ProgramData "DragonForge\\Security\\privileged-service"
$ConfigPath = Join-Path $DataRoot "service-config.json"
New-Item -ItemType Directory -Force -Path $DataRoot | Out-Null
$Config = [ordered]@{
    schemaVersion = 1
    expectedPublisherSubject = $ServiceSubject
    maxRequestsPerMinute = $MaxRequestsPerMinute
    auditMaxBytes = $AuditMaxBytes
}
$Utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[IO.File]::WriteAllText($ConfigPath, ($Config | ConvertTo-Json -Depth 3), $Utf8NoBom)
$ServiceAcl = $ServiceAccount + ":(OI)(CI)M"
& icacls.exe $DataRoot /inheritance:r /grant:r "SYSTEM:(OI)(CI)F" "BUILTIN\\Administrators:(OI)(CI)F" $ServiceAcl | Out-Null
if ($LASTEXITCODE -ne 0) { throw "Failed to protect privileged-service data directory ACL." }

$Existing = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
if ($Existing) {
    if ($Existing.Status -ne "Stopped") {
        Stop-Service -Name $ServiceName -Force
        $Existing.WaitForStatus("Stopped", [TimeSpan]::FromSeconds(15))
    }
    & sc.exe delete $ServiceName | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "Unable to remove previous privileged service registration." }
    Start-Sleep -Milliseconds 750
}
$BinPath = '"' + $ServiceExe + '"'
& sc.exe create $ServiceName "binPath= $BinPath" "start= auto" "obj= $ServiceAccount" "DisplayName= $ServiceDisplayName" | Out-Null
if ($LASTEXITCODE -ne 0) { throw "Failed to register DragonForge Privileged Service." }
try {
    & sc.exe description $ServiceName "Narrow authenticated DragonForge Windows privilege boundary. No generic elevated command execution." | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "Failed to configure service description." }
    & sc.exe sidtype $ServiceName restricted | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "Failed to configure restricted service SID." }
    & sc.exe failure $ServiceName "reset= 86400" "actions= restart/5000/restart/15000/none/0" | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "Failed to configure bounded service recovery." }
    & sc.exe sdset $ServiceName "D:(A;;CCLCSWRPWPDTLOCRRC;;;SY)(A;;CCDCLCSWRPWPDTLOCRSDRCWDWO;;;BA)" | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "Failed to configure protected service DACL." }
    Start-Service -Name $ServiceName
    (Get-Service -Name $ServiceName).WaitForStatus("Running", [TimeSpan]::FromSeconds(15))
} catch {
    & sc.exe delete $ServiceName | Out-Null
    throw
}
Write-Host "DRAGONFORGE PRIVILEGED SERVICE INSTALL: PASS" -ForegroundColor Green
Write-Host "Service: $ServiceName"
Write-Host "Account: $ServiceAccount"
Write-Host "Executable: $ServiceExe"
Write-Host "Config: $ConfigPath"
Write-Host "Publisher: $ServiceSubject"
