param(
    [string]$Tag = "v1.0.47",
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
### Novità e Migliorie di MySession v1.0.47

- **Inserimento Orario tramite Prompt & Dettatura Vocale**: nuova modalità interattiva "Prompt & Voce" nel modal Orario Lezioni. Permette di digitare liberamente, incollare il testo dal portale universitario (es. Esse3 o PoliMi) oppure dettare direttamente a voce con il microfono (`it-IT`). L'assistente AI (locale o cloud) estrae e compila automaticamente giorni, orari, aule e materie.
- **Scansione Completa di Blocchi Multipli e Impilati su Griglia**: perfezionamento del prompt Vision OCR per analizzare l'intera colonna di ciascun giorno da inizio mattina a fine pomeriggio, garantendo l'estrazione di tutte le lezioni consecutive o pomeridiane senza fermarsi alla prima lezione della giornata.
- **Pulsante di Navigazione Input**: aggiunto il tasto "← Modifica input" nella tabella di revisione lezioni per passare agevolmente tra testo/voce, immagine e dati estratti.
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
