<script lang="ts">
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";
  import { APP_AUTHOR, APP_VERSION } from "../../constants/version";
  import { sessionStore } from "../../stores/session.svelte";
  import { themeStore } from "../../stores/theme.svelte";
  import { planStore } from "../../stores/plan.svelte";
  import { updaterStore } from "../../stores/updater.svelte";
  import type {
    AiProvider,
    BackdropMaterial,
    CalendarViewMode,
    ThemeChoice,
    WindowControlsStyle
  } from "../../types/domain";
  import ColorPickerField from "../common/ColorPickerField.svelte";

  export type SettingsTab =
    | "appearance"
    | "colors"
    | "calendar"
    | "workload"
    | "plan"
    | "ai"
    | "profile"
    | "backup"
    | "updates";

  interface Props {
    initialTab?: SettingsTab;
    onClose: () => void;
    onToast?: (msg: string) => void;
  }

  let { initialTab = "appearance", onClose, onToast }: Props = $props();

  let activeTab = $state<SettingsTab>("appearance");

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
  let startDayOfWeek = $state<number>(themeStore.settings.startDayOfWeek ?? 1);
  let weeklyHolidayDay = $state<number>(themeStore.settings.weeklyHolidayDay ?? 0);

  let accentColor = $state<string>(themeStore.settings.accentColor || "#0284C7");
  let holidayColor = $state<string>(themeStore.settings.holidayColor || "#DC2626");
  let sundayColor = $state<string>(themeStore.settings.sundayColor || "#EA580C");
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
        startDayOfWeek,
        weeklyHolidayDay,
        studyHoursPerCfu: Math.max(10, Math.min(50, Math.round(studyHoursPerCfu))),
        accentColor,
        holidayColor,
        sundayColor,
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
    <!-- Header del Pannello Impostazioni -->
    <header class="sheet-header">
      <div>
        <span class="sheet-kicker">PREFERENZE DI SISTEMA & ARCHITETTURA</span>
        <h2 class="sheet-title">Impostazioni MySession</h2>
      </div>
      <button type="button" class="close-btn pressable" onclick={onClose} aria-label="Chiudi">
        ✕
      </button>
    </header>

    <!-- Layout Impostazioni: Barra Laterale Sinistra + Area Contenuti -->
    <div class="settings-layout">
      <!-- Barra Laterale Sinistra con Categorie Dedicate & Footer Autore/Versione -->
      <nav class="settings-nav" aria-label="Navigazione sezioni impostazioni">
        <div class="nav-items-scroll">
          <div class="nav-group-header">INTERFACCIA</div>
          <button
            type="button"
            class="nav-item pressable"
            class:active={activeTab === "appearance"}
            onclick={() => (activeTab = "appearance")}
          >
            <span class="nav-icon">🎨</span>
            <span>Aspetto & Finestra</span>
          </button>
          <button
            type="button"
            class="nav-item pressable"
            class:active={activeTab === "colors"}
            onclick={() => (activeTab = "colors")}
          >
            <span class="nav-icon">🌈</span>
            <span>Palette & Colori</span>
          </button>

          <div class="nav-group-header">PIANIFICAZIONE</div>
          <button
            type="button"
            class="nav-item pressable"
            class:active={activeTab === "calendar"}
            onclick={() => (activeTab = "calendar")}
          >
            <span class="nav-icon">📅</span>
            <span>Calendario & Viste</span>
          </button>
          <button
            type="button"
            class="nav-item pressable"
            class:active={activeTab === "workload"}
            onclick={() => (activeTab = "workload")}
          >
            <span class="nav-icon">⚡</span>
            <span>Algoritmo CFU & Carico</span>
          </button>
          <button
            type="button"
            class="nav-item pressable"
            class:active={activeTab === "plan"}
            onclick={() => (activeTab = "plan")}
          >
            <span class="nav-icon">📁</span>
            <span>Piani di Studio & Builder</span>
          </button>

          <div class="nav-group-header">INTELLIGENZA & DATI</div>
          <button
            type="button"
            class="nav-item pressable"
            class:active={activeTab === "ai"}
            onclick={() => (activeTab = "ai")}
          >
            <span class="nav-icon">✨</span>
            <span>Assistente AI & NLP</span>
          </button>
          <button
            type="button"
            class="nav-item pressable"
            class:active={activeTab === "profile"}
            onclick={() => (activeTab = "profile")}
          >
            <span class="nav-icon">🎓</span>
            <span>Profilo Accademico</span>
          </button>
          <button
            type="button"
            class="nav-item pressable"
            class:active={activeTab === "backup"}
            onclick={() => (activeTab = "backup")}
          >
            <span class="nav-icon">💾</span>
            <span>Dati & Backup JSON</span>
          </button>
          <button
            type="button"
            class="nav-item pressable"
            class:active={activeTab === "updates"}
            onclick={() => (activeTab = "updates")}
          >
            <span class="nav-icon">🚀</span>
            <span>Aggiornamenti</span>
            {#if updaterStore.status === "available"}
              <span class="nav-update-dot"></span>
            {/if}
          </button>
        </div>

        <!-- Sezione Dedicata in Basso alla Barra Laterale Sinistra (Autore & Versione) -->
        <div class="settings-nav-footer">
          <div class="nav-brand-row">
            <span class="nav-brand-title">MySession</span>
            <span class="nav-version-badge">v{APP_VERSION}</span>
          </div>
          <div class="nav-author-row">
            <span class="nav-author-label">Autore:</span>
            <span class="nav-author-name">{APP_AUTHOR}</span>
          </div>
        </div>
      </nav>

      <!-- Contenuto della Sezione Attiva -->
      <div class="settings-content">
        <!-- 1. ASPETTO & FINESTRA -->
        {#if activeTab === "appearance"}
          <div class="panel-group">
            <h3 class="group-title">Tema di Sistema (Sincronizzazione Windows 11)</h3>
            <p class="field-hint">Adatta automaticamente la palette al tema Chiaro/Scuro di Windows oppure forza una modalità fissa.</p>
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
          </div>

          <div class="panel-group">
            <h3 class="group-title">Materiale DWM Nativo Windows 11 (Vibrancy & Mica)</h3>
            <p class="field-hint">Iniezione di sistema del materiale di sfondo DWM tramite Win32 DwmSetWindowAttribute.</p>
            <div class="segmented-row">
              {#each [
                { id: "mica", label: "Mica (Nativo Win11)" },
                { id: "acrylic", label: "Acrylic (Vetro)" },
                { id: "tabbed", label: "Mica Alt (Tabbed)" },
                { id: "solid", label: "Solido Opaco" }
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
            <h3 class="group-title">Stile Controlli Finestra Desktop</h3>
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
          </div>

          <div class="panel-group">
            <h3 class="group-title">Scala Interfaccia & Zoom Dinamico</h3>
            <div class="zoom-control-box">
              <div class="zoom-header">
                <span>Ridimensionamento fluido dell'interfaccia utente (80% – 150%)</span>
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

        <!-- 2. PALETTE & COLORI -->
        {:else if activeTab === "colors"}
          <div class="panel-group">
            <h3 class="group-title">Colori di Sistema & Contrasto Elevato (WCAG AA/AAA)</h3>
            <p class="field-hint">Personalizza i colori guida. I testi e i contrasti vengono calcolati dinamicamente per garantire leggibilità ottimale.</p>
            <div class="colors-grid">
              <ColorPickerField
                label="Colore d'Accento Principale"
                value={accentColor}
                onChange={(hex) => (accentColor = hex)}
              />
              <ColorPickerField
                label="Festività Nazionali"
                value={holidayColor}
                onChange={(hex) => (holidayColor = hex)}
              />
              <ColorPickerField
                label="Domenica / Giorno di Riposo"
                value={sundayColor}
                onChange={(hex) => (sundayColor = hex)}
              />
            </div>
          </div>

          <div class="panel-group">
            <h3 class="group-title">Colori Categorie Eventi & Appunti</h3>
            <p class="field-hint">Tinte attribuite automaticamente alle card e agli indicatori di eventi generici e task giornalieri.</p>
            <div class="colors-grid">
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
              <ColorPickerField
                label="Categoria Lezione"
                value={catLezioneColor}
                onChange={(hex) => (catLezioneColor = hex)}
              />
              <ColorPickerField
                label="Categoria Personale"
                value={catPersonaleColor}
                onChange={(hex) => (catPersonaleColor = hex)}
              />
            </div>
          </div>

        <!-- 3. CALENDARIO & VISTE -->
        {:else if activeTab === "calendar"}
          <div class="panel-group">
            <h3 class="group-title">Vista Calendario Predefinita all'Avvio</h3>
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
            <h3 class="group-title">Configurazione Settimanale & Riposo</h3>
            <div class="form-grid-2">
              <label class="field">
                <span>Primo giorno della settimana</span>
                <select bind:value={startDayOfWeek}>
                  <option value={1}>Lunedì (Standard Europeo ISO 8601)</option>
                  <option value={0}>Domenica (Standard Anglosassone)</option>
                </select>
              </label>

              <label class="field">
                <span>Giorno di riposo settimanale evidenziato</span>
                <select bind:value={weeklyHolidayDay}>
                  <option value={0}>Domenica</option>
                  <option value={6}>Sabato</option>
                  <option value={-1}>Nessuno (Mostra tutti i giorni uguali)</option>
                </select>
              </label>
            </div>
          </div>

          <div class="panel-group">
            <h3 class="group-title">Visualizzazione Lezioni nel Calendario</h3>
            <p class="field-hint">
              Controlla la visualizzazione rapida dell'orario delle lezioni nelle celle del calendario. Il cassetto di dettaglio della singola giornata continuerà a mostrare sempre l'orario completo.
            </p>
            <label class="toggle-row" style="margin-top: 0.65rem;">
              <input
                type="checkbox"
                checked={themeStore.showLessonTimes}
                onchange={() => themeStore.toggleShowLessonTimes()}
              />
              <span>Mostra orario delle lezioni nelle celle del calendario (vista rapida)</span>
            </label>
          </div>

          <div class="panel-group">
            <h3 class="group-title">Scorciatoie da Tastiera per la Navigazione</h3>
            <div class="info-card">
              <p>• <strong>T</strong>: Salta istantaneamente alla data di oggi nel calendario.</p>
              <p>• <strong>Spazio</strong> oppure <strong>Ctrl+K</strong>: Apri la Command Palette / Assistente AI.</p>
              <p>• <strong>Ctrl + / -</strong>: Ingrandisci o riduci lo zoom dell'interfaccia.</p>
              <p>• <strong>Esc</strong>: Chiudi qualsiasi modale o cassetto laterale aperto.</p>
            </div>
          </div>

        <!-- 4. ALGORITMO CFU & CARICO -->
        {:else if activeTab === "workload"}
          <div class="panel-group">
            <h3 class="group-title">Parametro di Calcolo del Carico di Studio</h3>
            <label class="field">
              <span>Ore di studio stimate per singolo Credito Formativo Universitario (CFU)</span>
              <input
                type="number"
                min="10"
                max="50"
                bind:value={studyHoursPerCfu}
              />
            </label>
            <p class="field-hint">
              Il valore accademico standard in Italia è pari a <strong>25 ore per CFU</strong> (comprese le lezioni frontali e lo studio individuale).
            </p>
          </div>

          <div class="panel-group">
            <h3 class="group-title">Architettura del Workload Engine & Conflict Engine in Rust</h3>
            <div class="info-card">
              <strong>Motore di Calcolo Deterministico Integrato:</strong>
              <p>
                Il core nativo in Rust combina le ore per CFU con il fattore di difficoltà di ogni materia,
                le unità didattiche ancora da completare e la distanza temporale dagli appelli per determinare:
              </p>
              <p>• <strong>Ore di studio giornaliere consigliate</strong> per ciascuna data.</p>
              <p>• <strong>Buffer di sicurezza dinamico ($B$)</strong> tra appelli vicini, rilevando conflitti critici o avvisi di sovrapposizione.</p>
              <p>• <strong>Festività automatiche</strong> con calcolo perpetuo della Pasqua (algoritmo di Butcher) per preservare i giorni di riposo.</p>
            </div>
          </div>

        <!-- 5. PIANI DI STUDIO & BUILDER -->
        {:else if activeTab === "plan"}
          <div class="panel-group">
            <h3 class="group-title">Cartella Condivisa dei Piani di Studio (Progetto B)</h3>
            <p class="field-hint">
              Indica la directory in cui il Builder genera e aggiorna i file JSON dei piani di studio.
              MySession monitora questa cartella in background ed estrae automaticamente modifiche, nuovi corsi e to-do.
            </p>
            <div style="display: flex; gap: 0.5rem; align-items: center; margin-top: 0.5rem;">
              <input
                type="text"
                readonly
                value={planStore.planFolder || "Nessuna cartella configurata"}
                style="flex: 1; opacity: {planStore.planFolder ? 1 : 0.6};"
              />
              <button
                type="button"
                class="btn-action pressable"
                onclick={() => planStore.browseFolder()}
              >
                Sfoglia...
              </button>
              <button
                type="button"
                class="btn-action pressable"
                disabled={!planStore.planFolder}
                title="Apri la cartella in Esplora Risorse di Windows"
                onclick={() => planStore.openPlanFolder()}
              >
                📁 Apri Cartella
              </button>
              <button
                type="button"
                class="btn-action pressable"
                disabled={planStore.isScanning || !planStore.planFolder}
                onclick={() => planStore.scan()}
              >
                {planStore.isScanning ? "Scansione..." : "Verifica Ora"}
              </button>
            </div>
          </div>

          <div class="panel-group">
            <h3 class="group-title">Piani Rilevati & In Attesa</h3>
            {#if planStore.pendingPlans.length === 0}
              <div class="info-card">
                <p>Nessun nuovo piano o aggiornamento in attesa nella cartella selezionata.</p>
                <p style="font-size: 0.78rem; color: var(--text-tertiary); margin-top: 0.25rem;">
                  Quando crei o modifichi un piano con la skill in Progetto B, apparirà qui e tramite notifica interattiva.
                </p>
              </div>
            {:else}
              <div style="display: flex; flex-direction: column; gap: 0.5rem;">
                {#each planStore.pendingPlans as plan (plan.planId)}
                  <div
                    style="display: flex; align-items: center; justify-content: space-between; padding: 0.75rem 1rem; border-radius: var(--radius-sm); background: rgba(128, 128, 128, 0.06); border: 0.5px solid var(--hairline-border);"
                  >
                    <div>
                      <div style="font-weight: 600; font-size: 0.9rem; color: var(--text-primary);">
                        {plan.planName} ({plan.planId})
                      </div>
                      <div style="font-size: 0.75rem; color: var(--text-secondary); margin-top: 0.15rem;">
                        Revisione {plan.revision} • {plan.isNewSession ? "Nuova sessione" : "Aggiornamento sessione"}
                      </div>
                    </div>
                    <button
                      type="button"
                      class="btn-action pressable"
                      style="background: var(--accent, #007AFF); color: #fff; border: none; font-weight: 600;"
                      onclick={() => {
                        onClose();
                        planStore.openPreview(plan);
                      }}
                    >
                      Esamina & Applica
                    </button>
                  </div>
                {/each}
              </div>
            {/if}
          </div>

        <!-- 5. ASSISTENTE AI & NLP -->
        {:else if activeTab === "ai"}
          <div class="panel-group">
            <h3 class="group-title">Architettura Disaccoppiata Assistente AI</h3>
            <p class="field-hint">
              Nessuna credenziale è cablata nel codice. Puoi sfruttare il motore locale a latenza zero
              oppure collegare chiavi personali o modelli locali senza inviare dati a terzi.
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
                  placeholder="Inserisci la tua API Key (memorizzata localmente in SQLite)..."
                />
              </label>

              <label class="field">
                <span>Modello LLM</span>
                <input
                  type="text"
                  bind:value={aiModel}
                  placeholder={aiProvider === "gemini" ? "gemini-3.8-flash" : "gpt-4o-mini"}
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
                <strong>Motore NLP Deterministico Offline a Latenza Zero</strong>
                <p>
                  Esegue in modo nativo su Rust l'analisi dei comandi in italiano, consentendo l'aggiunta rapida
                  di appelli, materie, task e to-do, interrogazioni sullo stato dei CFU e cambi istantanei
                  di tema o materiali senza richiedere connessione internet o token esterni.
                </p>
              </div>
            {/if}
          </div>

        <!-- 6. PROFILO ACCADEMICO -->
        {:else if activeTab === "profile"}
          <div class="panel-group">
            <h3 class="group-title">Dati Personali Studente & Percorso di Studi</h3>
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

        <!-- 7. DATI & BACKUP JSON -->
        {:else if activeTab === "backup"}
          <div class="panel-group">
            <h3 class="group-title">Isolamento Locale & Portabilità Dati Multi-Utente</h3>
            <div class="info-card">
              <div>
                <strong>Database SQLite Locale (Modalità WAL):</strong>
                <code class="db-path-code">{sessionStore.dbPath || "%APPDATA%\\com.mysession.app\\mysession.db"}</code>
              </div>
              <div>
                <strong>Versione Software Ufficiale:</strong> MySession v{APP_VERSION} — Autore: {APP_AUTHOR}
              </div>
            </div>

            <div class="backup-actions-row">
              <button type="button" class="action-card-btn" onclick={handleExportBackup}>
                <span class="action-title">⬇ Esporta Backup Completo (.json)</span>
                <span class="action-desc">
                  Salva sessioni, corsi, appelli, note Markdown, to-do giornalieri e preferenze in un unico archivio.
                </span>
              </button>

              <button type="button" class="action-card-btn" onclick={handleImportBackupNative}>
                <span class="action-title">⬆ Importa Backup da File (.json)</span>
                <span class="action-desc">
                  Ripristina un workspace completo senza perdita di dati su qualsiasi PC.
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

        <!-- 9. AGGIORNAMENTI & GITHUB -->
        {:else if activeTab === "updates"}
          <div class="panel-group">
            <h3 class="group-title">Stato Installazione & Versione MySession</h3>
            <p class="field-hint">
              MySession supporta l'aggiornamento automatico e trasparente tramite le release ufficiali di GitHub con firma crittografica Minisign.
            </p>

            <div class="update-hero-card">
              <div class="uh-icon-col">
                <span class="uh-icon">🚀</span>
              </div>
              <div class="uh-info-col">
                <div class="uh-version-row">
                  <span class="uh-app-name">MySession</span>
                  <span class="uh-badge">v{APP_VERSION}</span>
                  {#if updaterStore.status === "up-to-date"}
                    <span class="badge-status-ok">✓ Aggiornato</span>
                  {:else if updaterStore.status === "available"}
                    <span class="badge-status-new">Nuova v{updaterStore.availableVersion}</span>
                  {/if}
                </div>
                <p class="uh-author-text">Software sviluppato da <strong>{APP_AUTHOR}</strong> · Repository: <code>riccardobettini/mysession</code></p>
                {#if updaterStore.lastCheckDate}
                  <span class="uh-time-text">Ultimo controllo: oggi alle {updaterStore.lastCheckDate}</span>
                {/if}
              </div>
            </div>
          </div>

          <div class="panel-group">
            <h3 class="group-title">Controllo e Download Aggiornamenti</h3>
            {#if updaterStore.status === "checking"}
              <div class="info-card">
                <p>⏳ Verifica della disponibilità di nuove release su GitHub in corso...</p>
              </div>
            {:else if updaterStore.status === "available"}
              <div class="update-available-card">
                <div class="uac-header">
                  <div>
                    <h4 style="margin: 0; font-size: 0.95rem; font-weight: 700;">Nuova versione v{updaterStore.availableVersion} disponibile!</h4>
                    {#if updaterStore.releaseDate}
                      <span style="font-size: 0.74rem; color: var(--text-tertiary);">Rilasciata il {updaterStore.releaseDate}</span>
                    {/if}
                  </div>
                  <button
                    type="button"
                    class="btn-install-update pressable"
                    onclick={() => updaterStore.downloadAndInstall()}
                  >
                    ⬇ Scarica e Installa Ora
                  </button>
                </div>
                {#if updaterStore.releaseNotes}
                  <div class="uac-notes">
                    <strong>Note di rilascio:</strong>
                    <div class="notes-body">{updaterStore.releaseNotes}</div>
                  </div>
                {/if}
              </div>
            {:else if updaterStore.status === "downloading"}
              <div class="update-progress-box">
                <div class="upb-header">
                  <span>Download aggiornamento v{updaterStore.availableVersion}...</span>
                  <strong>{updaterStore.downloadProgress}%</strong>
                </div>
                <div class="progress-track">
                  <div class="progress-bar-fill" style:width="{updaterStore.downloadProgress}%"></div>
                </div>
              </div>
            {:else if updaterStore.status === "ready"}
              <div class="update-ready-box">
                <div class="urb-text">
                  <span class="urb-icon">✓</span>
                  <div>
                    <h4 style="margin: 0; font-size: 0.95rem; font-weight: 700;">Aggiornamento installato con successo!</h4>
                    <p style="margin: 0.2rem 0 0; font-size: 0.77rem; color: var(--text-secondary);">La nuova versione v{updaterStore.availableVersion} è pronta. Riavvia per completare l'aggiornamento.</p>
                  </div>
                </div>
                <button
                  type="button"
                  class="btn-relaunch pressable"
                  onclick={() => updaterStore.relaunch()}
                >
                  Riavvia Ora
                </button>
              </div>
            {:else}
              <div style="display: flex; gap: 0.75rem; align-items: center;">
                <button
                  type="button"
                  class="btn-action pressable"
                  onclick={() => updaterStore.checkForUpdates(false)}
                >
                  🔄 Verifica Aggiornamenti Ora
                </button>
                {#if updaterStore.status === "up-to-date"}
                  <span style="font-size: 0.8rem; color: #30D158; font-weight: 600;">
                    ✓ Stai utilizzando l'ultima versione disponibile.
                  </span>
                {/if}
              </div>
              {#if updaterStore.errorMessage}
                <div class="info-card" style="border-color: rgba(255, 69, 58, 0.4); margin-top: 0.75rem;">
                  <p style="color: #FF453A; font-weight: 600; margin: 0;">Errore durante la verifica:</p>
                  <p style="font-size: 0.75rem; color: var(--text-secondary); margin-top: 0.2rem;">{updaterStore.errorMessage}</p>
                </div>
              {/if}
            {/if}
          </div>
        {/if}
      </div>
    </div>

    <!-- Footer Azioni della Scheda -->
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
    width: min(1040px, 94vw);
    max-width: 1040px;
    height: min(780px, 90vh);
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
    padding: 1.1rem 1.6rem;
    border-bottom: 0.5px solid var(--hairline-border);
    flex-shrink: 0;
  }

  .sheet-kicker {
    font-size: 0.68rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    color: var(--accent, #007AFF);
  }

  .sheet-title {
    margin: 0.15rem 0 0;
    font-size: 1.22rem;
    font-weight: 700;
    letter-spacing: -0.024em;
  }

  .close-btn {
    width: 2rem;
    height: 2rem;
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
    grid-template-columns: 260px 1fr;
    min-height: 0;
    overflow: hidden;
  }

  /* Barra Laterale Sinistra */
  .settings-nav {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: 1rem 0.85rem;
    border-right: 0.5px solid var(--hairline-border);
    background: rgba(128, 128, 128, 0.04);
    overflow: hidden;
  }

  .nav-items-scroll {
    display: flex;
    flex-direction: column;
    gap: 0.28rem;
    overflow-y: auto;
    padding-right: 0.25rem;
    scrollbar-width: thin;
    scrollbar-color: var(--hairline-border-strong) transparent;
  }

  .nav-group-header {
    font-size: 0.66rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    color: var(--text-tertiary);
    padding: 0.65rem 0.85rem 0.25rem;
  }

  .nav-group-header:first-of-type {
    padding-top: 0.2rem;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 0.72rem;
    padding: 0.62rem 0.85rem;
    border-radius: var(--radius-sm);
    border: none;
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.84rem;
    font-weight: 600;
    text-align: left;
    cursor: pointer;
    transition: transform 120ms cubic-bezier(0.16, 1, 0.3, 1), background 120ms ease, color 120ms ease;
  }

  .nav-icon {
    font-size: 0.98rem;
    width: 1.3rem;
    display: inline-flex;
    justify-content: center;
  }

  .nav-item:active {
    transform: scale(0.98);
  }

  .nav-item:hover {
    color: var(--text-primary);
    background: rgba(128, 128, 128, 0.08);
  }

  .nav-item.active {
    background: color-mix(in srgb, var(--accent, #007AFF) 15%, transparent);
    color: var(--accent, #007AFF);
  }

  /* Sezione Dedicata in Basso alla Barra Laterale Sinistra */
  .settings-nav-footer {
    margin-top: 0.65rem;
    padding: 0.75rem 0.85rem 0.4rem;
    border-top: 0.5px solid var(--hairline-border);
    display: flex;
    flex-direction: column;
    gap: 0.32rem;
    background: rgba(128, 128, 128, 0.03);
    border-radius: var(--radius-sm);
    flex-shrink: 0;
  }

  .nav-brand-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .nav-brand-title {
    font-size: 0.82rem;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: -0.01em;
  }

  .nav-version-badge {
    font-size: 0.7rem;
    font-weight: 600;
    padding: 0.12rem 0.45rem;
    border-radius: var(--radius-xs);
    background: var(--surface-pill);
    border: 0.5px solid var(--hairline-border);
    color: var(--text-secondary);
  }

  .nav-author-row {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.72rem;
    color: var(--text-tertiary);
  }

  .nav-author-label {
    font-weight: 500;
  }

  .nav-author-name {
    font-weight: 600;
    color: var(--text-secondary);
  }

  /* Area Contenuto Destra */
  .settings-content {
    padding: 1.45rem 2rem;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 1.4rem;
    scrollbar-width: thin;
    scrollbar-color: var(--hairline-border-strong) transparent;
  }

  .panel-group {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .group-title {
    margin: 0;
    font-size: 0.94rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .segmented-row {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
    gap: 0.4rem;
    padding: 0.3rem;
    border-radius: var(--radius-md);
    background: rgba(128, 128, 128, 0.1);
    border: 0.5px solid var(--hairline-border);
  }

  .seg-option {
    padding: 0.48rem 0.65rem;
    border-radius: var(--radius-xs);
    border: none;
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 140ms cubic-bezier(0.16, 1, 0.3, 1);
    text-align: center;
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
    font-size: 0.82rem;
    color: var(--text-secondary);
  }

  .zoom-presets {
    display: flex;
    gap: 0.4rem;
  }

  .preset-chip {
    padding: 0.28rem 0.65rem;
    border-radius: var(--radius-pill);
    border: 0.5px solid var(--hairline-border);
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.74rem;
    font-weight: 600;
    cursor: pointer;
  }

  .preset-chip.active {
    background: var(--accent, #007AFF);
    border-color: var(--accent, #007AFF);
    color: #fff;
  }

  .colors-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 1.1rem;
  }

  .form-grid-2 {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.85rem;
  }

  .toggle-row {
    display: inline-flex;
    align-items: center;
    gap: 0.65rem;
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--text-primary);
    cursor: pointer;
    user-select: none;
  }

  .toggle-row input[type="checkbox"] {
    width: 1.15rem;
    height: 1.15rem;
    accent-color: var(--accent-primary, #007AFF);
    cursor: pointer;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .field input,
  .field select {
    padding: 0.54rem 0.75rem;
    border-radius: var(--radius-md);
    border: 0.5px solid var(--hairline-border);
    background: var(--surface-input, rgba(128, 128, 128, 0.08));
    color: var(--text-primary);
    font-size: 0.84rem;
    outline: none;
    transition: border-color 140ms ease, box-shadow 140ms ease;
  }

  .field input:focus-visible,
  .field select:focus-visible {
    border-color: var(--accent, #007AFF);
    box-shadow: 0 0 0 2.5px rgba(0, 122, 255, 0.25);
  }

  .field-hint {
    margin: 0;
    font-size: 0.77rem;
    line-height: 1.45;
    color: var(--text-muted);
  }

  .info-card {
    padding: 0.9rem 1.1rem;
    border-radius: var(--radius-md);
    border: 0.5px solid var(--hairline-border);
    background: rgba(128, 128, 128, 0.06);
    font-size: 0.8rem;
    line-height: 1.5;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .info-card p {
    margin: 0;
    color: var(--text-secondary);
  }

  .db-path-code {
    display: block;
    margin-top: 0.25rem;
    padding: 0.35rem 0.55rem;
    border-radius: 6px;
    background: rgba(0, 0, 0, 0.2);
    font-family: var(--font-mono);
    font-size: 0.74rem;
    word-break: break-all;
  }

  .backup-actions-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.85rem;
  }

  .action-card-btn {
    padding: 0.95rem;
    border-radius: var(--radius-lg);
    border: 0.5px solid var(--hairline-border-strong);
    background: rgba(128, 128, 128, 0.07);
    text-align: left;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    cursor: pointer;
    transition: all 140ms ease;
  }

  .action-card-btn:hover {
    background: rgba(128, 128, 128, 0.14);
    border-color: var(--accent, #007AFF);
  }

  .action-title {
    font-size: 0.85rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .action-desc {
    font-size: 0.74rem;
    line-height: 1.4;
    color: var(--text-muted);
  }

  .backup-status-banner {
    padding: 0.65rem 0.85rem;
    border-radius: var(--radius-md);
    background: rgba(16, 185, 129, 0.14);
    border: 0.5px solid rgba(16, 185, 129, 0.4);
    color: #34d399;
    font-size: 0.78rem;
  }

  .sheet-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 0.65rem;
    padding: 1rem 1.6rem;
    border-top: 0.5px solid var(--hairline-border);
    flex-shrink: 0;
  }

  .secondary-btn {
    padding: 0.52rem 1.05rem;
    border-radius: var(--radius-md);
    border: 0.5px solid var(--hairline-border);
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.84rem;
    font-weight: 600;
    cursor: pointer;
  }

  .primary-btn {
    padding: 0.52rem 1.25rem;
    border-radius: var(--radius-md);
    border: none;
    background: var(--accent, #007AFF);
    color: #fff;
    font-size: 0.84rem;
    font-weight: 600;
    cursor: pointer;
  }

  /* Scheda Aggiornamenti */
  .nav-update-dot {
    width: 6px;
    height: 6px;
    border-radius: 999px;
    background: var(--accent-primary, #0A84FF);
    margin-left: auto;
  }

  .update-hero-card {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 1.15rem 1.25rem;
    border-radius: var(--radius-md);
    background: var(--surface-card);
    border: 0.5px solid var(--hairline-border);
  }

  .uh-icon-col {
    width: 3.25rem;
    height: 3.25rem;
    border-radius: 999px;
    background: rgba(10, 132, 255, 0.12);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.6rem;
    flex-shrink: 0;
  }

  .uh-info-col {
    flex: 1;
    min-width: 0;
  }

  .uh-version-row {
    display: flex;
    align-items: center;
    gap: 0.55rem;
  }

  .uh-app-name {
    font-size: 1.1rem;
    font-weight: 800;
  }

  .uh-badge {
    padding: 0.12rem 0.55rem;
    border-radius: 999px;
    background: var(--surface-pill);
    border: 0.5px solid var(--hairline-border);
    font-size: 0.72rem;
    font-weight: 700;
  }

  .badge-status-ok {
    padding: 0.12rem 0.55rem;
    border-radius: 999px;
    background: rgba(48, 209, 88, 0.15);
    color: #30D158;
    font-size: 0.7rem;
    font-weight: 700;
  }

  .badge-status-new {
    padding: 0.12rem 0.55rem;
    border-radius: 999px;
    background: var(--accent-primary, #0A84FF);
    color: #fff;
    font-size: 0.7rem;
    font-weight: 700;
  }

  .uh-author-text {
    margin: 0.25rem 0 0;
    font-size: 0.78rem;
    color: var(--text-secondary);
  }

  .uh-author-text code {
    font-family: monospace;
    font-size: 0.74rem;
    background: rgba(0, 0, 0, 0.15);
    padding: 0.1rem 0.35rem;
    border-radius: var(--radius-xs);
  }

  .uh-time-text {
    display: inline-block;
    margin-top: 0.35rem;
    font-size: 0.7rem;
    color: var(--text-tertiary);
  }

  .update-available-card {
    padding: 1.15rem;
    border-radius: var(--radius-md);
    background: var(--surface-card);
    border: 0.5px solid var(--accent-primary, #0A84FF);
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
  }

  .uac-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  .btn-install-update {
    padding: 0.5rem 1.1rem;
    border-radius: var(--radius-sm);
    border: none;
    background: var(--accent-primary, #0A84FF);
    color: #fff;
    font-size: 0.8rem;
    font-weight: 700;
    cursor: pointer;
    box-shadow: 0 2px 8px rgba(10, 132, 255, 0.35);
    white-space: nowrap;
  }

  .uac-notes {
    padding: 0.75rem;
    border-radius: var(--radius-sm);
    background: var(--surface-base);
    border: 0.5px solid var(--hairline-border);
    font-size: 0.78rem;
  }

  .notes-body {
    margin-top: 0.35rem;
    color: var(--text-secondary);
    white-space: pre-wrap;
    line-height: 1.4;
  }

  .update-progress-box {
    padding: 1.15rem;
    border-radius: var(--radius-md);
    background: var(--surface-card);
    border: 0.5px solid var(--hairline-border);
  }

  .upb-header {
    display: flex;
    justify-content: space-between;
    font-size: 0.82rem;
    font-weight: 600;
    margin-bottom: 0.5rem;
  }

  .progress-track {
    width: 100%;
    height: 6px;
    background: rgba(128, 128, 128, 0.2);
    border-radius: 999px;
    overflow: hidden;
  }

  .progress-bar-fill {
    height: 100%;
    background: var(--accent-primary, #0A84FF);
    border-radius: 999px;
    transition: width 140ms ease;
  }

  .update-ready-box {
    padding: 1.15rem;
    border-radius: var(--radius-md);
    background: rgba(48, 209, 88, 0.1);
    border: 0.5px solid rgba(48, 209, 88, 0.3);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  .urb-text {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .urb-icon {
    width: 2rem;
    height: 2rem;
    border-radius: 999px;
    background: #30D158;
    color: #fff;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 800;
  }

  .btn-relaunch {
    padding: 0.5rem 1.2rem;
    border-radius: var(--radius-sm);
    border: none;
    background: #30D158;
    color: #fff;
    font-size: 0.82rem;
    font-weight: 700;
    cursor: pointer;
    box-shadow: 0 2px 8px rgba(48, 209, 88, 0.35);
    white-space: nowrap;
  }
</style>
