@echo off
:: usage-radar — produce an installer (Windows)
:: Double-click to build. Output lands in src-tauri\target\release\bundle\.

setlocal
set "ROOT=%~dp0.."
pushd "%ROOT%"

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0build.ps1"
set EXIT_CODE=%ERRORLEVEL%

popd
echo.
if %EXIT_CODE% NEQ 0 (
    echo Build exited with code %EXIT_CODE%.
) else (
    echo Build finished.
)
pause
endlocal
