#!/usr/bin/env pwsh
# usage-radar — produce a release installer (Windows)

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
    if ($LASTEXITCODE -ne 0) { Write-Host "bun install failed" -ForegroundColor Red; exit 1 }
}

# Bundle must be enabled for tauri to produce installers
$confPath = "src-tauri\tauri.conf.json"
$conf = Get-Content $confPath -Raw | ConvertFrom-Json
$originalActive = $conf.bundle.active
if (-not $originalActive) {
    Write-Host "==> Temporarily enabling bundle.active in tauri.conf.json" -ForegroundColor DarkGray
    $conf.bundle.active = $true
    $conf | ConvertTo-Json -Depth 32 | Set-Content $confPath -Encoding utf8
}

try {
    Write-Host "==> Building release (this can take several minutes)..." -ForegroundColor Cyan
    bun run tauri build
    if ($LASTEXITCODE -ne 0) { throw "tauri build failed" }

    Write-Host ""
    Write-Host "==> Output files:" -ForegroundColor Green
    Get-ChildItem -Path "src-tauri\target\release\bundle\" -Recurse -File -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -match '\.(msi|exe|dmg|deb|AppImage)$' } |
        ForEach-Object { Write-Host "   $($_.FullName)" }
} finally {
    if (-not $originalActive) {
        Write-Host ""
        Write-Host "==> Restoring bundle.active = false in tauri.conf.json" -ForegroundColor DarkGray
        $conf.bundle.active = $false
        $conf | ConvertTo-Json -Depth 32 | Set-Content $confPath -Encoding utf8
    }
}
