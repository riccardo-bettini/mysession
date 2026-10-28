<script lang="ts">
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";
  import { APP_AUTHOR, APP_VERSION } from "../../constants/version";
  import { sessionStore } from "../../stores/session.svelte";
  import { themeStore } from "../../stores/theme.svelte";
  import type {
    AiProvider,
    BackdropMaterial,
    CalendarViewMode,
    ThemeChoice,
    WindowControlsStyle
  } from "../../types/domain";
  import ColorPickerField from "../common/ColorPickerField.svelte";

  interface Props {
    initialTab?: "appearance" | "calendar" | "ai" | "profile" | "backup";
    onClose: () => void;
    onToast?: (msg: string) => void;
  }

  let { initialTab = "appearance", onClose, onToast }: Props = $props();

  let activeTab = $state<"appearance" | "calendar" | "ai" | "profile" | "backup">(
    "appearance"
  );

  $effect(() => {
    activeTab = initialTab;
  });

  // Stato locale modificabile sincronizzato con themeStore.settings e sessionStore.profile
  let studentName = $state<string>(sessionStore.profile.studentName);
  let university = $state<string>(sessionStore.profile.university);
  let faculty = $state<string>(sessionStore.profile.faculty);
  let degreeCourse = $state<string>(sessionStore.profile.degreeCourse);
  let enrollmentYear = $state<number>(
    sessionStore.profile.enrollmentYear ?? new Date().getFullYear()
  );

  let aiProvider = $state<AiProvider>(themeStore.settings.aiProvider);
  let aiApiKey = $state<string>(themeStore.settings.aiApiKey);
  let aiModel = $state<string>(themeStore.settings.aiModel);
  let aiEndpointUrl = $state<string>(themeStore.settings.aiEndpointUrl);
  let studyHoursPerCfu = $state<number>(themeStore.settings.studyHoursPerCfu || 25);

  let accentColor = $state<string>(themeStore.settings.accentColor || "#0284C7");
  let holidayColor = $state<string>(themeStore.settings.holidayColor || "#DC2626");
  let catStudioColor = $state<string>(themeStore.settings.catStudioColor || "#0284C7");
  let catScadenzaColor = $state<string>(themeStore.settings.catScadenzaColor || "#EA580C");
  let catLezioneColor = $state<string>(themeStore.settings.catLezioneColor || "#7C3AED");
  let catPersonaleColor = $state<string>(
    themeStore.settings.catPersonaleColor || "#16A34A"
  );

  let isSaving = $state<boolean>(false);
  let backupStatus = $state<string | null>(null);
  let fileInputEl = $state<HTMLInputElement | null>(null);

  async function handleThemeSelect(theme: ThemeChoice) {
    await themeStore.setTheme(theme);
  }

  async function handleMaterialSelect(material: BackdropMaterial) {
    await themeStore.setBackdropMaterial(material);
  }

  async function handleControlsStyleSelect(style: WindowControlsStyle) {
    await themeStore.setWindowControlsStyle(style);
  }

  async function handleDefaultViewSelect(view: CalendarViewMode) {
    await themeStore.setDefaultView(view);
  }

  async function handleSaveAllSettings() {
    isSaving = true;
    try {
      await themeStore.updateSettings({
        studyHoursPerCfu: Math.max(10, Math.min(50, Math.round(studyHoursPerCfu))),
        accentColor,
        holidayColor,
        catStudioColor,
        catScadenzaColor,
        catLezioneColor,
        catPersonaleColor,
        aiProvider,
        aiApiKey: aiApiKey.trim(),
        aiModel: aiModel.trim(),
        aiEndpointUrl: aiEndpointUrl.trim()
      });

      await sessionStore.upsertProfile({
        studentName: studentName.trim() || "Studente",
        university: university.trim(),
        faculty: faculty.trim(),
        degreeCourse: degreeCourse.trim(),
        enrollmentYear: enrollmentYear || null,
        onboardingCompleted: true
      });

      await sessionStore.refreshActiveSnapshot();
      onToast?.("Impostazioni e profilo salvati con successo");
      onClose();
    } catch (e) {
      backupStatus = e instanceof Error ? e.message : String(e);
    } finally {
      isSaving = false;
    }
  }

  async function handleExportBackup() {
    backupStatus = null;
    try {
      const json = await sessionStore.exportBackupJson();
      const defaultFileName = `mysession-backup-${new Date().toISOString().slice(0, 10)}.json`;

      try {
        const targetPath = await save({
          title: "Esporta Backup Completo MySession (.json)",
          defaultPath: defaultFileName,
          filters: [{ name: "JSON Backup", extensions: ["json"] }]
        });

        if (targetPath) {
          await writeTextFile(targetPath, json);
          backupStatus = `Backup esportato correttamente in: ${targetPath}`;
          onToast?.("Backup JSON salvato su disco");
          return;
        }
      } catch {
        // Fallback download diretto nel caso in cui il file dialog nativo non sia invocabile
        const blob = new Blob([json], { type: "application/json" });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = defaultFileName;
        a.click();
        URL.revokeObjectURL(url);
        backupStatus = "Backup JSON scaricato con successo.";
      }
    } catch (e) {
      backupStatus = `Errore esportazione: ${e instanceof Error ? e.message : String(e)}`;
    }
  }

  async function handleImportBackupNative() {
    backupStatus = null;
    try {
      const selected = await open({
        title: "Importa Backup MySession (.json)",
        multiple: false,
        filters: [{ name: "JSON Backup", extensions: ["json"] }]
      });

      if (typeof selected === "string" && selected) {
        const rawJson = await readTextFile(selected);
        await sessionStore.importBackupJson(rawJson);
        backupStatus = "Workspace ripristinato con successo dal file di backup.";
        onToast?.("Backup importato con successo");
      }
    } catch {
      fileInputEl?.click();
    }
  }

  async function handleHtmlFileImport(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    try {
      const rawJson = await file.text();
      await sessionStore.importBackupJson(rawJson);
      backupStatus = "Workspace ripristinato con successo dal file selezionato.";
      onToast?.("Backup importato con successo");
    } catch (err) {
      backupStatus = `Errore importazione: ${err instanceof Error ? err.message : String(err)}`;
    } finally {
      input.value = "";
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="modal-backdrop" onclick={onClose}>
  <div
    class="settings-sheet squircle"
    onclick={(e) => e.stopPropagation()}
    role="dialog"
    aria-modal="true"
    aria-label="Impostazioni MySession"
    tabindex="-1"
  >
    <header class="sheet-header">
      <div>
        <span class="sheet-kicker">PREFERENZE DI SISTEMA & AI</span>
        <h2 class="sheet-title">Impostazioni MySession</h2>
      </div>
      <button type="button" class="close-btn pressable" onclick={onClose} aria-label="Chiudi">
        ✕
      </button>
    </header>

    <div class="settings-layout">
      <!-- Barra Laterale Tab -->
      <nav class="settings-nav">
        <button
          type="button"
          class="nav-item pressable"
          class:active={activeTab === "appearance"}
          onclick={() => (activeTab = "appearance")}
        >
          <span>🎨</span>
          <span>Aspetto & Materiali DWM</span>
        </button>
        <button
          type="button"
          class="nav-item pressable"
          class:active={activeTab === "calendar"}
          onclick={() => (activeTab = "calendar")}
        >
          <span>📅</span>
          <span>Calendario & Carico CFU</span>
        </button>
        <button
          type="button"
          class="nav-item pressable"
          class:active={activeTab === "ai"}
          onclick={() => (activeTab = "ai")}
        >
          <span>✨</span>
          <span>Assistente AI & Voce</span>
        </button>
        <button
          type="button"
          class="nav-item pressable"
          class:active={activeTab === "profile"}
          onclick={() => (activeTab = "profile")}
        >
          <span>🎓</span>
          <span>Profilo Accademico</span>
        </button>
        <button
          type="button"
          class="nav-item pressable"
          class:active={activeTab === "backup"}
          onclick={() => (activeTab = "backup")}
        >
          <span>💾</span>
          <span>Portabilità & Backup JSON</span>
        </button>
      </nav>

      <!-- Contenuto Tab -->
      <div class="settings-content">
        {#if activeTab === "appearance"}
          <div class="panel-group">
            <h3 class="group-title">Tema & Materiale Nativo Windows 11 (DWM)</h3>
            <div class="segmented-row">
              {#each [
                { id: "auto", label: "Automatico (OS)" },
                { id: "dark", label: "Scuro" },
                { id: "light", label: "Chiaro" }
              ] as item (item.id)}
                <button
                  type="button"
                  class="seg-option pressable"
                  class:active={themeStore.settings.theme === item.id}
                  onclick={() => handleThemeSelect(item.id as ThemeChoice)}
                >
                  {item.label}
                </button>
              {/each}
            </div>

            <div class="segmented-row">
              {#each [
                { id: "mica", label: "Mica (Nativo Win11)" },
                { id: "acrylic", label: "Acrylic (Vetro)" },
                { id: "tabbed", label: "Mica Alt (Tabbed)" },
                { id: "solid", label: "Solido" }
              ] as mat (mat.id)}
                <button
                  type="button"
                  class="seg-option pressable"
                  class:active={themeStore.settings.backdropMaterial === mat.id}
                  onclick={() => handleMaterialSelect(mat.id as BackdropMaterial)}
                >
                  {mat.label}
                </button>
              {/each}
            </div>
          </div>

          <div class="panel-group">
            <h3 class="group-title">Stile Controlli Finestra & Scaling Interfaccia</h3>
            <div class="segmented-row">
              <button
                type="button"
                class="seg-option pressable"
                class:active={themeStore.settings.windowControlsStyle === "macos-left"}
                onclick={() => handleControlsStyleSelect("macos-left")}
              >
                Semaforo Apple (Sinistra)
              </button>
              <button
                type="button"
                class="seg-option pressable"
                class:active={themeStore.settings.windowControlsStyle === "win11-right"}
                onclick={() => handleControlsStyleSelect("win11-right")}
              >
                Windows 11 Nativo (Destra)
              </button>
            </div>

            <div class="zoom-control-box">
              <div class="zoom-header">
                <span>Scala Fluida Interfaccia (80% – 150%)</span>
                <strong>{themeStore.settings.uiScale}%</strong>
              </div>
              <input
                type="range"
                min="80"
                max="150"
                step="5"
                value={themeStore.settings.uiScale}
                oninput={(e) =>
                  themeStore.setUiScale(Number((e.currentTarget as HTMLInputElement).value))}
              />
              <div class="zoom-presets">
                {#each [85, 100, 110, 125, 140] as pct (pct)}
                  <button
                    type="button"
                    class="preset-chip pressable"
                    class:active={themeStore.settings.uiScale === pct}
                    onclick={() => themeStore.setUiScale(pct)}
                  >
                    {pct}%
                  </button>
                {/each}
              </div>
            </div>
          </div>

          <div class="panel-group">
            <h3 class="group-title">Palette Colori Standard Unificata (WCAG AA/AAA)</h3>
            <div class="colors-grid">
              <ColorPickerField
                label="Colore d'Accento"
                value={accentColor}
                onChange={(hex) => (accentColor = hex)}
              />
              <ColorPickerField
                label="Festività Nazionali"
                value={holidayColor}
                onChange={(hex) => (holidayColor = hex)}
              />
              <ColorPickerField
                label="Categoria Studio"
                value={catStudioColor}
                onChange={(hex) => (catStudioColor = hex)}
              />
              <ColorPickerField
                label="Categoria Scadenza"
                value={catScadenzaColor}
                onChange={(hex) => (catScadenzaColor = hex)}
              />
            </div>
          </div>
        {:else if activeTab === "calendar"}
          <div class="panel-group">
            <h3 class="group-title">Vista Calendario Predefinita</h3>
            <div class="segmented-row">
              {#each [
                { id: "continuous", label: "Continua Infinita" },
                { id: "month", label: "Vista Mese" },
                { id: "week", label: "Vista Settimana" },
                { id: "agenda", label: "Agenda Appelli" }
              ] as v (v.id)}
                <button
                  type="button"
                  class="seg-option pressable"
                  class:active={themeStore.settings.defaultView === v.id}
                  onclick={() => handleDefaultViewSelect(v.id as CalendarViewMode)}
                >
                  {v.label}
                </button>
              {/each}
            </div>
          </div>

          <div class="panel-group">
            <h3 class="group-title">Motore Deterministico di Carico Studio (Workload Engine)</h3>
            <label class="field">
              <span>Ore di studio stimate per singolo CFU (default: 25 ore/CFU)</span>
              <input
                type="number"
                min="10"
                max="50"
                bind:value={studyHoursPerCfu}
              />
            </label>
            <p class="field-hint">
              Il backend Rust utilizza questo parametro insieme ai pesi di difficoltà dei corsi e
              alle Unità Didattiche rimanenti per calcolare il carico giornaliero consigliato e il
              buffer minimo di giorni nel Conflict Engine.
            </p>
          </div>
        {:else if activeTab === "ai"}
          <div class="panel-group">
            <h3 class="group-title">Architettura Disaccoppiata Assistente AI</h3>
            <p class="field-hint">
              Nessuna chiave API è precaricata nel codice sorgente. Puoi usare il motore locale
              deterministico a latenza zero (offline) oppure collegare la tua chiave API personale o
              un modello locale (Ollama / LM Studio).
            </p>

            <div class="segmented-row">
              {#each [
                { id: "local", label: "Locale Deterministico" },
                { id: "gemini", label: "Google Gemini" },
                { id: "openai", label: "OpenAI" },
                { id: "ollama", label: "Ollama / LM Studio" }
              ] as p (p.id)}
                <button
                  type="button"
                  class="seg-option pressable"
                  class:active={aiProvider === p.id}
                  onclick={() => (aiProvider = p.id as AiProvider)}
                >
                  {p.label}
                </button>
              {/each}
            </div>

            {#if aiProvider === "gemini" || aiProvider === "openai"}
              <label class="field">
                <span>API Key Personale ({aiProvider === "gemini" ? "Google AI Studio" : "OpenAI"})</span>
                <input
                  type="password"
                  bind:value={aiApiKey}
                  placeholder="Inserisci la tua API Key (salvata localmente in %APPDATA%)..."
                />
              </label>

              <label class="field">
                <span>Modello LLM</span>
                <input
                  type="text"
                  bind:value={aiModel}
                  placeholder={aiProvider === "gemini" ? "gemini-2.5-flash" : "gpt-4o-mini"}
                />
              </label>
            {:else if aiProvider === "ollama"}
              <label class="field">
                <span>Endpoint Locale Ollama / LM Studio</span>
                <input
                  type="text"
                  bind:value={aiEndpointUrl}
                  placeholder="http://localhost:11434/api/generate"
                />
              </label>
              <label class="field">
                <span>Nome Modello Locale</span>
                <input
                  type="text"
                  bind:value={aiModel}
                  placeholder="llama3.2"
                />
              </label>
            {:else}
              <div class="info-card">
                <strong>Motore NLP & Tool Calling Deterministico Attivo</strong>
                <p>
                  Esegue istantaneamente su Rust l'analisi grammaticale dei comandi in italiano,
                  inserendo corsi, appelli e To-Do, interrogando statistiche CFU/criticità e
                  modificando tema, materiali DWM e zoom senza necessità di connessione internet.
                </p>
              </div>
            {/if}
          </div>
        {:else if activeTab === "profile"}
          <div class="panel-group">
            <h3 class="group-title">Profilo Studente & Ateneo</h3>
            <div class="form-grid-2">
              <label class="field">
                <span>Nome e Cognome</span>
                <input type="text" bind:value={studentName} placeholder="Es. Marco Rossi" />
              </label>
              <label class="field">
                <span>Anno di Immatricolazione</span>
                <input type="number" min="1990" max="2040" bind:value={enrollmentYear} />
              </label>
              <label class="field">
                <span>Università / Ateneo</span>
                <input
                  type="text"
                  bind:value={university}
                  placeholder="Es. Politecnico di Milano"
                />
              </label>
              <label class="field">
                <span>Facoltà / Scuola</span>
                <input
                  type="text"
                  bind:value={faculty}
                  placeholder="Es. Ingegneria Industriale e dell'Informazione"
                />
              </label>
            </div>
            <label class="field">
              <span>Corso di Laurea</span>
              <input
                type="text"
                bind:value={degreeCourse}
                placeholder="Es. Ingegneria Meccanica / Informatica"
              />
            </label>
          </div>
        {:else if activeTab === "backup"}
          <div class="panel-group">
            <h3 class="group-title">Isolamento Dati Multi-Utente & Portabilità</h3>
            <div class="info-card">
              <div>
                <strong>Database SQLite Locale (Modalità WAL):</strong>
                <code class="db-path-code">{sessionStore.dbPath || "%APPDATA%\\com.mysession.app\\mysession.db"}</code>
              </div>
              <div>
                <strong>Versione Software:</strong> MySession v{APP_VERSION} — Autore: {APP_AUTHOR}
              </div>
            </div>

            <div class="backup-actions-row">
              <button type="button" class="action-card-btn" onclick={handleExportBackup}>
                <span class="action-title">⬇ Esporta Backup Completo (.json)</span>
                <span class="action-desc">
                  Salva tutte le sessioni, i corsi, gli appelli, le note Markdown, i To-Do e le
                  preferenze in un singolo file portabile.
                </span>
              </button>

              <button type="button" class="action-card-btn" onclick={handleImportBackupNative}>
                <span class="action-title">⬆ Importa Backup da File (.json)</span>
                <span class="action-desc">
                  Ripristina un workspace completo su questo o su un altro PC Windows 11.
                </span>
              </button>
              <input
                bind:this={fileInputEl}
                type="file"
                accept=".json,application/json"
                style="display: none"
                onchange={handleHtmlFileImport}
              />
            </div>

            {#if backupStatus}
              <div class="backup-status-banner">{backupStatus}</div>
            {/if}
          </div>
        {/if}
      </div>
    </div>

    <footer class="sheet-footer">
      <button type="button" class="secondary-btn" onclick={onClose}>Chiudi</button>
      <button
        type="button"
        class="primary-btn"
        onclick={handleSaveAllSettings}
        disabled={isSaving}
      >
        {isSaving ? "Salvataggio..." : "Salva Impostazioni"}
      </button>
    </footer>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.48);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1.5rem;
    z-index: 360;
  }

  .settings-sheet {
    width: 100%;
    max-width: 780px;
    height: min(620px, 88vh);
    background: var(--popover-bg);
    backdrop-filter: blur(28px) saturate(195%);
    -webkit-backdrop-filter: blur(28px) saturate(195%);
    border: 0.5px solid var(--hairline-border-strong);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-modal);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .sheet-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    border-bottom: 0.5px solid var(--hairline-border);
  }

  .sheet-kicker {
    font-size: 0.64rem;
    font-weight: 700;
    letter-spacing: 0.06em;
    color: var(--accent-primary);
  }

  .sheet-title {
    margin: 0.1rem 0 0;
    font-size: 1.05rem;
    font-weight: 700;
    letter-spacing: -0.022em;
  }

  .close-btn {
    width: 1.75rem;
    height: 1.75rem;
    border-radius: 999px;
    border: 0.5px solid var(--hairline-border);
    background: var(--surface-pill);
    color: var(--text-secondary);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: transform 120ms cubic-bezier(0.16, 1, 0.3, 1), background 120ms ease, color 120ms ease;
  }

  .close-btn:hover {
    background: var(--surface-card);
    color: var(--text-primary);
  }

  .close-btn:active {
    transform: scale(0.92);
  }

  .settings-layout {
    flex: 1;
    display: grid;
    grid-template-columns: 215px 1fr;
    min-height: 0;
    overflow: hidden;
  }

  .settings-nav {
    padding: 0.85rem 0.65rem;
    border-right: 0.5px solid var(--hairline-border);
    display: flex;
    flex-direction: column;
    gap: 0.28rem;
    background: rgba(128, 128, 128, 0.04);
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 0.55rem;
    padding: 0.52rem 0.7rem;
    border-radius: var(--radius-md);
    border: none;
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.76rem;
    font-weight: 600;
    text-align: left;
    cursor: pointer;
    transition: transform 120ms cubic-bezier(0.16, 1, 0.3, 1), background 120ms ease, color 120ms ease;
  }

  .nav-item:active {
    transform: scale(0.98);
  }

  .nav-item:hover {
    color: var(--text-primary);
    background: rgba(128, 128, 128, 0.09);
  }

  .nav-item.active {
    background: color-mix(in srgb, var(--accent-primary) 16%, transparent);
    color: var(--accent-primary);
  }

  .settings-content {
    padding: 1.1rem 1.35rem;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 1.15rem;
  }

  .panel-group {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
  }

  .group-title {
    margin: 0;
    font-size: 0.82rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .segmented-row {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
    gap: 0.35rem;
    padding: 0.25rem;
    border-radius: var(--radius-md);
    background: rgba(128, 128, 128, 0.1);
    border: 0.5px solid var(--hairline-border);
  }

  .seg-option {
    padding: 0.42rem 0.55rem;
    border-radius: var(--radius-xs);
    border: none;
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.73rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 140ms cubic-bezier(0.16, 1, 0.3, 1);
  }

  .seg-option:active {
    transform: scale(0.98);
  }

  .seg-option.active {
    background: var(--surface-card);
    color: var(--text-primary);
    box-shadow: 0 1.5px 5px rgba(0, 0, 0, 0.14);
  }

  .zoom-control-box {
    padding: 0.75rem 0.9rem;
    border-radius: var(--radius-md);
    border: 0.5px solid var(--hairline-border);
    background: rgba(128, 128, 128, 0.05);
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .zoom-header {
    display: flex;
    justify-content: space-between;
    font-size: 0.76rem;
    color: var(--text-secondary);
  }

  .zoom-presets {
    display: flex;
    gap: 0.35rem;
  }

  .preset-chip {
    padding: 0.2rem 0.55rem;
    border-radius: var(--radius-pill);
    border: 0.5px solid var(--hairline-border);
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.68rem;
    font-weight: 600;
    cursor: pointer;
  }

  .preset-chip.active {
    background: var(--accent-primary);
    border-color: var(--accent-primary);
    color: #fff;
  }

  .colors-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
  }

  .form-grid-2 {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    font-size: 0.74rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .field input {
    padding: 0.48rem 0.68rem;
    border-radius: var(--radius-md);
    border: 0.5px solid var(--hairline-border);
    background: var(--surface-input, rgba(128, 128, 128, 0.08));
    color: var(--text-primary);
    font-size: 0.8rem;
    outline: none;
    transition: border-color 140ms ease, box-shadow 140ms ease;
  }

  .field input:focus-visible {
    border-color: var(--accent-primary);
    box-shadow: 0 0 0 2.5px var(--accent-soft);
  }

  .field-hint {
    margin: 0;
    font-size: 0.72rem;
    line-height: 1.45;
    color: var(--text-muted);
  }

  .info-card {
    padding: 0.75rem 0.9rem;
    border-radius: var(--radius-md);
    border: 0.5px solid var(--hairline-border);
    background: rgba(128, 128, 128, 0.06);
    font-size: 0.75rem;
    line-height: 1.5;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .info-card p {
    margin: 0;
    color: var(--text-secondary);
  }

  .db-path-code {
    display: block;
    margin-top: 0.2rem;
    padding: 0.3rem 0.5rem;
    border-radius: 6px;
    background: rgba(0, 0, 0, 0.2);
    font-family: var(--font-mono);
    font-size: 0.68rem;
    word-break: break-all;
  }

  .backup-actions-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
  }

  .action-card-btn {
    padding: 0.85rem;
    border-radius: var(--radius-lg);
    border: 0.5px solid var(--hairline-border-strong);
    background: rgba(128, 128, 128, 0.07);
    text-align: left;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    cursor: pointer;
    transition: all 140ms ease;
  }

  .action-card-btn:hover {
    background: rgba(128, 128, 128, 0.14);
    border-color: var(--accent-primary);
  }

  .action-title {
    font-size: 0.8rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .action-desc {
    font-size: 0.7rem;
    line-height: 1.4;
    color: var(--text-muted);
  }

  .backup-status-banner {
    padding: 0.55rem 0.75rem;
    border-radius: var(--radius-md);
    background: rgba(16, 185, 129, 0.14);
    border: 0.5px solid rgba(16, 185, 129, 0.4);
    color: #34d399;
    font-size: 0.74rem;
  }

  .sheet-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 0.55rem;
    padding: 0.85rem 1.25rem;
    border-top: 0.5px solid var(--hairline-border);
  }

  .secondary-btn {
    padding: 0.45rem 0.9rem;
    border-radius: var(--radius-md);
    border: 0.5px solid var(--hairline-border);
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.78rem;
    font-weight: 600;
    cursor: pointer;
  }

  .primary-btn {
    padding: 0.45rem 1.05rem;
    border-radius: var(--radius-md);
    border: none;
    background: var(--accent-primary);
    color: #fff;
    font-size: 0.78rem;
    font-weight: 600;
    cursor: pointer;
  }
</style>
