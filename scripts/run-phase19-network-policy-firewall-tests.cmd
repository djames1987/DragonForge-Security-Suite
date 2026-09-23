@echo off
setlocal
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0run-phase19-network-policy-firewall-tests.ps1"
exit /b %ERRORLEVEL%
