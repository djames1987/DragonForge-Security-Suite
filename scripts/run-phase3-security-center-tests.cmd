@echo off
setlocal
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0run-phase3-security-center-tests.ps1" %*
exit /b %ERRORLEVEL%
