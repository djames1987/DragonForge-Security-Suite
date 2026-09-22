# Phase 12.1 — Windows Installer

## Status

**Implementation Complete — Local Verification Pending**

Phase 12.1 adds a first-class Windows installer while preserving the portable package and the suite's exact-sibling executable model.

## Installer design

DragonForge uses a suite-level Inno Setup installer rather than enabling Tauri bundling on one application. Security Center, the nine sibling applications/services, and tester/support files are installed together so exact-sibling launch behavior remains unchanged.

The installer is intentionally **per-user** for this phase:

- default install path: `%LOCALAPPDATA%\Programs\DragonForge Security Suite`;
- no administrator elevation is required for the normal installation path;
- all suite executables remain in one directory;
- Start Menu shortcuts are created for Security Center, the prerequisite checker, the external-test checklist, and uninstall;
- an optional desktop shortcut can be selected;
- post-install launch targets Security Center.

## User-data preservation

Uninstall removes the installed program files managed by the installer. It does **not** intentionally delete:

- DragonForge component data under the user's local application-data locations;
- Password Manager vault files stored elsewhere;
- encrypted backup/share/vault containers stored elsewhere;
- Windows Credential Manager secrets used by Password Manager sync;
- user-selected files or recovery material.

A future explicit "remove all user data" flow must be separately designed and reviewed. Phase 12.1 does not add one.

## Upgrade behavior

The installer has a stable application ID, so later builds update the same installed product. Before an upgrade copies files, it invokes the already-installed exact-path `Stop-DragonForge-Agent.ps1` when present. Inno Setup also uses Windows application-closing support for DragonForge executables.

## WebView2

The installer checks common machine/user EdgeUpdate client registration locations for Microsoft Edge WebView2 Runtime.

If WebView2 is not detected:

- installation is still allowed;
- the installer displays an explicit prerequisite warning;
- no third-party executable is silently downloaded or executed;
- the installed `Check-Prerequisites.cmd` remains available from the Start Menu.

## Build the installer

Install Inno Setup 6 on the Windows build machine if needed:

~~~powershell
winget install --id JRSoftware.InnoSetup -e
~~~

Then:

~~~powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\package-windows-installer.ps1
~~~

The script stages the same release payload used by the portable ZIP, compiles the suite installer, writes it under `dist\`, and emits a SHA-256 sidecar.

The default development target is `v0.1.0-alpha.2`. Pass `-Version` to select another version.

## Verification

Run:

~~~powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\run-phase12.1-installer-tests.ps1
~~~

The verifier performs source checks plus a real installer compilation and writes a timestamped log and SHA-256 sidecar under `test-logs\`.
