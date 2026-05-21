@echo off
:: usage-radar — one-click launcher (Windows)
:: Double-click this file from File Explorer.
:: Wraps scripts\start.ps1 with PowerShell ExecutionPolicy Bypass so
:: no policy / signing setup is required.

setlocal
cd /d "%~dp0"

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\start.ps1"
set EXIT_CODE=%ERRORLEVEL%

echo.
if %EXIT_CODE% NEQ 0 (
    echo Launcher exited with code %EXIT_CODE%.
) else (
    echo Launcher finished.
)
pause
endlocal
