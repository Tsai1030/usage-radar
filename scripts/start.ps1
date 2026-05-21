#!/usr/bin/env pwsh
# usage-radar — one-click developer launcher (Windows)
# Checks prerequisites, installs dependencies if needed, then runs the app.

$ErrorActionPreference = "Stop"

# Run from the project root no matter where the script is invoked from
$projectRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $projectRoot

Write-Host ""
Write-Host "==> usage-radar launcher" -ForegroundColor Cyan
Write-Host ""

# Fix common PATH gotcha: Rust is installed but the shell predates the install
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if ((Test-Path "$cargoBin\cargo.exe") -and ($env:Path -notlike "*$cargoBin*")) {
    $env:Path = "$cargoBin;$env:Path"
}

function Test-Tool {
    param([string]$Cmd, [string]$InstallHint)
    $exe = Get-Command $Cmd -ErrorAction SilentlyContinue
    if ($exe) {
        $ver = & $Cmd --version 2>&1 | Select-Object -First 1
        Write-Host ("  [OK]      {0,-6} {1}" -f $Cmd, $ver) -ForegroundColor Green
        return $true
    } else {
        Write-Host ("  [MISSING] {0,-6} (install: {1})" -f $Cmd, $InstallHint) -ForegroundColor Red
        return $false
    }
}

Write-Host "Checking prerequisites..."
$cargoOk = Test-Tool "cargo" "winget install --id Rustlang.Rustup"
$bunOk   = Test-Tool "bun"   "winget install --id Oven-sh.Bun"

if (-not ($cargoOk -and $bunOk)) {
    Write-Host ""
    Write-Host "Please install the missing tools, OPEN A NEW TERMINAL, then re-run this script." -ForegroundColor Yellow
    exit 1
}

Write-Host ""

if (-not (Test-Path "node_modules")) {
    Write-Host "==> Installing frontend dependencies (bun install)..." -ForegroundColor Cyan
    bun install
    if ($LASTEXITCODE -ne 0) { Write-Host "bun install failed" -ForegroundColor Red; exit 1 }
} else {
    Write-Host "  node_modules present, skipping install" -ForegroundColor DarkGray
}

Write-Host ""
Write-Host "==> Launching usage-radar (Ctrl+C to stop)" -ForegroundColor Cyan
Write-Host "    First run compiles Rust crates — may take 5-10 minutes." -ForegroundColor DarkGray
Write-Host ""

bun run tauri dev
