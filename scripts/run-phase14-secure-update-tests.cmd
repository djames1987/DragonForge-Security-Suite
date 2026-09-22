@echo off
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0run-phase14-secure-update-tests.ps1"
exit /b %ERRORLEVEL%
