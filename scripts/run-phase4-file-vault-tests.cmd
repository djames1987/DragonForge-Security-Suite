@echo off
setlocal
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0run-phase4-file-vault-tests.ps1" %*
exit /b %ERRORLEVEL%
