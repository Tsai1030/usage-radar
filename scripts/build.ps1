#!/usr/bin/env pwsh
# usage-radar — produce a release installer (Windows)
# Uses `tauri build --bundles all` so we don't have to mutate tauri.conf.json.

$ErrorActionPreference = "Stop"

$projectRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $projectRoot

$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if ((Test-Path "$cargoBin\cargo.exe") -and ($env:Path -notlike "*$cargoBin*")) {
    $env:Path = "$cargoBin;$env:Path"
}

Write-Host ""
Write-Host "==> usage-radar release build" -ForegroundColor Cyan
Write-Host ""

if (-not (Test-Path "node_modules")) {
    Write-Host "==> Installing frontend dependencies..."
    bun install
    if ($LASTEXITCODE -ne 0) {
        Write-Host "bun install failed" -ForegroundColor Red
        exit 1
    }
}

Write-Host "==> Building release (this can take several minutes)..." -ForegroundColor Cyan
bun run tauri build --bundles all
$exit = $LASTEXITCODE

if ($exit -ne 0) {
    Write-Host ""
    Write-Host "tauri build failed (exit $exit)" -ForegroundColor Red
    exit $exit
}

Write-Host ""
Write-Host "==> Output files:" -ForegroundColor Green
Get-ChildItem -Path "src-tauri\target\release\bundle\" -Recurse -File -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -match '\.(msi|exe|dmg|deb|AppImage|rpm)$' } |
    ForEach-Object { Write-Host "   $($_.FullName)" }
