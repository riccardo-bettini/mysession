param(
    [string]$Tag = "v1.0.43",
    [string]$Token = $env:GITHUB_TOKEN
)

if (-not $Token) {
    Write-Error "Fornisci il token GitHub tramite parametro -Token o impostando `$env:GITHUB_TOKEN."
    exit 1
}

$repo = "riccardo-bettini/mysession"

$headers = @{
    "Authorization" = "token $Token"
    "Accept" = "application/vnd.github.v3+json"
}

# Verifica se esiste già una release per questo tag
$existing = $null
try {
    $existing = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/tags/$Tag" -Headers $headers -Method Get
} catch {}

if ($existing) {
    Write-Host "Cancellazione release esistente $($existing.id)..."
    Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/$($existing.id)" -Headers $headers -Method Delete
}

$releaseNotes = @"
### Novità e Migliorie di MySession v1.0.43

- **OCR Orari Accademici ad Alta Fedeltà**: decodifica greedy deterministica (temperature 0.0) per eliminare del tutto le allucinazioni e gli orari casuali.
- **Prompt Strutturato con Ragionamento Spaziale**: istruzioni avanzate per allineare colonne giorni (1..7) e fasce orarie, leggendo con priorità l'orario scritto dentro la singola casella della lezione.
- **Normalizzazione Oraria Rigorosa**: formato 24 ore 'HH:MM' con zero iniziale (es. '08:30', '09:00', '14:30') pienamente compatibile con gli input time nativi HTML5 e il database SQLite.
- **Finestra di Contesto Espansa a 8192 token**: supporto completo a immagini ad alta risoluzione (2000-3000 token visivi) senza rischio di troncamento dei token.
- **Ottimizzatore Dimensioni Immagini**: ridimensionamento proporzionale ad alta qualità per scatti pesanti da smartphone senza saturazione della RAM/VRAM.
"@

$releasePayload = @{
    tag_name = $Tag
    target_commitish = "main"
    name = "MySession $Tag"
    body = $releaseNotes
    draft = $false
    prerelease = $false
} | ConvertTo-Json

Write-Host "Creazione GitHub Release per $Tag..."
$releaseResp = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases" -Method Post -Headers $headers -Body $releasePayload -ContentType "application/json; charset=utf-8"
$releaseId = $releaseResp.id
Write-Host "Release creata con ID: $releaseId"

$versionClean = $Tag.TrimStart('v')
$uploadUrl = "https://uploads.github.com/repos/$repo/releases/$releaseId/assets?name="

$assets = @(
    @{ Path = "release\MySession_${versionClean}_x64-setup.exe"; ContentType = "application/octet-stream"; Name = "MySession_${versionClean}_x64-setup.exe" },
    @{ Path = "release\MySession_${versionClean}_x64-setup.exe.sig"; ContentType = "text/plain"; Name = "MySession_${versionClean}_x64-setup.exe.sig" },
    @{ Path = "release\latest.json"; ContentType = "application/json"; Name = "latest.json" },
    @{ Path = "release\SHA256SUMS-${versionClean}.txt"; ContentType = "text/plain"; Name = "SHA256SUMS-${versionClean}.txt" }
)

foreach ($asset in $assets) {
    Write-Host "Caricamento $($asset.Name)..."
    $bytes = [System.IO.File]::ReadAllBytes((Resolve-Path $asset.Path))
    $uri = "$uploadUrl$($asset.Name)"
    $res = Invoke-RestMethod -Uri $uri -Method Post -Headers $headers -Body $bytes -ContentType $asset.ContentType
    Write-Host "Caricato con successo: $($res.browser_download_url)"
}

Write-Host "Pubblicazione completata!"
