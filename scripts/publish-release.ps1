param(
    [string]$Tag = "v1.0.57",
    [string]$Token = $env:GITHUB_TOKEN
)

if (-not $Token) {
    try {
        $Token = (gh auth token 2>$null)
    } catch {}
}

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
### Novità e Migliorie di MySession v1.0.57

- **Sotto-elementi To-Do (Subtasks Checklist)**:
  - Ogni attività giornaliera può ora contenere una propria mini to-do list di sotto-elementi / sotto-argomenti spuntabili singolarmente.
  - **Cascata Intelligente**: spuntando tutti i sotto-elementi il task padre si completa in automatico; spuntando il padre si completano tutti i sotto-elementi; deselezionando un sotto-elemento il padre torna non completato.
  - **Interazione Fluida Apple HIG**: contatore di avanzamento con badge e chevron espandibile/collassabile, aggiunta rapida inline continua con Invio, doppio click per modificare il testo in linea e rimozione rapida.
- **Raggruppamento per Materia nell'Ispettore Giornata**:
  - Le attività della giornata sono ora suddivise in schede/box dedicati per ciascuna materia programmata nel giorno (con barra d'accento, sigla e nome completo) con contatore di completamento e input rapido per inserire nuovi task direttamente in quella materia.
  - Box dedicato separato per le attività Generali.
- **Coerenza Multivista & Arretrati**:
  - Sotto-elementi e fisarmonica interattiva integrati anche nella sezione **Arretrati da recuperare**.
  - Visualizzazione compatta della frazione di avanzamento dei sotto-elementi nella vista **Settimana** del calendario.
- **Supporto Piani di Studio v2**:
  - Compatibilità nativa con schema v1 e v2 (`subtasks` con ID stabili per non perdere progressi tra revisioni).
  - Preservazione intelligente delle spunte e dei sotto-elementi manuali dell'utente durante gli aggiornamenti del piano.
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
