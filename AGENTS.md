# Regole di Sviluppo per Agenti AI — Progetto MySession (Tauri v2 + Svelte 5 + Rust)

Questo documento stabilisce le regole operative, architetturali e di design per tutti gli agenti AI che collaborano allo sviluppo del progetto **MySession** (Suite Desktop Nativa per la Gestione delle Sessioni Accademiche & Calendario Infinito).

---

## 1. Regola di Versionamento Incrementale (Obbligatoria)

- **Numero di Versione di Partenza**: `1.0.00` (Attuale: `1.0.41`).
- **Incremento di +0.0.01 per ogni modifica**: Ad ogni iterazione, fase, correzione di bug, rifattorizzazione o aggiunta di funzionalità, la versione deve essere incrementata di **0.0.01** (es. `1.0.04` → `1.0.05` → `1.0.06`...).
- **Sincronizzazione Multi-file**: Ogni incremento di versione deve essere sincronizzato contestualmente in:
  1. **[package.json](file:///C:/Users/Riccardo/Personal/Antigravity/app_programmazione_sessione/package.json)**: Nel campo `"version"`.
  2. **[src-tauri/Cargo.toml](file:///C:/Users/Riccardo/Personal/Antigravity/app_programmazione_sessione/src-tauri/Cargo.toml)**: Nel campo `version` del package Rust (formato SemVer compatibile `1.0.X`).
  3. **[src-tauri/tauri.conf.json](file:///C:/Users/Riccardo/Personal/Antigravity/app_programmazione_sessione/src-tauri/tauri.conf.json)**: Nel campo `"version"`.
  4. **[src/lib/constants/version.ts](file:///C:/Users/Riccardo/Personal/Antigravity/app_programmazione_sessione/src/lib/constants/version.ts)**: Nella costante `APP_VERSION = "1.0.XX"`.
- **Autore Ufficiale del Software**: **Riccardo Bettini** (`APP_AUTHOR = "Riccardo Bettini"`).

---

## 2. Directory di Lavoro & Integrità del Workspace (Zero-Legacy & Zero-Hardcoding)

- **Percorso Assoluto di Progetto**:
  `C:\Users\Riccardo\Personal\Antigravity\app_programmazione_sessione`
- **Struttura Pulita Tauri v2 + Svelte 5**:
  - `src-tauri/`: Core backend Rust (SQLite WAL, DWM Win32 Mica/Acrylic, Conflict Engine, Workload Engine, AI Tool Calling).
  - `src/`: Frontend Svelte 5 (Runes) + TypeScript strict + Design System Apple HIG.
- **Divieto Assoluto di Hardcoding**:
  Non inserire mai percorsi locali cablati o dati utente precaricati nel codice sorgente. Tutti i dati risiedono nel database SQLite isolato in `%APPDATA%\com.mysession.app\`.
- **Esecuzione Comandi PowerShell**:
  Ogni comando PowerShell deve posizionarsi esplicitamente nella cartella corretta:
  ```powershell
  Set-Location "C:\Users\Riccardo\Personal\Antigravity\app_programmazione_sessione"
  ```

---

## 3. Gestione e Igiene del Version Control (Git)

- Eseguire un commit semantico e mirato al completamento di ogni fase o richiesta dell'utente (`feat:`, `fix:`, `style:`, `refactor:`, `perf:`).
- Prima di ogni commit, verificare lo stato con `git status` per evitare di includere file non tracciati o temporanei indesiderati.
- Mantenere la storia lineare e pulita sul branch principale `main`.

---

## 4. Architettura della Finestra Desktop & Materiali DWM (Rust + Win32)

- **Finestra Borderless Nativizzata (`decorations: false`, `transparent: true`)**:
  - Supporto completo a **Windows 11 Snap Layouts** (intercettando `WM_NCHITTEST` / `HTMAXBUTTON`), ombre native DWM e animazioni di riduzione a icona.
  - Iniezione a livello di sistema di `DWMWA_SYSTEMBACKDROP_TYPE` (Mica / Acrylic) sincronizzata con il tema Chiaro, Scuro e Automatico di Windows.
  - **Unified Toolbar**: Barra superiore custom con controlli finestra (stile semaforo Apple a sinistra o Windows 11 a destra), selettore rapido sessione, Command Bar centrale a pillola (`Ctrl+K` / `Spazio`) e area di trascinamento `data-tauri-drag-region`.

---

## 5. Design System Visivo (Apple HIG + Rendering a 120/180 Hz)

- **Palette Colori Standard Unificata**:
  Tutti i selettori di colore condividono la palette di 12 colori standard (`UNIFIED_STANDARD_PALETTE`), lo slider di tonalità a 360°, l'input Hex e la generazione automatica di tinte ad alto contrasto (WCAG AA/AAA).
- **Geometria e Fisica**:
  Raccordi con continuità di curvatura $G2$ (stile squircle), bordi subpixel da `0.5px` e transizioni basate esclusivamente su fisica delle molle (smorzamento armonico sottocritico) senza curve lineari, garantendo zero stuttering a $120/180\text{ Hz}$ e $< 50\text{ MB}$ di RAM a riposo.

---

## 6. Stile di Risposta e Comunicazione dell'Agente

- Rispondere in modo sintetico, chiaro e professionale in lingua italiana.
- Includere sempre link cliccabili a tutti i file e simboli modificati con schema `file:///` (es. `[App.svelte](file:///C:/Users/Riccardo/Personal/Antigravity/app_programmazione_sessione/src/App.svelte)`).

---

## 7. Aggiornamento Automatico della Cartella Release (Obbligatorio)

- **Generazione Pipeline di Release**:
  Ad ogni iterazione, modifica o rilascio di versione, è **obbligatorio** aggiornare la cartella `release/` eseguendo lo script di packaging automatizzato:
  ```powershell
  Set-Location "C:\Users\Riccardo\Personal\Antigravity\app_programmazione_sessione"
  npm run release
  ```
- **Artefatti Generati nella cartella `release/`**:
  - Installer Ufficiale NSIS Setup (`MySession_<version>_x64-setup.exe`)
  - Firma di sicurezza auto-updater (`MySession_<version>_x64-setup.exe.sig`)
  - Manifest GitHub Releases (`latest.json`)
  - Tabella Checksum SHA256 (`SHA256SUMS-<version>.txt`)

