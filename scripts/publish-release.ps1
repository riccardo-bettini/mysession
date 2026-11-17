param(
    [string]$Tag = "v2.0.0",
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
### Novità Principali di MySession v2.00 ($Tag)

- **Nuova Interfaccia Bento & Floating Pill Sidebar**:
  - Barra laterale verticale flottante a pillola con logo, indicatore circolare fluido a molla, tooltip laterale, Selettore Sessioni e Impostazioni in basso (nessuna barra superiore visibile).
  - 7 sezioni riordinabili e personalizzabili dalle Impostazioni: **Home**, **Calendario**, **Oggi**, **Corsi**, **Quiz**, **Progressione**, **AI**.
- **Piano Operativo Giornaliero & To-Do List Strutturata**:
  - Sezione **Oggi** (con switcher rapido data e espansione dal Calendario) divisa in Piano Operativo sequenziale, To-Do List filtrabile nelle **5 categorie** (`Teoria`, `Esercizi`, `Temi d'esame`, `Dimostrazioni`, `Domande orale`) con campi dedicati per Capitolo, Argomento, Pagine e Fonte, e **Quiz del giorno**.
- **Quiz Giornalieri Interattivi**:
  - Un quiz dedicato per ogni materia studiata nel giorno, con domande a scelta multipla e flashcard aperte/orali, spiegazioni dettagliate e storico tentativi/errori.
- **Corsi & Progressione**:
  - Schermate dedicate per ogni singolo corso e panoramica di avanzamento per materia, per categoria e countdown dei giorni rimanenti agli esami.
- **Agente AI & Agenti CLI Integrati**:
  - Supporto al modello locale base leggero (`Qwen2.5-VL 3B` on-demand) e collegamento diretto ad **Antigravity CLI (`agy`)**, **Claude Code (`claude`)**, **Codex (`codex`)** o CLI custom, con scansione foto orario lezioni e **Undo** immediato delle modifiche.
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
