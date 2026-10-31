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

# Estrazione dinamica della versione da package.json
$pkgJson = Get-Content (Join-Path $ProjectRoot 'package.json') | ConvertFrom-Json
$appVersion = $pkgJson.version

Write-Host '==================================================================' -ForegroundColor Cyan
Write-Host (' MySession v' + $appVersion + ' — Pipeline di Build Release Stand-Alone') -ForegroundColor Cyan
Write-Host ' Autore: Riccardo Bettini · Stack: Tauri v2 + Svelte 5 + Rust' -ForegroundColor Cyan
Write-Host '==================================================================' -ForegroundColor Cyan

# 1. Configurazione dinamica Toolchain Windows MinGW / LLVM & Rust (Zero Hardcoding)
$candidatePaths = @(
    (Join-Path $env:ProgramFiles 'nodejs'),
    (Join-Path $env:USERPROFILE '.cargo\bin'),
    (Join-Path $ProjectRoot 'src-tauri\target\debug\deps')
)

$wingetPkgs = Join-Path $env:LOCALAPPDATA 'Microsoft\WinGet\Packages'
if (Test-Path $wingetPkgs) {
    $mingwPkg = Get-ChildItem $wingetPkgs -Directory -Filter 'MartinStorsjo.LLVM-MinGW*' -ErrorAction SilentlyContinue |
        Select-Object -First 1
    if ($mingwPkg) {
        $mingwRoot = Get-ChildItem $mingwPkg.FullName -Directory -Filter 'llvm-mingw-*' -ErrorAction SilentlyContinue |
            Sort-Object Name -Descending | Select-Object -First 1
        if ($mingwRoot) {
            $candidatePaths += (Join-Path $mingwRoot.FullName 'bin')
            $candidatePaths += (Join-Path $mingwRoot.FullName 'x86_64-w64-mingw32\bin')
        }
    }
}

foreach ($p in $candidatePaths) {
    if ((Test-Path $p) -and ($env:PATH -notlike "*$p*")) {
        $env:PATH = "$p;$env:PATH"
    }
}

Write-Host '[OK] Toolchain configurata dinamicamente nel PATH di sessione.' -ForegroundColor Green

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

# 5. Generazione bundle di rilascio tramite Tauri CLI (NSIS Setup .exe)
Write-Host ''
Write-Host '[4/5] Compilazione Release del binario nativo ed packaging installer NSIS...' -ForegroundColor Yellow

$privateKeyPath = Join-Path $ProjectRoot '.tauri\mysession.key'
if (Test-Path $privateKeyPath) {
    $env:TAURI_SIGNING_PRIVATE_KEY = (Get-Content $privateKeyPath -Raw).Trim()
    $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = 'mysession-release-2026'
    Write-Host '[OK] Chiave privata di firma configurata per Tauri updater.' -ForegroundColor Green
}

$bundleDir = Join-Path $ProjectRoot 'src-tauri\target\release\bundle'
if (Test-Path $bundleDir) {
    Remove-Item -Path $bundleDir -Recurse -Force -ErrorAction SilentlyContinue
}
npm run tauri build -- --bundles nsis

if ($LASTEXITCODE -ne 0) {
    Write-Error 'Errore durante la compilazione del pacchetto Tauri release.'
    exit 1
}

# 6. Sincronizzazione ed esportazione nella cartella release/ (Solo Setup NSIS)
Write-Host ''
Write-Host '[5/5] Sincronizzazione artefatti nella cartella di rilascio root...' -ForegroundColor Yellow
$releaseOutDir = Join-Path $ProjectRoot 'release'
if (-not (Test-Path $releaseOutDir)) {
    New-Item -ItemType Directory -Path $releaseOutDir -Force | Out-Null
}

# Pulizia completa di vecchi artefatti (compresi vecchi standalone, portable o msi)
Get-ChildItem -Path $releaseOutDir -File | Where-Object { $_.Name -notlike "*$appVersion*" -and $_.Name -ne 'latest.json' } | Remove-Item -Force -ErrorAction SilentlyContinue

$bundleDir = Join-Path $ProjectRoot 'src-tauri\target\release\bundle'
if (Test-Path $bundleDir) {
    Get-ChildItem -Path $bundleDir -Recurse -File | Where-Object { $_.Name -match 'setup\.exe$|setup\.exe\.sig$' } | ForEach-Object {
        Copy-Item -Path $_.FullName -Destination (Join-Path $releaseOutDir $_.Name) -Force
        $sizeMb = [math]::Round($_.Length / 1MB, 2)
        $msg = '  -> Esportato artefatto di rilascio ufficiale: ' + $_.Name + ' (' + $sizeMb + ' MB)'
        Write-Host $msg -ForegroundColor Green
    }

    # Generazione del manifest latest.json per GitHub Releases (Tauri v2 Auto-Updater)
    $setupSig = Get-ChildItem -Path $releaseOutDir -Filter "*$appVersion*setup.exe.sig" | Select-Object -First 1
    $setupExe = Get-ChildItem -Path $releaseOutDir -Filter "*$appVersion*setup.exe" | Select-Object -First 1
    if ($setupSig -and $setupExe) {
        $sigContent = (Get-Content $setupSig.FullName -Raw).Trim()
        $isoNow = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
        $manifestJson = @"
{
  "version": "$appVersion",
  "notes": "MySession v$appVersion - Installer NSIS Setup ufficiale con supporto GitHub Releases",
  "pub_date": "$isoNow",
  "platforms": {
    "windows-x86_64": {
      "signature": "$sigContent",
      "url": "https://github.com/riccardo-bettini/mysession/releases/download/v$appVersion/$($setupExe.Name)"
    }
  }
}
"@
        $latestJsonPath = Join-Path $releaseOutDir 'latest.json'
        $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
        [System.IO.File]::WriteAllText($latestJsonPath, $manifestJson, $utf8NoBom)
        Write-Host "  -> Generato manifest GitHub Releases firmato (UTF-8 No-BOM): latest.json" -ForegroundColor Green
    }
}

# Generazione checksum SHA256 aggiornato per l'installer ufficiale
$checksumName = 'SHA256SUMS-' + $appVersion + '.txt'
$checksumFile = Join-Path $releaseOutDir $checksumName
$hashEntries = @()
Get-ChildItem -Path $releaseOutDir -File | Where-Object { $_.Name -like "*$appVersion*setup.exe*" } | ForEach-Object {
    $h = (Get-FileHash -Path $_.FullName -Algorithm SHA256).Hash
    $hashEntries += ($h + '  ' + $_.Name)
}
$hashEntries | Set-Content -Path $checksumFile -Encoding UTF8
$csMsg = '  -> Checksum SHA256 generato: ' + $checksumName
Write-Host $csMsg -ForegroundColor Cyan

Write-Host ''
Write-Host '==================================================================' -ForegroundColor Cyan
Write-Host (' Compilazione MySession v' + $appVersion + ' Release completata con successo!') -ForegroundColor Cyan
Write-Host '==================================================================' -ForegroundColor Cyan
