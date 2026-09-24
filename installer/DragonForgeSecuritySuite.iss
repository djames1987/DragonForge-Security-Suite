#ifndef MyAppVersion
  #define MyAppVersion "1.0.0"
#endif
#ifndef StageDir
  #error StageDir must be supplied by package-windows-installer.ps1
#endif
#ifndef OutputDir
  #error OutputDir must be supplied by package-windows-installer.ps1
#endif
#ifndef BuildCommit
  #define BuildCommit "unknown"
#endif

#define MyAppName "DragonForge Security Suite"
#define MyPublisher "DragonForge"
#define MyExeName "dragonforge-security-center.exe"
#define MyAppId "{{7AFC6539-9A42-42E5-9A8F-1DB127C5AD31}"

[Setup]
AppId={#MyAppId}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyPublisher}
DefaultDirName={localappdata}\Programs\DragonForge Security Suite
DefaultGroupName=DragonForge Security Suite
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
OutputDir={#OutputDir}
OutputBaseFilename=DragonForge-Security-Suite-v{#MyAppVersion}-win-x64-setup
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
InfoBeforeFile=PHASE23-PRIVACY.txt
UninstallDisplayName={#MyAppName}
UninstallDisplayIcon={app}\{#MyExeName}
CloseApplications=yes
CloseApplicationsFilter=dragonforge-security-center.exe,dragonforge-desktop.exe,dragonforge-file-vault.exe,dragonforge-authenticator.exe,dragonforge-security-scanner.exe,dragonforge-integrity-monitor.exe,dragonforge-network-guard.exe,dragonforge-backup-recovery.exe,dragonforge-secure-share.exe,dragonforge-agent.exe
RestartApplications=no
SetupLogging=yes
UsePreviousAppDir=yes
VersionInfoVersion=1.0.0.0
VersionInfoProductName={#MyAppName}
VersionInfoProductVersion=1.0.0.0
VersionInfoProductTextVersion={#MyAppVersion}
VersionInfoCompany={#MyPublisher}
VersionInfoDescription=DragonForge Security Suite Windows installer
VersionInfoCopyright=DragonForge

[Tasks]
Name: "desktopicon"; Description: "Create a &desktop shortcut"; GroupDescription: "Additional shortcuts:"; Flags: unchecked
Name: "privilegedservice"; Description: "Install DragonForge Privileged Service (requires signed build and Administrator approval)"; GroupDescription: "Optional security boundary:"; Flags: unchecked

[Files]
Source: "{#StageDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{userprograms}\DragonForge Security Suite\DragonForge Security Center"; Filename: "{app}\{#MyExeName}"; WorkingDir: "{app}"
Name: "{userprograms}\DragonForge Security Suite\Check prerequisites"; Filename: "{app}\Check-Prerequisites.cmd"; WorkingDir: "{app}"
Name: "{userprograms}\DragonForge Security Suite\External test checklist"; Filename: "{app}\EXTERNAL-TEST-CHECKLIST.md"; WorkingDir: "{app}"
Name: "{userprograms}\DragonForge Security Suite\Install Privileged Service (Administrator)"; Filename: "{app}\install-privileged-service.cmd"; WorkingDir: "{app}"
Name: "{userprograms}\DragonForge Security Suite\Remove Privileged Service (Administrator)"; Filename: "{app}\uninstall-privileged-service.cmd"; WorkingDir: "{app}"
Name: "{userprograms}\DragonForge Security Suite\Uninstall DragonForge Security Suite"; Filename: "{uninstallexe}"
Name: "{userdesktop}\DragonForge Security Center"; Filename: "{app}\{#MyExeName}"; WorkingDir: "{app}"; Tasks: desktopicon
Name: "{userstartup}\DragonForge Agent"; Filename: "{app}\dragonforge-agent.exe"; Parameters: "--serve"; WorkingDir: "{app}"

[Run]
Filename: "{app}\install-privileged-service.cmd"; Description: "Install DragonForge Privileged Service"; WorkingDir: "{app}"; Tasks: privilegedservice; Flags: runhidden waituntilterminated
Filename: "{app}\{#MyExeName}"; Description: "Launch DragonForge Security Center"; WorkingDir: "{app}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "{sys}\WindowsPowerShell\v1.0\powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -File ""{app}\Stop-DragonForge-Agent.ps1"""; WorkingDir: "{app}"; RunOnceId: "DragonForgeAgentStop"; Flags: runhidden waituntilterminated skipifdoesntexist
Filename: "{app}\uninstall-privileged-service.cmd"; RunOnceId: "DragonForgePrivilegedServiceRemove"; Flags: runhidden waituntilterminated skipifdoesntexist

[Code]
function ContainsText(const Haystack, Needle: String): Boolean;
begin
  Result := Pos(Lowercase(Needle), Lowercase(Haystack)) > 0;
end;

function WebView2InRoot(RootKey: Integer; const BaseKey: String): Boolean;
var
  Names: TArrayOfString;
  I: Integer;
  DisplayName: String;
begin
  Result := False;
  if not RegGetSubkeyNames(RootKey, BaseKey, Names) then
    Exit;

  for I := 0 to GetArrayLength(Names) - 1 do
  begin
    DisplayName := '';
    RegQueryStringValue(RootKey, BaseKey + '\' + Names[I], 'name', DisplayName);
    if not ContainsText(DisplayName, 'WebView2') then
      RegQueryStringValue(RootKey, BaseKey + '\' + Names[I], 'DisplayName', DisplayName);
    if ContainsText(DisplayName, 'WebView2') then
    begin
      Result := True;
      Exit;
    end;
  end;
end;

function WebView2Installed(): Boolean;
begin
  Result :=
    WebView2InRoot(HKLM64, 'SOFTWARE\Microsoft\EdgeUpdate\Clients') or
    WebView2InRoot(HKLM32, 'SOFTWARE\Microsoft\EdgeUpdate\Clients') or
    WebView2InRoot(HKCU, 'SOFTWARE\Microsoft\EdgeUpdate\Clients');
end;

function InitializeSetup(): Boolean;
begin
  Result := True;
  if not WebView2Installed() then
  begin
    MsgBox(
      'Microsoft Edge WebView2 Runtime was not detected.' + #13#10 + #13#10 +
      'DragonForge can still be installed, but its desktop applications require WebView2 to run. ' +
      'Install the Microsoft Edge WebView2 Evergreen Runtime from Microsoft, then run the included prerequisite checker.',
      mbInformation,
      MB_OK);
  end;
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  ResultCode: Integer;
  StopScript: String;
begin
  Result := '';
  StopScript := ExpandConstant('{app}\Stop-DragonForge-Agent.ps1');
  if FileExists(StopScript) then
  begin
    Exec(
      ExpandConstant('{sys}\WindowsPowerShell\v1.0\powershell.exe'),
      '-NoProfile -ExecutionPolicy Bypass -File "' + StopScript + '"',
      ExpandConstant('{app}'),
      SW_HIDE,
      ewWaitUntilTerminated,
      ResultCode);
  end;
end;
