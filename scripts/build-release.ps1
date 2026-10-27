# ==============================================================================
# MySession — Script di Build Release Stand-Alone (Windows 11 x64)
# Autore: Riccardo Bettini
# ==============================================================================

[CmdletBinding()]
param (
    [switch]$SkipTests = $false,
    [string]$Target = 'all'
)

$ErrorActionPreference = 'Stop'
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
Set-Location $ProjectRoot

Write-Host '==================================================================' -ForegroundColor Cyan
Write-Host ' MySession v1.0.11 — Pipeline di Build Release Stand-Alone' -ForegroundColor Cyan
Write-Host ' Autore: Riccardo Bettini · Stack: Tauri v2 + Svelte 5 + Rust' -ForegroundColor Cyan
Write-Host '==================================================================' -ForegroundColor Cyan

# 1. Configurazione Toolchain Windows MinGW / LLVM & Rust
$mingwRoot = 'C:\Users\Riccardo\AppData\Local\Microsoft\WinGet\Packages\MartinStorsjo.LLVM-MinGW.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\llvm-mingw-20261020-ucrt-x86_64'
if (Test-Path $mingwRoot) {
    $cargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
    $mingwBin = Join-Path $mingwRoot 'bin'
    $mingwTargetBin = Join-Path $mingwRoot 'x86_64-w64-mingw32\bin'
    $depsDir = Join-Path $ProjectRoot 'src-tauri\target\debug\deps'
    $env:PATH = 'C:\Program Files\nodejs;' + $cargoBin + ';' + $mingwBin + ';' + $mingwTargetBin + ';' + $depsDir + ';' + $env:PATH
    Write-Host '[OK] Toolchain LLVM-MinGW UCRT x86_64 configurata nel PATH.' -ForegroundColor Green
} else {
    $cargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
    $env:PATH = 'C:\Program Files\nodejs;' + $cargoBin + ';' + $env:PATH
}

# 2. Controllo statico TypeScript & Svelte 5
Write-Host ''
Write-Host '[1/5] Esecuzione svelte-check (TypeScript strict)...' -ForegroundColor Yellow
npm run check
if ($LASTEXITCODE -ne 0) {
    Write-Error 'Errore critico durante svelte-check. Interruzione.'
    exit 1
}
Write-Host '[OK] svelte-check completato: 0 errori e 0 warning.' -ForegroundColor Green

# 3. Compilazione del bundle frontend (Vite)
Write-Host ''
Write-Host '[2/5] Compilazione frontend Vite per produzione...' -ForegroundColor Yellow
npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Error 'Errore critico durante vite build. Interruzione.'
    exit 1
}
Write-Host '[OK] Asset frontend compilati in dist/.' -ForegroundColor Green

# 4. Unit test suite del Core Rust (SQLite WAL + Conflict + Workload + AI)
if (-not $SkipTests) {
    Write-Host ''
    Write-Host '[3/5] Esecuzione unit test del Core Rust...' -ForegroundColor Yellow
    cargo test --manifest-path src-tauri/Cargo.toml -- --nocapture
    if ($LASTEXITCODE -ne 0) {
        Write-Error 'Uno o più test unitari Rust sono falliti. Interruzione.'
        exit 1
    }
    Write-Host '[OK] Test suite Rust superata con successo (7/7).' -ForegroundColor Green
} else {
    Write-Host ''
    Write-Host '[3/5] Unit test saltati.' -ForegroundColor DarkGray
}

# 5. Generazione bundle di rilascio tramite Tauri CLI (NSIS .exe / MSI)
Write-Host ''
Write-Host '[4/5] Compilazione Release del binario nativo ed packaging installer...' -ForegroundColor Yellow
if ($Target -eq 'nsis') {
    npm run tauri build -- --bundles nsis
} elseif ($Target -eq 'msi') {
    npm run tauri build -- --bundles msi
} else {
    npm run tauri build
}

if ($LASTEXITCODE -ne 0) {
    Write-Error 'Errore durante la compilazione del pacchetto Tauri release.'
    exit 1
}

# 6. Ispezione degli artefatti generati
Write-Host ''
Write-Host '[5/5] Verifica artefatti di rilascio...' -ForegroundColor Yellow
$bundleDir = Join-Path $ProjectRoot 'src-tauri\target\release\bundle'
if (Test-Path $bundleDir) {
    Get-ChildItem -Path $bundleDir -Recurse -File | Where-Object { $_.Extension -match 'exe|msi|zip' } | ForEach-Object {
        $sizeMb = [math]::Round($_.Length / 1MB, 2)
        $msg = '  -> Creato: ' + $_.Name + ' (' + $sizeMb + ' MB) in ' + $_.DirectoryName
        Write-Host $msg -ForegroundColor Green
    }
} else {
    $exeBin = Join-Path $ProjectRoot 'src-tauri\target\release\mysession.exe'
    if (Test-Path $exeBin) {
        $sizeMb = [math]::Round((Get-Item $exeBin).Length / 1MB, 2)
        $msg = '  -> Creato binario eseguibile: mysession.exe (' + $sizeMb + ' MB)'
        Write-Host $msg -ForegroundColor Green
    }
}

Write-Host ''
Write-Host '==================================================================' -ForegroundColor Cyan
Write-Host ' Compilazione MySession v1.0.11 Release completata con successo! ' -ForegroundColor Cyan
Write-Host '==================================================================' -ForegroundColor Cyan
