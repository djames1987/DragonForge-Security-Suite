@echo off
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0run-phase16-windows-security-boundary-tests.ps1"
exit /b %ERRORLEVEL%
