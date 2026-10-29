# MySession — Scheda Tecnica & Architettura di Sistema

**Autore Ufficiale:** Riccardo Bettini  
**Versione:** 1.0.11  
**Identificatore Bundle:** `com.mysession.app`  
**Piattaforma Target:** Windows 10 / 11 (64-bit Desktop)  
**Stack Tecnologico:** Tauri v2, Rust 2021, Svelte 5 (Runes), TypeScript (Strict), Tailwind CSS, SQLite WAL.

---

## 1. Visione Architetturale e Filosofia di Progetto

**MySession** è una suite desktop nativa ad alte prestazioni per la pianificazione accademica avanzata, la gestione degli esami universitari e il bilanciamento del carico di studio attraverso un calendario a scorrimento continuo (Infinite Canvas).

Il software è concepito secondo i principi del paradigma **Local-First**:
- **Zero dipendenza da cloud esterni:** Tutti i dati risiedono localmente sul dispositivo dell'utente.
- **Latenza computazionale minima (< 16ms per frame):** Rendering a 120/180 Hz privo di stuttering per monitor gaming e ProMotion.
- **Efficienza delle risorse:** Utilizzo della memoria a riposo inferiore a 50 MB di RAM.
- **Integrità ACID garantita:** Protezione contro arresti anomali o interruzioni improvvise di corrente.

---

## 2. Backend Nativo (Rust Core & Tauri v2)

Il layer di backend è scritto in Rust ed esposto al frontend mediante comandi Tauri IPC asincroni fortemente tipizzati.

### 2.1 Storage & Concorrenza (SQLite WAL)
- **Database Engine:** SQLite nativo gestito con driver `rusqlite`.
- **Modalità WAL (Write-Ahead Logging):** Consente letture e scritture simultanee ad alto throughput senza lock su tabelle intere.
- **Percorso Dati Isolato:** `%APPDATA%\com.mysession.app\database.sqlite` (nessun dato hardcoded nel codice sorgente o nella directory di compilazione).
- **Schema & Migrazioni:** Sistema di migrazioni incrementali per garantire la retrocompatibilità del database tra aggiornamenti di versione.

### 2.2 Shell Windows & Materiali DWM (Desktop Window Manager)
- **Finestra Borderless Nativizzata (`decorations: false`, `transparent: true`):**
  - Implementazione custom dei controlli finestra (riduci a icona, massimizza/ripristina, chiudi).
  - Gestione avanzata del protocollo Win32 tramite intercettazione `WM_NCHITTEST` / `HTMAXBUTTON`, abilitando il supporto completo a **Windows 11 Snap Layouts**.
- **Materiali Mica & Acrylic Dinamici:**
  - Chiamate dirette alle API DWM (`DWMWA_SYSTEMBACKDROP_TYPE`) per fondere la finestra applicativa con il tema di sistema (Chiaro, Scuro, Automatico).
- **Unified Toolbar & Drag Regions:**
  - Zona di trascinamento integrata con direttiva nativa `data-tauri-drag-region`.

### 2.3 Motori Computazionali Rust
1. **Conflict Engine:**
   - Analisi istantanea delle collisioni temporali tra blocchi di studio, lezioni, appelli d'esame e scadenze propedeutiche.
2. **Workload Engine:**
   - Modello matematico che calcola la pressione oraria e la fatica cognitiva stimata in funzione di CFU (Crediti Formativi Universitari), complessità percepita e giorni residui alla data dell'appello.
3. **AI Tool Calling Bridge:**
   - Modulo predisposto per l'orchestrazione di strumenti e comandi tramite modelli di intelligenza artificiale locali o endpoint protetti.

---

## 3. Frontend Reattivo (Svelte 5 Runes & Design System)

Il frontend è strutturato su Svelte 5 con TypeScript in modalità strict, garantendo assenza di runtime overhead superfluo.

### 3.1 Reattività con Svelte 5 Runes
- **`$state()`**: Gestione dello stato atomico per viste, calendari, sessioni ed esami.
- **`$derived()`**: Memoization automatica dei filtri su eventi, metriche del carico di lavoro e calcolo delle medie ponderate.
- **`$effect()`**: Sincronizzazione contestuale del tema, animazioni e trigger per storage e persistenza.

### 3.2 Design System Visivo (Apple Human Interface Guidelines + Windows 11)
- **Continuità di Curvatura G2 (Squircle):** Raccordi geometrici morbidi applicati su contenitori, modali e controlli interattivi.
- **Fisica delle Molle (Spring Physics):** Transizioni con smorzamento armonico sottocritico applicate a micro-interazioni e aperture di drawer/pannelli.
- **Bordi Subpixel:** Bordi rifiniti a `0.5px` con contrasto variabile per profondità percettiva ottimale sia in Dark Mode che in Light Mode.
- **Palette Colori Standard Unificata:** Insieme calibrato di 12 tinte native con calcolo automatico del contrasto (WCAG AA/AAA) e selettore cromatico HSL/Hex a 360°.

### 3.3 Moduli e Funzionalità Chiave dell'Interfaccia
- **Infinite Canvas Calendar:** Vista temporale continua senza interruzioni di pagina rigide, con virtualizzazione degli elementi per mantenere alto il framerate.
- **Command Bar Centrale (`Ctrl+K` / `Spazio`):** HUD rapido multifunzione per eseguire azioni, cambiare sessione, aggiungere compiti o cercare esami in millisecondi.
- **Gestore Sessioni Accademiche:** Visualizzazione gerarchica di anni accademici, sessioni (estiva, invernale, autunnale, straordinaria) e stati di superamento degli esami.

---

## 4. Specifiche Tecniche Riassuntive

| Componente | Dettaglio / Implementazione |
| :--- | :--- |
| **Framework Backend** | Tauri v2 (Rust 2021) |
| **Framework Frontend** | Svelte 5 (Runes Mode) |
| **Linguaggio Frontend** | TypeScript 5 (Strict Mode) |
| **Styling** | Tailwind CSS + CSS Variables custom |
| **Database** | SQLite con Write-Ahead Logging (WAL) |
| **Window Decorator** | Custom borderless + Snap Layouts Win32 |
| **Consumo RAM (Idle)** | < 50 MB |
| **Target Refresh Rate** | 120 Hz / 180 Hz nativi |
| **Privacy & Rete** | 100% Offline / Local-First |
| **Autore Ufficiale** | Riccardo Bettini |
| **Versione Attuale** | 1.0.11 |
