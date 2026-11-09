param(
    [string]$Tag = "v1.0.50",
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
### Novità e Migliorie di MySession v1.0.50

- **Ristrutturazione Toolbar Superiore**: rimosso dalla barra il pulsante modale dell'orario per massima pulizia visiva (l'orario settimanale resta sempre accessibile dal menu a tendina della sessione). Preservato il comodo toggle rapido dell'orologio nel calendario.
- **Compattazione Dinamica Responsive**:
  - **Tasto Conflitti**: passa automaticamente a pillola compatta con badge numerico (🔴 N / 🙈 N) e tooltip informativo esteso su hover quando lo spazio si riduce.
  - **Blocco Sessione**: nasconde l'etichetta testuale mantenendo l'icona del lucchetto animato con fisica Apple HIG a molla e indicatore cromatico.
  - **Selettore Viste Calendario**: passa a 4 icone dedicate ad alta definizione SVG (Infinito ∞, Mese, Settimana, Agenda) con tooltip dedicato su ciascuna vista.
- **Scorrimento Ciclico con Rotella del Mouse**: posizionando il cursore sopra il selettore di sessione, la rotella del mouse permette di scorrere istantaneamente e ciclicamente tra tutte le sessioni configurate (con throttling a 150ms anti-scatto).
- **Ridimensionamento Fluido del Selettore di Sessione**: il titolo della sessione si adatta con troncamento intelligente (...), nascondendo prima l'anno accademico in schermi ridotti.
- **Area di Drag Nativa Garantita**: riservati sempre almeno 48px di spazio libero centrale con `data-tauri-drag-region`, permettendo il trascinamento fluido della finestra a qualsiasi risoluzione.
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
