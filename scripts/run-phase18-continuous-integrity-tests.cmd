@echo off
setlocal
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0run-phase18-continuous-integrity-tests.ps1"
exit /b %ERRORLEVEL%
