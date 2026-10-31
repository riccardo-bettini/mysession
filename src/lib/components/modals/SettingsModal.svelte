<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";
  import { APP_AUTHOR, APP_VERSION } from "../../constants/version";
  import { sessionStore } from "../../stores/session.svelte";
  import { themeStore } from "../../stores/theme.svelte";
  import { planStore } from "../../stores/plan.svelte";
  import { updaterStore } from "../../stores/updater.svelte";
  import { ipc } from "../../api/ipc";
  import type {
    AiProvider,
    BackdropMaterial,
    CalendarViewMode,
    ThemeChoice,
    WindowControlsStyle,
    AiModelCatalogItem,
    SystemHardwareInfo,
    AiRuntimeStatus,
    AiModelDownloadProgress
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
  let aiLocalModelId = $state<string>(themeStore.settings.aiLocalModelId || "qwen2.5-vl-3b");
  let aiIdleTimeoutMinutes = $state<number>(themeStore.settings.aiIdleTimeoutMinutes ?? 5);
  let aiBootStrategy = $state<"on_demand" | "on_app_start" | "contextual">(
    themeStore.settings.aiBootStrategy || "on_demand"
  );
  let aiHardwarePreference = $state<"auto" | "gpu" | "cpu">(
    themeStore.settings.aiHardwarePreference || "auto"
  );

  let hardwareInfo = $state<SystemHardwareInfo | null>(null);
  let modelCatalog = $state<AiModelCatalogItem[]>([]);
  let runtimeStatus = $state<AiRuntimeStatus | null>(null);
  let downloadProgressMap = $state<Record<string, AiModelDownloadProgress>>({});
  let isAiLoading = $state<boolean>(false);
  let isWakingAi = $state<boolean>(false);
  let isStoppingAi = $state<boolean>(false);
  let runtimePollTimer: number | null = null;
  let unlistenDownloadProgress: UnlistenFn | null = null;

  async function loadAiData() {
    isAiLoading = true;
    try {
      const [hw, models, status] = await Promise.all([
        ipc.getSystemHardwareInfo().catch(() => null),
        ipc.getAiModelsStatus().catch(() => []),
        ipc.getAiRuntimeStatus().catch(() => null)
      ]);
      if (hw) hardwareInfo = hw;
      if (models) modelCatalog = models;
      if (status) runtimeStatus = status;
    } finally {
      isAiLoading = false;
    }
  }

  async function handleDownloadModel(modelId: string) {
    try {
      await ipc.startAiModelDownload(modelId);
      onToast?.("Download del modello avviato in background");
    } catch (e) {
      onToast?.(`Errore avvio download: ${e}`);
    }
  }

  async function handleCancelDownload(modelId: string) {
    try {
      await ipc.cancelAiModelDownload(modelId);
      const nextMap = { ...downloadProgressMap };
      delete nextMap[modelId];
      downloadProgressMap = nextMap;
      onToast?.("Download annullato");
    } catch (e) {
      onToast?.(`Errore annullamento: ${e}`);
    }
  }

  async function handleDeleteModel(modelId: string) {
    try {
      await ipc.deleteAiModel(modelId);
      await loadAiData();
      onToast?.("File del modello eliminato dal disco");
    } catch (e) {
      onToast?.(`Errore eliminazione: ${e}`);
    }
  }

  async function handleSetActiveModel(modelId: string) {
    aiLocalModelId = modelId;
    await themeStore.updateSettings({ aiLocalModelId: modelId });
    await loadAiData();
    onToast?.("Modello AI impostato come predefinito");
  }

  async function handleWakeUpRuntime() {
    isWakingAi = true;
    try {
      await ipc.startAiRuntime(aiLocalModelId);
      runtimeStatus = await ipc.getAiRuntimeStatus();
      onToast?.("Modello AI pronto in memoria");
    } catch (e) {
      onToast?.(`Errore avvio AI: ${e}`);
    } finally {
      isWakingAi = false;
    }
  }

  async function handleStopRuntime() {
    isStoppingAi = true;
    try {
      await ipc.stopAiRuntime();
      runtimeStatus = await ipc.getAiRuntimeStatus();
      onToast?.("Memoria RAM e VRAM liberata con successo");
    } catch (e) {
      onToast?.(`Errore spegnimento AI: ${e}`);
    } finally {
      isStoppingAi = false;
    }
  }

  onMount(async () => {
    await loadAiData();
    try {
      unlistenDownloadProgress = await listen<AiModelDownloadProgress>(
        "ai-model-download-progress",
        async (event) => {
          const p = event.payload;
          downloadProgressMap = {
            ...downloadProgressMap,
            [p.modelId]: p
          };
          if (p.isCompleted) {
            const nextMap = { ...downloadProgressMap };
            delete nextMap[p.modelId];
            downloadProgressMap = nextMap;
            await loadAiData();
            onToast?.("Download del modello completato con successo!");
          } else if (p.error) {
            const nextMap = { ...downloadProgressMap };
            delete nextMap[p.modelId];
            downloadProgressMap = nextMap;
            onToast?.(`Errore download: ${p.error}`);
          }
        }
      );
    } catch {}

    runtimePollTimer = window.setInterval(async () => {
      if (activeTab === "ai") {
        try {
          runtimeStatus = await ipc.getAiRuntimeStatus();
        } catch {}
      }
    }, 2000);
  });

  onDestroy(() => {
    if (unlistenDownloadProgress) unlistenDownloadProgress();
    if (runtimePollTimer) clearInterval(runtimePollTimer);
  });
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
        aiEndpointUrl: aiEndpointUrl.trim(),
        aiLocalModelId,
        aiIdleTimeoutMinutes,
        aiBootStrategy,
        aiHardwarePreference
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
            <h3 class="group-title">Architettura Assistente AI Multimodale</h3>
            <p class="field-hint">
              Scegli tra l'AI locale on-demand a privacy totale (senza token o abbonamenti) oppure provider esterni cloud.
              I modelli locali vengono eseguiti direttamente su questa macchina e liberano la memoria RAM/VRAM non appena finisci di usarli.
            </p>

            <div class="segmented-row">
              {#each [
                { id: "local", label: "AI Locale On-Demand" },
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

            {#if aiProvider === "local"}
              <!-- 1. Scheda Hardware Rilevato -->
              {#if hardwareInfo}
                <div class="ai-hw-banner">
                  <div class="hw-icon-col">💻</div>
                  <div class="hw-details-col">
                    <div class="hw-title-row">
                      <strong>Hardware Rilevato:</strong>
                      <span class="hw-gpu-tag">{hardwareInfo.gpuName} ({hardwareInfo.dedicatedVramGb > 0 ? `${hardwareInfo.dedicatedVramGb} GB VRAM` : "Integrata"})</span>
                      <span class="hw-ram-tag">{hardwareInfo.totalRamGb} GB RAM Totale</span>
                    </div>
                    <p class="hw-reason-text">
                      <span class="star-badge">★</span> {hardwareInfo.recommendedReason}
                    </p>
                  </div>
                </div>
              {/if}

              <!-- 2. Pannello Controllo Memoria & Alimentazione On-Demand -->
              <div class="ai-control-box">
                <div class="ai-control-header">
                  <div class="ai-status-indicator">
                    {#if runtimeStatus?.state === "ready"}
                      <span class="status-dot ready"></span>
                      <div>
                        <strong style="color: #34C759;">Pronto in Memoria</strong>
                        <span class="status-sub">
                          {#if runtimeStatus.secondsUntilIdleShutdown !== null && runtimeStatus.secondsUntilIdleShutdown !== undefined}
                            Spegnimento automatico tra {Math.floor(runtimeStatus.secondsUntilIdleShutdown / 60)}m {runtimeStatus.secondsUntilIdleShutdown % 60}s
                          {:else}
                            Sempre attivo (inattività disattivata)
                          {/if}
                        </span>
                      </div>
                    {:else if runtimeStatus?.state === "starting"}
                      <span class="status-dot starting"></span>
                      <div>
                        <strong style="color: #FF9500;">Caricamento in memoria...</strong>
                        <span class="status-sub">Inizializzazione pesi e layer GPU</span>
                      </div>
                    {:else if runtimeStatus?.state === "error"}
                      <span class="status-dot error"></span>
                      <div>
                        <strong style="color: #FF3B30;">Stato: Errore</strong>
                        <span class="status-sub">{runtimeStatus.errorMessage || "Impossibile avviare il modello"}</span>
                      </div>
                    {:else}
                      <span class="status-dot stopped"></span>
                      <div>
                        <strong>Stato: Spento a Riposo</strong>
                        <span class="status-sub">0 MB di RAM e VRAM occupati (latenza zero all'avvio app)</span>
                      </div>
                    {/if}
                  </div>

                  <div class="ai-quick-actions">
                    {#if runtimeStatus?.state === "ready" || runtimeStatus?.state === "starting"}
                      <button
                        type="button"
                        class="btn-danger-sub pressable"
                        disabled={isStoppingAi}
                        onclick={handleStopRuntime}
                      >
                        {isStoppingAi ? "Spegnimento..." : "🛑 Libera Memoria Adesso"}
                      </button>
                    {:else}
                      <button
                        type="button"
                        class="btn-wake-sub pressable"
                        disabled={isWakingAi}
                        onclick={handleWakeUpRuntime}
                      >
                        {isWakingAi ? "Avvio in corso..." : "⚡ Sveglia / Avvia Ora"}
                      </button>
                    {/if}
                  </div>
                </div>

                <div class="form-grid-3" style="margin-top: 1rem;">
                  <label class="field">
                    <span>Spegnimento per Inattività</span>
                    <select bind:value={aiIdleTimeoutMinutes}>
                      <option value={1}>Dopo 1 minuto</option>
                      <option value={3}>Dopo 3 minuti</option>
                      <option value={5}>Dopo 5 minuti (Consigliato)</option>
                      <option value={10}>Dopo 10 minuti</option>
                      <option value={15}>Dopo 15 minuti</option>
                      <option value={30}>Dopo 30 minuti</option>
                      <option value={0}>Mai (Sempre attivo)</option>
                    </select>
                  </label>

                  <label class="field">
                    <span>Strategia di Avvio</span>
                    <select bind:value={aiBootStrategy}>
                      <option value="on_demand">Su richiesta (Zero RAM all'avvio)</option>
                      <option value="on_app_start">Precarica all'avvio dell'app</option>
                      <option value="contextual">Contestuale (All'apertura orario)</option>
                    </select>
                  </label>

                  <label class="field">
                    <span>Accelerazione Hardware</span>
                    <select bind:value={aiHardwarePreference}>
                      <option value="auto">Automatico (GPU raccomandata)</option>
                      <option value="gpu">Forza GPU (Vulkan / DirectML)</option>
                      <option value="cpu">Forza solo CPU (Preserva VRAM)</option>
                    </select>
                  </label>
                </div>
              </div>

              <!-- 3. Model Hub & Catalogo 5 Modelli -->
              <div class="models-catalog-section">
                <div class="models-catalog-header">
                  <div>
                    <h4 style="margin: 0; font-size: 0.95rem; font-weight: 700;">Catalogo Modelli Multimodali Locali</h4>
                    <p class="field-hint">Seleziona o scarica il modello più adatto alla potenza del tuo computer.</p>
                  </div>
                  <button
                    type="button"
                    class="btn-refresh-sub pressable"
                    disabled={isAiLoading}
                    onclick={loadAiData}
                  >
                    🔄 Aggiorna Stato
                  </button>
                </div>

                <div class="models-grid">
                  {#each modelCatalog as m (m.id)}
                    {@const isRec = hardwareInfo?.recommendedModelId === m.id}
                    {@const isSelected = aiLocalModelId === m.id}
                    {@const dlProgress = downloadProgressMap[m.id]}

                    <div
                      class="model-card"
                      class:selected-active={isSelected && m.isDownloaded}
                      class:recommended-card={isRec}
                    >
                      <div class="model-card-top">
                        <div style="display: flex; align-items: center; gap: 0.4rem; flex-wrap: wrap;">
                          <span class="tier-pill tier-{m.tier}">Livello {m.tierLevel} · {m.tier.toUpperCase()}</span>
                          {#if isRec}
                            <span class="badge-recommended">★ Consigliato per te</span>
                          {/if}
                          {#if isSelected && m.isDownloaded}
                            <span class="badge-active-model">● Modello Predefinito</span>
                          {/if}
                        </div>
                        <h5 class="model-name">{m.name}</h5>
                        <p class="model-tagline">{m.tagline}</p>
                      </div>

                      <div class="model-specs-row">
                        <span title="Spazio occupato sul disco SSD">💾 ~{m.sizeGb} GB</span>
                        <span title="Memoria RAM/VRAM occupata durante l'uso">⚡ ~{m.vramReqGb} GB</span>
                        <span title="Finestra di contesto token">📝 {m.contextWindow} tok</span>
                      </div>

                      <p class="model-desc">{m.description}</p>
                      <p class="model-rec-for"><strong>Adatto a:</strong> {m.recommendedFor}</p>

                      <div class="model-card-actions">
                        {#if dlProgress}
                          <div class="download-progress-container">
                            <div class="dl-text-row">
                              <span>Download in corso... ({dlProgress.progressPct}%)</span>
                              <span>{dlProgress.speedMbPerSec} MB/s</span>
                            </div>
                            <div class="progress-bar-track">
                              <div class="progress-bar-fill" style="width: {dlProgress.progressPct}%"></div>
                            </div>
                            <button
                              type="button"
                              class="btn-cancel-dl pressable"
                              onclick={() => handleCancelDownload(m.id)}
                            >
                              Annulla
                            </button>
                          </div>
                        {:else if m.isDownloaded}
                          <div class="action-btn-row">
                            {#if !isSelected}
                              <button
                                type="button"
                                class="btn-set-active pressable"
                                onclick={() => handleSetActiveModel(m.id)}
                              >
                                Imposta come Attivo
                              </button>
                            {:else}
                              <span class="installed-tag">✓ Attivo per Vision & Chat</span>
                            {/if}
                            <button
                              type="button"
                              class="btn-delete-model pressable"
                              title="Elimina file dal disco per liberare spazio"
                              onclick={() => handleDeleteModel(m.id)}
                            >
                              🗑️ Elimina (~{m.sizeGb} GB)
                            </button>
                          </div>
                        {:else}
                          <button
                            type="button"
                            class="btn-download-model pressable"
                            class:btn-rec-cta={isRec}
                            onclick={() => handleDownloadModel(m.id)}
                          >
                            ⬇️ Scarica Modello (~{m.sizeGb} GB)
                          </button>
                        {/if}
                      </div>
                    </div>
                  {/each}
                </div>
              </div>

            {:else if aiProvider === "gemini" || aiProvider === "openai"}
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
                <p class="uh-author-text">Software sviluppato da <strong>{APP_AUTHOR}</strong> · Repository: <code>riccardo-bettini/mysession</code></p>
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

  /* =========================================================================
     AI Multimodale Locale & Model Hub (Apple HIG)
     ========================================================================= */
  .ai-hw-banner {
    display: flex;
    align-items: flex-start;
    gap: 0.85rem;
    padding: 0.85rem 1rem;
    border-radius: var(--radius-md);
    background: rgba(0, 122, 255, 0.08);
    border: 0.5px solid rgba(0, 122, 255, 0.25);
  }

  .hw-icon-col {
    font-size: 1.5rem;
    line-height: 1;
  }

  .hw-details-col {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-size: 0.82rem;
  }

  .hw-title-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
    color: var(--text-primary);
  }

  .hw-gpu-tag,
  .hw-ram-tag {
    padding: 0.15rem 0.5rem;
    border-radius: var(--radius-pill);
    background: rgba(0, 122, 255, 0.14);
    color: var(--accent, #007AFF);
    font-size: 0.74rem;
    font-weight: 600;
  }

  .hw-reason-text {
    margin: 0;
    color: var(--text-secondary);
    font-size: 0.77rem;
    line-height: 1.4;
  }

  .star-badge {
    color: #FF9500;
    font-weight: 800;
  }

  .ai-control-box {
    padding: 1rem 1.15rem;
    border-radius: var(--radius-md);
    border: 0.5px solid var(--hairline-border);
    background: rgba(128, 128, 128, 0.06);
    display: flex;
    flex-direction: column;
  }

  .ai-control-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .ai-status-indicator {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    font-size: 0.82rem;
  }

  .status-dot {
    width: 0.75rem;
    height: 0.75rem;
    border-radius: 999px;
    flex-shrink: 0;
  }

  .status-dot.ready {
    background: #34C759;
    box-shadow: 0 0 8px rgba(52, 199, 89, 0.6);
  }

  .status-dot.starting {
    background: #FF9500;
    box-shadow: 0 0 8px rgba(255, 149, 0, 0.6);
    animation: pulse 1.2s infinite alternate;
  }

  .status-dot.stopped {
    background: #8E8E93;
  }

  .status-dot.error {
    background: #FF3B30;
  }

  @keyframes pulse {
    from { opacity: 0.5; }
    to { opacity: 1; }
  }

  .status-sub {
    display: block;
    font-size: 0.72rem;
    color: var(--text-muted);
    font-weight: 500;
  }

  .ai-quick-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .btn-danger-sub {
    padding: 0.45rem 0.85rem;
    border-radius: var(--radius-sm);
    border: 0.5px solid rgba(255, 59, 48, 0.4);
    background: rgba(255, 59, 48, 0.12);
    color: #FF3B30;
    font-size: 0.78rem;
    font-weight: 700;
    cursor: pointer;
  }

  .btn-wake-sub {
    padding: 0.45rem 0.85rem;
    border-radius: var(--radius-sm);
    border: 0.5px solid rgba(0, 122, 255, 0.4);
    background: rgba(0, 122, 255, 0.12);
    color: var(--accent, #007AFF);
    font-size: 0.78rem;
    font-weight: 700;
    cursor: pointer;
  }

  .form-grid-3 {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 0.75rem;
  }

  .models-catalog-section {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-top: 0.5rem;
  }

  .models-catalog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  .btn-refresh-sub {
    padding: 0.35rem 0.65rem;
    border-radius: var(--radius-pill);
    border: 0.5px solid var(--hairline-border);
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.74rem;
    font-weight: 600;
    cursor: pointer;
  }

  .models-grid {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
  }

  .model-card {
    padding: 1rem 1.15rem;
    border-radius: var(--radius-md);
    border: 0.5px solid var(--hairline-border);
    background: rgba(128, 128, 128, 0.04);
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
    transition: all 180ms ease;
  }

  .model-card:hover {
    border-color: rgba(128, 128, 128, 0.35);
    background: rgba(128, 128, 128, 0.07);
  }

  .model-card.recommended-card {
    border-color: rgba(255, 149, 0, 0.4);
    background: rgba(255, 149, 0, 0.03);
  }

  .model-card.selected-active {
    border-color: var(--accent, #007AFF);
    box-shadow: 0 0 0 1px var(--accent, #007AFF);
  }

  .tier-pill {
    padding: 0.15rem 0.5rem;
    border-radius: var(--radius-pill);
    font-size: 0.68rem;
    font-weight: 700;
    background: rgba(128, 128, 128, 0.18);
    color: var(--text-secondary);
  }

  .tier-pill.tier-bilanciato {
    background: rgba(0, 122, 255, 0.18);
    color: var(--accent, #007AFF);
  }

  .tier-pill.tier-avanzato {
    background: rgba(175, 82, 222, 0.18);
    color: #AF52DE;
  }

  .badge-recommended {
    padding: 0.15rem 0.5rem;
    border-radius: var(--radius-pill);
    font-size: 0.68rem;
    font-weight: 700;
    background: rgba(255, 149, 0, 0.18);
    color: #FF9500;
  }

  .badge-active-model {
    padding: 0.15rem 0.5rem;
    border-radius: var(--radius-pill);
    font-size: 0.68rem;
    font-weight: 700;
    background: rgba(52, 199, 89, 0.18);
    color: #34C759;
  }

  .model-name {
    margin: 0.25rem 0 0;
    font-size: 0.92rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .model-tagline {
    margin: 0;
    font-size: 0.77rem;
    color: var(--text-secondary);
    font-style: italic;
  }

  .model-specs-row {
    display: flex;
    gap: 0.85rem;
    flex-wrap: wrap;
    font-size: 0.74rem;
    color: var(--text-muted);
    font-weight: 600;
  }

  .model-desc {
    margin: 0;
    font-size: 0.79rem;
    line-height: 1.45;
    color: var(--text-secondary);
  }

  .model-rec-for {
    margin: 0;
    font-size: 0.74rem;
    color: var(--text-muted);
  }

  .model-card-actions {
    margin-top: 0.35rem;
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .action-btn-row {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    flex-wrap: wrap;
  }

  .btn-set-active {
    padding: 0.4rem 0.85rem;
    border-radius: var(--radius-sm);
    border: none;
    background: var(--accent, #007AFF);
    color: #fff;
    font-size: 0.78rem;
    font-weight: 600;
    cursor: pointer;
  }

  .installed-tag {
    font-size: 0.76rem;
    font-weight: 700;
    color: #34C759;
  }

  .btn-delete-model {
    padding: 0.4rem 0.75rem;
    border-radius: var(--radius-sm);
    border: 0.5px solid var(--hairline-border);
    background: transparent;
    color: var(--text-muted);
    font-size: 0.74rem;
    font-weight: 600;
    cursor: pointer;
  }

  .btn-delete-model:hover {
    color: #FF3B30;
    border-color: rgba(255, 59, 48, 0.4);
    background: rgba(255, 59, 48, 0.08);
  }

  .btn-download-model {
    padding: 0.48rem 1rem;
    border-radius: var(--radius-sm);
    border: 0.5px solid var(--hairline-border);
    background: var(--surface-card, rgba(128, 128, 128, 0.12));
    color: var(--text-primary);
    font-size: 0.78rem;
    font-weight: 600;
    cursor: pointer;
  }

  .btn-download-model.btn-rec-cta {
    background: var(--accent, #007AFF);
    border-color: var(--accent, #007AFF);
    color: #fff;
    font-weight: 700;
  }

  .download-progress-container {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .dl-text-row {
    display: flex;
    justify-content: space-between;
    font-size: 0.74rem;
    color: var(--accent, #007AFF);
    font-weight: 600;
  }

  .progress-bar-track {
    width: 100%;
    height: 0.45rem;
    border-radius: 999px;
    background: rgba(128, 128, 128, 0.2);
    overflow: hidden;
  }

  .progress-bar-fill {
    height: 100%;
    background: var(--accent, #007AFF);
    border-radius: 999px;
    transition: width 200ms ease;
  }

  .btn-cancel-dl {
    align-self: flex-start;
    padding: 0.25rem 0.55rem;
    border-radius: var(--radius-xs);
    border: 0.5px solid var(--hairline-border);
    background: transparent;
    color: var(--text-muted);
    font-size: 0.72rem;
    cursor: pointer;
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

  /* =========================================================================
     AI Multimodale Locale & Model Hub (Apple HIG)
     ========================================================================= */
  .ai-hw-banner {
    display: flex;
    align-items: flex-start;
    gap: 0.85rem;
    padding: 0.85rem 1rem;
    border-radius: var(--radius-md);
    background: rgba(0, 122, 255, 0.08);
    border: 0.5px solid rgba(0, 122, 255, 0.25);
  }

  .hw-icon-col {
    font-size: 1.5rem;
    line-height: 1;
  }

  .hw-details-col {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-size: 0.82rem;
  }

  .hw-title-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
    color: var(--text-primary);
  }

  .hw-gpu-tag,
  .hw-ram-tag {
    padding: 0.15rem 0.5rem;
    border-radius: var(--radius-pill);
    background: rgba(0, 122, 255, 0.14);
    color: var(--accent, #007AFF);
    font-size: 0.74rem;
    font-weight: 600;
  }

  .hw-reason-text {
    margin: 0;
    color: var(--text-secondary);
    font-size: 0.77rem;
    line-height: 1.4;
  }

  .star-badge {
    color: #FF9500;
    font-weight: 800;
  }

  .ai-control-box {
    padding: 1rem 1.15rem;
    border-radius: var(--radius-md);
    border: 0.5px solid var(--hairline-border);
    background: rgba(128, 128, 128, 0.06);
    display: flex;
    flex-direction: column;
  }

  .ai-control-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .ai-status-indicator {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    font-size: 0.82rem;
  }

  .status-dot {
    width: 0.75rem;
    height: 0.75rem;
    border-radius: 999px;
    flex-shrink: 0;
  }

  .status-dot.ready {
    background: #34C759;
    box-shadow: 0 0 8px rgba(52, 199, 89, 0.6);
  }

  .status-dot.starting {
    background: #FF9500;
    box-shadow: 0 0 8px rgba(255, 149, 0, 0.6);
    animation: pulse 1.2s infinite alternate;
  }

  .status-dot.stopped {
    background: #8E8E93;
  }

  .status-dot.error {
    background: #FF3B30;
  }

  @keyframes pulse {
    from { opacity: 0.5; }
    to { opacity: 1; }
  }

  .status-sub {
    display: block;
    font-size: 0.72rem;
    color: var(--text-muted);
    font-weight: 500;
  }

  .ai-quick-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .btn-danger-sub {
    padding: 0.45rem 0.85rem;
    border-radius: var(--radius-sm);
    border: 0.5px solid rgba(255, 59, 48, 0.4);
    background: rgba(255, 59, 48, 0.12);
    color: #FF3B30;
    font-size: 0.78rem;
    font-weight: 700;
    cursor: pointer;
  }

  .btn-wake-sub {
    padding: 0.45rem 0.85rem;
    border-radius: var(--radius-sm);
    border: 0.5px solid rgba(0, 122, 255, 0.4);
    background: rgba(0, 122, 255, 0.12);
    color: var(--accent, #007AFF);
    font-size: 0.78rem;
    font-weight: 700;
    cursor: pointer;
  }

  .form-grid-3 {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 0.75rem;
  }

  .models-catalog-section {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-top: 0.5rem;
  }

  .models-catalog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  .btn-refresh-sub {
    padding: 0.35rem 0.65rem;
    border-radius: var(--radius-pill);
    border: 0.5px solid var(--hairline-border);
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.74rem;
    font-weight: 600;
    cursor: pointer;
  }

  .models-grid {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
  }

  .model-card {
    padding: 1rem 1.15rem;
    border-radius: var(--radius-md);
    border: 0.5px solid var(--hairline-border);
    background: rgba(128, 128, 128, 0.04);
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
    transition: all 180ms ease;
  }

  .model-card:hover {
    border-color: rgba(128, 128, 128, 0.35);
    background: rgba(128, 128, 128, 0.07);
  }

  .model-card.recommended-card {
    border-color: rgba(255, 149, 0, 0.4);
    background: rgba(255, 149, 0, 0.03);
  }

  .model-card.selected-active {
    border-color: var(--accent, #007AFF);
    box-shadow: 0 0 0 1px var(--accent, #007AFF);
  }

  .tier-pill {
    padding: 0.15rem 0.5rem;
    border-radius: var(--radius-pill);
    font-size: 0.68rem;
    font-weight: 700;
    background: rgba(128, 128, 128, 0.18);
    color: var(--text-secondary);
  }

  .tier-pill.tier-bilanciato {
    background: rgba(0, 122, 255, 0.18);
    color: var(--accent, #007AFF);
  }

  .tier-pill.tier-avanzato {
    background: rgba(175, 82, 222, 0.18);
    color: #AF52DE;
  }

  .badge-recommended {
    padding: 0.15rem 0.5rem;
    border-radius: var(--radius-pill);
    font-size: 0.68rem;
    font-weight: 700;
    background: rgba(255, 149, 0, 0.18);
    color: #FF9500;
  }

  .badge-active-model {
    padding: 0.15rem 0.5rem;
    border-radius: var(--radius-pill);
    font-size: 0.68rem;
    font-weight: 700;
    background: rgba(52, 199, 89, 0.18);
    color: #34C759;
  }

  .model-name {
    margin: 0.25rem 0 0;
    font-size: 0.92rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .model-tagline {
    margin: 0;
    font-size: 0.77rem;
    color: var(--text-secondary);
    font-style: italic;
  }

  .model-specs-row {
    display: flex;
    gap: 0.85rem;
    flex-wrap: wrap;
    font-size: 0.74rem;
    color: var(--text-muted);
    font-weight: 600;
  }

  .model-desc {
    margin: 0;
    font-size: 0.79rem;
    line-height: 1.45;
    color: var(--text-secondary);
  }

  .model-rec-for {
    margin: 0;
    font-size: 0.74rem;
    color: var(--text-muted);
  }

  .model-card-actions {
    margin-top: 0.35rem;
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .action-btn-row {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    flex-wrap: wrap;
  }

  .btn-set-active {
    padding: 0.4rem 0.85rem;
    border-radius: var(--radius-sm);
    border: none;
    background: var(--accent, #007AFF);
    color: #fff;
    font-size: 0.78rem;
    font-weight: 600;
    cursor: pointer;
  }

  .installed-tag {
    font-size: 0.76rem;
    font-weight: 700;
    color: #34C759;
  }

  .btn-delete-model {
    padding: 0.4rem 0.75rem;
    border-radius: var(--radius-sm);
    border: 0.5px solid var(--hairline-border);
    background: transparent;
    color: var(--text-muted);
    font-size: 0.74rem;
    font-weight: 600;
    cursor: pointer;
  }

  .btn-delete-model:hover {
    color: #FF3B30;
    border-color: rgba(255, 59, 48, 0.4);
    background: rgba(255, 59, 48, 0.08);
  }

  .btn-download-model {
    padding: 0.48rem 1rem;
    border-radius: var(--radius-sm);
    border: 0.5px solid var(--hairline-border);
    background: var(--surface-card, rgba(128, 128, 128, 0.12));
    color: var(--text-primary);
    font-size: 0.78rem;
    font-weight: 600;
    cursor: pointer;
  }

  .btn-download-model.btn-rec-cta {
    background: var(--accent, #007AFF);
    border-color: var(--accent, #007AFF);
    color: #fff;
    font-weight: 700;
  }

  .download-progress-container {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .dl-text-row {
    display: flex;
    justify-content: space-between;
    font-size: 0.74rem;
    color: var(--accent, #007AFF);
    font-weight: 600;
  }

  .progress-bar-track {
    width: 100%;
    height: 0.45rem;
    border-radius: 999px;
    background: rgba(128, 128, 128, 0.2);
    overflow: hidden;
  }

  .progress-bar-fill {
    height: 100%;
    background: var(--accent, #007AFF);
    border-radius: 999px;
    transition: width 200ms ease;
  }

  .btn-cancel-dl {
    align-self: flex-start;
    padding: 0.25rem 0.55rem;
    border-radius: var(--radius-xs);
    border: 0.5px solid var(--hairline-border);
    background: transparent;
    color: var(--text-muted);
    font-size: 0.72rem;
    cursor: pointer;
  }
</style>
