@echo off
setlocal
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0run-phase20-security-center-policy-event-hub-tests.ps1"
exit /b %ERRORLEVEL%
