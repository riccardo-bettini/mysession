#Requires -Version 5.1
<#
.SYNOPSIS
  Build di release stand-alone di MySession (Tauri v2 + Svelte 5) → installer NSIS (.exe) e MSI x64.

.DESCRIPTION
  1. Verifica i prerequisiti (Node.js, npm, cargo) senza percorsi cablati.
  2. Installa le dipendenze NPM (npm ci) se necessario.
  3. Esegue svelte-check e i test unitari Rust.
  4. Compila in modalità release (LTO fat, codegen-units=1, strip simboli, panic=abort).
  5. Copia gli installer in ./release con checksum SHA-256.

.EXAMPLE
  powershell -ExecutionPolicy Bypass -File scripts/build-release.ps1
  powershell -ExecutionPolicy Bypass -File scripts/build-release.ps1 -SkipTests

.NOTES
  Autore: Riccardo Bettini
#>
[CmdletBinding()]
param(
  [switch]$SkipTests,
  [switch]$SkipInstall
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot

function Write-Step([string]$msg) {
  Write-Host ""
  Write-Host "==> $msg" -ForegroundColor Cyan
}

function Invoke-Checked([string]$label, [scriptblock]$block) {
  # I tool nativi (cargo, tauri) scrivono log informativi su stderr:
  # con "Stop" PowerShell 5.1 li interpreterebbe come errori fatali.
  $prev = $ErrorActionPreference
  $ErrorActionPreference = "Continue"
  try {
    & $block
  } finally {
    $ErrorActionPreference = $prev
  }
  if ($LASTEXITCODE -ne 0) {
    throw "$label fallito (exit code $LASTEXITCODE)"
  }
}

# ---------------------------------------------------------------------------
# 1. Risoluzione dinamica della toolchain (nessun percorso utente cablato)
# ---------------------------------------------------------------------------
Write-Step "Risoluzione toolchain"

$candidatePaths = @(
  (Join-Path $env:ProgramFiles "nodejs"),
  (Join-Path $env:USERPROFILE ".cargo\bin")
)

# LLVM-MinGW opzionale (target x86_64-pc-windows-gnu) installato tramite WinGet
$wingetPkgs = Join-Path $env:LOCALAPPDATA "Microsoft\WinGet\Packages"
if (Test-Path $wingetPkgs) {
  $mingwPkg = Get-ChildItem $wingetPkgs -Directory -Filter "MartinStorsjo.LLVM-MinGW*" -ErrorAction SilentlyContinue |
    Select-Object -First 1
  if ($mingwPkg) {
    $mingwRoot = Get-ChildItem $mingwPkg.FullName -Directory -Filter "llvm-mingw-*" -ErrorAction SilentlyContinue |
      Sort-Object Name -Descending | Select-Object -First 1
    if ($mingwRoot) {
      $candidatePaths += (Join-Path $mingwRoot.FullName "bin")
      $candidatePaths += (Join-Path $mingwRoot.FullName "x86_64-w64-mingw32\bin")
    }
  }
}

foreach ($p in $candidatePaths) {
  if ((Test-Path $p) -and ($env:PATH -notlike "*$p*")) {
    $env:PATH = "$p;$env:PATH"
  }
}

foreach ($tool in @("node", "npm", "cargo")) {
  if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) {
    throw "Prerequisito mancante: '$tool' non trovato nel PATH."
  }
}

Write-Host ("node  " + (node --version))
Write-Host ("cargo " + (cargo --version))

# ---------------------------------------------------------------------------
# 2. Dipendenze NPM
# ---------------------------------------------------------------------------
if (-not $SkipInstall -and -not (Test-Path (Join-Path $ProjectRoot "node_modules"))) {
  Write-Step "Installazione dipendenze NPM (npm ci)"
  Invoke-Checked "npm ci" { npm ci }
}

# ---------------------------------------------------------------------------
# 3. Verifiche statiche e test
# ---------------------------------------------------------------------------
if (-not $SkipTests) {
  Write-Step "Type-check Svelte 5 + TypeScript strict"
  Invoke-Checked "svelte-check" { npm run check }

  Write-Step "Test unitari Core Rust"
  Invoke-Checked "cargo test" { cargo test --manifest-path src-tauri/Cargo.toml --quiet }
}

# ---------------------------------------------------------------------------
# 4. Build release + bundling NSIS / MSI
# ---------------------------------------------------------------------------
Write-Step "Compilazione release (LTO + strip) e generazione installer"
$sw = [System.Diagnostics.Stopwatch]::StartNew()
Invoke-Checked "tauri build" { npm run tauri -- build --bundles nsis,msi }
$sw.Stop()

# ---------------------------------------------------------------------------
# 5. Raccolta artefatti
# ---------------------------------------------------------------------------
Write-Step "Raccolta artefatti in ./release"

$version = (Get-Content (Join-Path $ProjectRoot "package.json") -Raw | ConvertFrom-Json).version
$targetDir = Join-Path $ProjectRoot "src-tauri\target"
$bundleDirs = Get-ChildItem $targetDir -Recurse -Directory -Filter "bundle" -ErrorAction SilentlyContinue |
  Where-Object { $_.FullName -match "\\release\\bundle$" }

$releaseDir = Join-Path $ProjectRoot "release"
New-Item -ItemType Directory -Force -Path $releaseDir | Out-Null

$artifacts = @()
foreach ($bd in $bundleDirs) {
  $artifacts += Get-ChildItem $bd.FullName -Recurse -File -Include "*.exe", "*.msi" |
    Where-Object { $_.Name -like "*$version*" }
}

if ($artifacts.Count -eq 0) {
  throw "Nessun installer trovato per la versione $version."
}

$exeBinary = Get-ChildItem $targetDir -Recurse -File -Filter "MySession.exe" -ErrorAction SilentlyContinue |
  Where-Object { $_.FullName -match "\\release\\MySession\.exe$" } | Select-Object -First 1

$hashLines = @()
foreach ($a in $artifacts) {
  $dest = Join-Path $releaseDir $a.Name
  Copy-Item $a.FullName $dest -Force
  $hash = (Get-FileHash $dest -Algorithm SHA256).Hash.ToLower()
  $hashLines += "$hash  $($a.Name)"
}
$hashLines | Set-Content -Path (Join-Path $releaseDir "SHA256SUMS-$version.txt") -Encoding ASCII

Write-Host ""
Write-Host "Build completata in $([math]::Round($sw.Elapsed.TotalSeconds, 1)) s" -ForegroundColor Green
if ($exeBinary) {
  Write-Host ("Binario MySession.exe: {0:N2} MB" -f ($exeBinary.Length / 1MB))
}
Get-ChildItem $releaseDir -File | Where-Object { $_.Name -like "*$version*" } |
  ForEach-Object { Write-Host ("  {0,-55} {1,8:N2} MB" -f $_.Name, ($_.Length / 1MB)) }
