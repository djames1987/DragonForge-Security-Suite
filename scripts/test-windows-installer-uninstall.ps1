param(
    [Parameter(Mandatory = $true)] [string]$Version,
    [string]$InstallerPath,
    [switch]$AllowLocal,
    [string]$StartMenuGroupName = "DragonForge Security Suite",
    [string]$StartupShortcutName = "DragonForge Agent"
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$LogDir = Join-Path $RepoRoot "test-logs"
$Stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogPath = Join-Path $LogDir "dragonforge-installer-uninstall-$Version-$Stamp.log"
$Failed = $false

if ($env:CI -ne "true" -and -not $AllowLocal) {
    throw "Installer lifecycle testing is destructive to the installed-product registration. Run on disposable CI/VM, or pass -AllowLocal knowingly."
}

if (-not $InstallerPath) {
    $InstallerPath = Join-Path $RepoRoot "dist\DragonForge-Security-Suite-v$Version-win-x64-setup.exe"
}
$InstallerPath = [IO.Path]::GetFullPath($InstallerPath)
if (-not (Test-Path -LiteralPath $InstallerPath -PathType Leaf)) {
    throw "Installer not found: $InstallerPath"
}

New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
$TestRoot = Join-Path ([IO.Path]::GetTempPath()) ("dragonforge-uninstall-" + [guid]::NewGuid().ToString("N"))
$InstallRoot = Join-Path $TestRoot "DragonForge Security Suite"
$PreserveRoot = Join-Path $TestRoot "preserved-user-data"
$PreserveSentinel = Join-Path $PreserveRoot "must-survive-uninstall.txt"
$StartMenuGroup = Join-Path $env:APPDATA ("Microsoft\Windows\Start Menu\Programs\" + $StartMenuGroupName)
$StartupShortcut = Join-Path $env:APPDATA ("Microsoft\Windows\Start Menu\Programs\Startup\" + $StartupShortcutName + ".lnk")
$AgentProcess = $null

function Require-File([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) { throw "Expected file missing: $Path" }
    Write-Host "OK  $Path"
}

try {
    Start-Transcript -Path $LogPath -Force | Out-Null
    Write-Host "DragonForge installer -> uninstaller lifecycle test"
    Write-Host "Version: $Version"
    Write-Host "Installer: $InstallerPath"
    Write-Host "Install root: $InstallRoot"

    New-Item -ItemType Directory -Force -Path $PreserveRoot | Out-Null
    Set-Content -LiteralPath $PreserveSentinel -Value "preserve-me" -Encoding ASCII

    $InstallArgs = @(
        "/VERYSILENT",
        "/SUPPRESSMSGBOXES",
        "/NORESTART",
        ('/DIR="' + $InstallRoot + '"')
    )
    $Install = Start-Process -FilePath $InstallerPath -ArgumentList $InstallArgs -Wait -PassThru
    if ($Install.ExitCode -ne 0) { throw "Installer exited with code $($Install.ExitCode)." }

    $SecurityCenter = Join-Path $InstallRoot "dragonforge-security-center.exe"
    $AgentPath = Join-Path $InstallRoot "dragonforge-agent.exe"
    $StopAgent = Join-Path $InstallRoot "Stop-DragonForge-Agent.ps1"
    $Uninstaller = Join-Path $InstallRoot "unins000.exe"
    Require-File $SecurityCenter
    Require-File $AgentPath
    Require-File $StopAgent
    Require-File $Uninstaller

    if (-not (Test-Path -LiteralPath $StartMenuGroup -PathType Container)) {
        throw "Start Menu group was not created."
    }
    Require-File $StartupShortcut

    $AgentProcess = Start-Process -FilePath $AgentPath -ArgumentList "--serve" -PassThru
    Start-Sleep -Seconds 2
    if ($AgentProcess.HasExited) { throw "Installed DragonForge Agent did not remain running for uninstall shutdown test." }
    Write-Host "Agent running before uninstall: PID $($AgentProcess.Id)"

    $UninstallArgs = @("/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART")
    $Uninstall = Start-Process -FilePath $Uninstaller -ArgumentList $UninstallArgs -Wait -PassThru
    if ($Uninstall.ExitCode -ne 0) { throw "Uninstaller exited with code $($Uninstall.ExitCode)." }

    for ($i = 0; $i -lt 20 -and (Test-Path -LiteralPath $InstallRoot); $i++) {
        Start-Sleep -Milliseconds 500
    }

    $AgentProcess.Refresh()
    if (-not $AgentProcess.HasExited) {
        throw "Exact installed DragonForge Agent was still running after uninstall."
    }
    if (Test-Path -LiteralPath $InstallRoot) {
        throw "Installer-managed application directory remains after uninstall: $InstallRoot"
    }
    if (Test-Path -LiteralPath $StartMenuGroup) {
        throw "DragonForge Start Menu group remains after uninstall."
    }
    if (Test-Path -LiteralPath $StartupShortcut) {
        throw "DragonForge Agent Startup shortcut remains after uninstall."
    }
    Require-File $PreserveSentinel

    Write-Host ""
    Write-Host "DRAGONFORGE INSTALLER/UNINSTALLER LIFECYCLE: PASS"
}
catch {
    $Failed = $true
    Write-Host ""
    Write-Host "DRAGONFORGE INSTALLER/UNINSTALLER LIFECYCLE: FAIL"
    Write-Host $_
}
finally {
    if ($AgentProcess -and -not $AgentProcess.HasExited) {
        Stop-Process -Id $AgentProcess.Id -Force -ErrorAction SilentlyContinue
    }
    try { Stop-Transcript | Out-Null } catch {}
    if (Test-Path -LiteralPath $LogPath) {
        $Hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $LogPath).Hash
        "$Hash  $([IO.Path]::GetFileName($LogPath))" | Set-Content -LiteralPath "$LogPath.sha256" -Encoding ASCII
        Write-Host "Log: $LogPath"
        Write-Host "SHA256: $Hash"
    }
    if (Test-Path -LiteralPath $TestRoot) {
        Remove-Item -LiteralPath $TestRoot -Recurse -Force -ErrorAction SilentlyContinue
    }
}

if ($Failed) { exit 1 }
