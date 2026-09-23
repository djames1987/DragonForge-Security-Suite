@echo off
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0run-phase17-privileged-service-tests.ps1"
exit /b %ERRORLEVEL%
