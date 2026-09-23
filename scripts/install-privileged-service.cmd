@echo off
powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "$p=Start-Process powershell.exe -Verb RunAs -Wait -PassThru -ArgumentList '-NoProfile','-ExecutionPolicy','Bypass','-File','""%~dp0install-privileged-service.ps1""'; exit $p.ExitCode"
exit /b %ERRORLEVEL%
