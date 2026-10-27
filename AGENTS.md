# Regole di Sviluppo per Agenti AI — Progetto MySession

Questo documento stabilisce le regole operative, architetturali e di design per tutti gli agenti AI che collaborano allo sviluppo del progetto **MySession** (Gestione Sessioni di Studio & Calendario Esami Polimi).

---

## 1. Regola di Versionamento Incrementale (Obbligatoria)

- **Numero di Versione di Partenza**: `1.0.00`.
- **Incremento di +0.0.01 per ogni modifica**: Ad ogni iterazione, correzione di bug, rifattorizzazione o aggiunta di funzionalità, la versione deve essere incrementata di **0.0.01** (es. `1.0.00` → `1.0.01` → `1.0.02`...).
- **Sincronizzazione Multi-file**: Ogni incremento di versione deve essere sincronizzato contestualmente in:
  1. **[index.html](file:///C:/Users/Riccardo/Personal/Antigravity/app_programmazione_sessione/index.html)**: Nel badge della barra laterale sinistra del modal Impostazioni:
     ```html
     <span class="settings-app-version">v1.0.XX</span>
     ```
  2. **[app.js](file:///C:/Users/Riccardo/Personal/Antigravity/app_programmazione_sessione/app.js)**: Nella costante all'inizio del file:
     ```javascript
     const APP_VERSION = "1.0.XX";
     ```
  3. **[Launcher.cs](file:///C:/Users/Riccardo/Personal/Antigravity/app_programmazione_sessione/Launcher.cs)**: Negli attributi assembly:
     ```csharp
     [assembly: AssemblyVersion("1.0.X.0")]
     [assembly: AssemblyFileVersion("1.0.X.0")]
     ```
  4. **[MySession.exe](file:///C:/Users/Riccardo/Personal/Antigravity/app_programmazione_sessione/MySession.exe)**: Ricompilare sempre l'eseguibile con `csc.exe` quando si modifica `Launcher.cs`.
- **Autore Ufficiale**: L'autore accreditato del progetto è **Riccardo Bettini** (`APP_AUTHOR = "Riccardo Bettini"`).

---

## 2. Directory di Lavoro & Integrità del Workspace

- **Percorso Assoluto di Progetto**:
  `C:\Users\Riccardo\Personal\Antigravity\app_programmazione_sessione`
- **Divieto di Usare Cartelle Obsolete**:
  Non creare, spostare o ripristinare mai cartelle obsolete precedentemente rimosse (in particolare `app_programmazine_sessione` con refuso o `programma_sessione_inv_2027`).
- **Esecuzione Comandi PowerShell**:
  Ogni comando PowerShell che interagisce con il file system, la compilazione o git deve posizionarsi esplicitamente nella cartella corretta:
  ```powershell
  Set-Location "C:\Users\Riccardo\Personal\Antigravity\app_programmazione_sessione"
  ```

---

## 3. Gestione e Igiene del Version Control (Git)

- Eseguire un commit semantico e mirato al completamento di ogni richiesta dell'utente (formato: `feat:`, `fix:`, `style:`, `refactor:`, `perf:`).
- Prima di ogni commit, verificare lo stato con `git status` per evitare di includere file non tracciati o temporanei indesiderati.
- Mantenere la storia lineare e pulita sul branch principale `main`.

---

## 4. Architettura della Finestra Desktop & Launcher C# (Anti-Doppia Barra)

- **Cornice Personalizzata (`frame_style = "custom"`)**:
  - La finestra dell'applicazione browser Edge/Chrome viene eseguita in modalità `--app` ed è controllata da [Launcher.cs](file:///C:/Users/Riccardo/Personal/Antigravity/app_programmazione_sessione/Launcher.cs) tramite P/Invoke 64-bit (`GetWindowLongPtr` / `SetWindowLongPtr`).
  - La barra nativa di Windows (`WS_CAPTION = 0x00C00000`) deve rimanere **completamente rimossa**, preservando `WS_THICKFRAME` per garantire lo Snap ai bordi di Windows 11 (`Win + Freccia`) e il ridimensionamento continuo.
  - La sola barra visibile in alto deve essere la barra personalizzata HTML (`#customAppTitlebar`).
  - Il thread di monitoraggio in background (`MonitorWindowLoop`) in `Launcher.cs` deve restare attivo per riapplicare la rimozione qualora Chromium tenti di ripristinare `WS_CAPTION` in fase di rendering.
- **Cornice Standard (`frame_style = "standard"`)**:
  - Ripristina la barra del titolo classica di sistema e nasconde con classe `.hidden` la barra personalizzata HTML per evitare duplicazioni.

---

## 5. Design System Visivo (Apple HIG + Windows 11 Mica)

- **Palette Colori Standard Unificata**:
  Tutti i selettori di colore (Accento UI, Festività, Domenica, Categorie Studio/Scadenza/Lezione/Personale e Corsi MAM/TM/MSE/DCM/PSM) devono condividere la stessa palette di 12 colori standard predefiniti (`UNIFIED_STANDARD_PALETTE`), oltre allo slider di tonalità a 360° e al picker esadecimale.
- **Token CSS Standard**:
  Utilizzare sempre le variabili CSS definite in [styles.css](file:///C:/Users/Riccardo/Personal/Antigravity/app_programmazione_sessione/styles.css):
  - Curve di animazione: `--ease-out`, `--spring-snappy`, `--dur-quick`, `--dur-normal`.
  - Bordi e raccordi: continuous squircles Apple (`--radius-sm`, `--radius-md`, `--radius-lg`, `--radius-pill`).
  - Superfici vitree/Mica: `--glass-sidebar-bg`, `--glass-blur`, `--border-hairline`.
- **Preservazione dei Commenti**:
  Mantenere intatti tutti i commenti e docstring esistenti nel codice per non degradare la documentazione tecnica.

---

## 6. Persistenza Locale & Protezione Dati (LocalStorage)

- Lo stato dell'applicazione è gestito in `state` e serializzato su `localStorage` (sessioni, esami, impegni, preferenze di tema, scala zoom e colori).
- Utilizzare sempre deserializzazione difensiva (`try/catch` su `JSON.parse`) e valori di fallback sicuri per evitare che dati corrotti blocchino l'avvio.
- Non cancellare o sovrascrivere mai i dati utente senza un'esplicita azione o conferma da interfaccia.

---

## 7. Performance & Architettura Zero-Dependencies

- Nessun package manager o runtime pesante (niente npm/node.js obbligatorio a runtime): l'app deve funzionare in modo ultraleggero con HTML5 standard, CSS3 puro, Vanilla JS (ES6+) e il compilatore nativo `C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe`.
- Budget di animazione: mantenere frame rate fluido a 60/120 fps sfruttando accelerazione hardware GPU (`transform: translate3d`, `opacity`) ed evitando layout thrashing.

---

## 8. Stile di Risposta e Comunicazione dell'Agente

- Rispondere in modo sintetico, chiaro e professionale in lingua italiana.
- Includere sempre link cliccabili a tutti i file e simboli modificati con schema `file:///` (es. `[app.js](file:///C:/Users/Riccardo/Personal/Antigravity/app_programmazione_sessione/app.js)`).
