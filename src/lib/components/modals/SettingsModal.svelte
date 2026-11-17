<script lang="ts">
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    ArrowDown,
    ArrowUp,
    CheckCircle2,
    Download,
    Eye,
    EyeOff,
    FolderOpen,
    Play,
    RefreshCw,
    Square,
    Terminal,
    Trash2,
    X
  } from "lucide-svelte";
  import { onMount } from "svelte";
  import { ipc } from "../../api/ipc";
  import { APP_AUTHOR, APP_VERSION } from "../../constants/version";
  import { navStore } from "../../stores/navigation.svelte";
  import { sessionStore } from "../../stores/session.svelte";
  import { themeStore, UI_SCALE_MAX, UI_SCALE_MIN, UI_SCALE_STEP } from "../../stores/theme.svelte";
  import { updaterStore } from "../../stores/updater.svelte";
  import type {
    AiModelCatalogItem,
    AiModelDownloadProgress,
    AiProvider,
    AiRuntimeStatus,
    BackdropMaterial,
    CliAgentInfo,
    SectionId,
    ThemeChoice
  } from "../../types/domain";
  import { CORE_FIVE_CATEGORIES, SECTION_LABELS } from "../../utils/categories";
  import ColorPickerField from "../common/ColorPickerField.svelte";

  let activeTab = $state<"appearance" | "sidebar" | "sessions" | "ai">("appearance");
  let cliAgents = $state<CliAgentInfo[]>([]);
  let aiModels = $state<AiModelCatalogItem[]>([]);
  let runtimeStatus = $state<AiRuntimeStatus | null>(null);
  let downloadProgress = $state<AiModelDownloadProgress | null>(null);
  let customCliInput = $state(themeStore.settings.aiCustomCliCommand);
  let unlistenDl: UnlistenFn | null = null;

  $effect(() => {
    if (navStore.isSettingsOpen) {
      activeTab = navStore.settingsInitialTab;
      customCliInput = themeStore.settings.aiCustomCliCommand;
      void refreshAiState();
    }
  });

  async function refreshAiState() {
    try {
      const [clis, mods, rt] = await Promise.all([
        ipc.getCliAgentsStatus(),
        ipc.getAiModelsStatus(),
        ipc.getAiRuntimeStatus()
      ]);
      cliAgents = clis;
      aiModels = mods;
      runtimeStatus = rt;
    } catch {
      // ignore
    }
  }

  onMount(() => {
    let active = true;
    void listen<AiModelDownloadProgress>("ai-model-download-progress", (ev) => {
      if (!active) return;
      downloadProgress = ev.payload;
      if (ev.payload.isCompleted || ev.payload.error) {
        void refreshAiState();
      }
    }).then((un) => {
      unlistenDl = un;
    });
    return () => {
      active = false;
      unlistenDl?.();
    };
  });

  function moveSection(idx: number, delta: number) {
    const list = [...themeStore.settings.sidebarSections];
    const target = idx + delta;
    if (target < 0 || target >= list.length) return;
    [list[idx], list[target]] = [list[target], list[idx]];
    void themeStore.updateSettings({ sidebarSections: list });
  }

  function toggleHiddenSection(id: SectionId) {
    if (id === "home") return; // Home sempre visibile
    const hidden = new Set(themeStore.settings.hiddenSections);
    if (hidden.has(id)) hidden.delete(id);
    else hidden.add(id);
    void themeStore.updateSettings({ hiddenSections: Array.from(hidden) });
  }

  async function choosePlanFolder() {
    const selected = await open({ directory: true, multiple: false, title: "Cartella file delle sessioni" });
    if (typeof selected === "string" && selected.trim()) {
      const pending = await sessionStore.setPlanFolder(selected);
      navStore.showToast(
        pending.length > 0 ? `Trovati ${pending.length} piani nella cartella` : "Cartella salvata",
        "info"
      );
    }
  }

  function getCatColorVal(id: string): string {
    switch (id) {
      case "teoria":
        return themeStore.settings.catTeoriaColor;
      case "esercizi":
        return themeStore.settings.catEserciziColor;
      case "temi_esame":
        return themeStore.settings.catTemiEsameColor;
      case "dimostrazioni":
        return themeStore.settings.catDimostrazioniColor;
      default:
        return themeStore.settings.catDomandeOraleColor;
    }
  }

  function setCatColor(id: string, hex: string) {
    switch (id) {
      case "teoria":
        void themeStore.updateSettings({ catTeoriaColor: hex });
        break;
      case "esercizi":
        void themeStore.updateSettings({ catEserciziColor: hex });
        break;
      case "temi_esame":
        void themeStore.updateSettings({ catTemiEsameColor: hex });
        break;
      case "dimostrazioni":
        void themeStore.updateSettings({ catDimostrazioniColor: hex });
        break;
      default:
        void themeStore.updateSettings({ catDomandeOraleColor: hex });
        break;
    }
  }
</script>

{#if navStore.isSettingsOpen}
  <div class="modal-backdrop" role="dialog" aria-modal="true" aria-label="Impostazioni">
    <div class="bento-card settings-modal bento-enter">
      <header class="sm-head">
        <div>
          <h2>Impostazioni</h2>
          <p>MySession v{APP_VERSION} · {APP_AUTHOR}</p>
        </div>
        <button type="button" class="close-btn" onclick={() => (navStore.isSettingsOpen = false)} aria-label="Chiudi">
          <X size={18} />
        </button>
      </header>

      <nav class="sm-tabs">
        <button type="button" class="pill-filter" class:active={activeTab === "appearance"} onclick={() => (activeTab = "appearance")}>
          Aspetto & Colori
        </button>
        <button type="button" class="pill-filter" class:active={activeTab === "sidebar"} onclick={() => (activeTab = "sidebar")}>
          Barra Laterale
        </button>
        <button type="button" class="pill-filter" class:active={activeTab === "sessions"} onclick={() => (activeTab = "sessions")}>
          Sessioni & Cartella
        </button>
        <button type="button" class="pill-filter" class:active={activeTab === "ai"} onclick={() => (activeTab = "ai")}>
          AI & Agenti CLI
        </button>
      </nav>

      <div class="sm-body">
        {#if activeTab === "appearance"}
          <div class="set-group">
            <h4>Tema</h4>
            <div class="seg-row">
              {#each [["light", "Chiaro (Video)"], ["dark", "Scuro"], ["auto", "Sistema"]] as [val, lbl] (val)}
                <button
                  type="button"
                  class="pill-filter"
                  class:active={themeStore.settings.theme === val}
                  onclick={() => themeStore.updateSettings({ theme: val as ThemeChoice })}
                >
                  {lbl}
                </button>
              {/each}
            </div>
          </div>

          <div class="set-group">
            <h4>Materiale Finestra Windows 11</h4>
            <div class="seg-row">
              {#each [["mica", "Mica"], ["acrylic", "Acrylic"], ["tabbed", "Tabbed"], ["solid", "Solido"]] as [val, lbl] (val)}
                <button
                  type="button"
                  class="pill-filter"
                  class:active={themeStore.settings.backdropMaterial === val}
                  onclick={() => themeStore.updateSettings({ backdropMaterial: val as BackdropMaterial })}
                >
                  {lbl}
                </button>
              {/each}
            </div>
          </div>

          <div class="set-group">
            <h4>Scala Interfaccia ({themeStore.settings.uiScale}%)</h4>
            <input
              type="range"
              min={UI_SCALE_MIN}
              max={UI_SCALE_MAX}
              step={UI_SCALE_STEP}
              value={themeStore.settings.uiScale}
              oninput={(e) => themeStore.updateSettings({ uiScale: Number((e.currentTarget as HTMLInputElement).value) })}
            />
          </div>

          <div class="set-group">
            <ColorPickerField
              label="Colore Accento App"
              previewText="ACCENTO"
              value={themeStore.settings.accentColor}
              onChange={(hex) => themeStore.updateSettings({ accentColor: hex })}
            />
          </div>

          <div class="set-group">
            <h4>Colori delle 5 Categorie</h4>
            <div class="cat-colors-grid">
              {#each CORE_FIVE_CATEGORIES as cat (cat.id)}
                <label class="cat-color-chip">
                  <input
                    type="color"
                    value={getCatColorVal(cat.id)}
                    onchange={(e) => setCatColor(cat.id, (e.currentTarget as HTMLInputElement).value)}
                  />
                  <span>{cat.label}</span>
                </label>
              {/each}
            </div>
          </div>

          <div class="set-group row-between">
            <div>
              <h4>Aggiornamenti Automatici</h4>
              <p class="hint">Versione corrente v{APP_VERSION}</p>
            </div>
            <button type="button" class="pill-btn pill-btn-soft" onclick={() => updaterStore.checkForUpdates(false)}>
              <RefreshCw size={14} />
              Controlla ora
            </button>
          </div>
        {:else if activeTab === "sidebar"}
          <div class="set-group">
            <h4>Ordine e Visibilità delle Sezioni nella Barra Laterale</h4>
            <p class="hint">Sposta le sezioni per cambiare l'ordine nella Floating Pill Sidebar (esclusi Logo, Sessioni e Impostazioni).</p>
            <div class="order-list">
              {#each themeStore.settings.sidebarSections as secId, idx (secId)}
                {@const isHidden = themeStore.settings.hiddenSections.includes(secId)}
                <div class="order-row">
                  <span class="ord-idx">{idx + 1}</span>
                  <strong class="ord-lbl">{SECTION_LABELS[secId] ?? secId}</strong>
                  <div class="ord-actions">
                    <button type="button" class="icon-mini" disabled={idx === 0} onclick={() => moveSection(idx, -1)} title="Sposta su">
                      <ArrowUp size={14} />
                    </button>
                    <button
                      type="button"
                      class="icon-mini"
                      disabled={idx === themeStore.settings.sidebarSections.length - 1}
                      onclick={() => moveSection(idx, 1)}
                      title="Sposta giù"
                    >
                      <ArrowDown size={14} />
                    </button>
                    <button
                      type="button"
                      class="icon-mini"
                      disabled={secId === "home"}
                      onclick={() => toggleHiddenSection(secId)}
                      title={isHidden ? "Mostra sezione" : "Nascondi sezione"}
                    >
                      {#if isHidden}<EyeOff size={14} />{:else}<Eye size={14} />{/if}
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {:else if activeTab === "sessions"}
          <div class="set-group">
            <h4>Cartella File delle Sessioni (.json v3)</h4>
            <p class="hint">MySession monitora questa cartella per rilevare nuovi piani o revisioni dal Builder.</p>
            <div class="folder-row">
              <input type="text" class="soft-input" readonly value={themeStore.settings.planFolder || "Nessuna cartella selezionata"} />
              <button type="button" class="pill-btn pill-btn-primary" onclick={choosePlanFolder}>
                <FolderOpen size={14} />
                Scegli
              </button>
              {#if themeStore.settings.planFolder}
                <button type="button" class="pill-btn pill-btn-soft" onclick={() => ipc.openFolderInExplorer(themeStore.settings.planFolder)}>
                  Apri
                </button>
              {/if}
            </div>
          </div>

          <div class="set-group">
            <h4>Gestione Sessioni Esistenti ({sessionStore.sessions.length})</h4>
            {#if sessionStore.sessions.length === 0}
              <p class="hint">Nessuna sessione salvata nel database.</p>
            {:else}
              <div class="order-list">
                {#each sessionStore.sessions as s (s.id)}
                  <div class="order-row">
                    <div class="sess-det">
                      <strong>{s.name}</strong>
                      <span>{s.startDate} → {s.endDate}</span>
                    </div>
                    <button type="button" class="icon-mini danger" onclick={() => sessionStore.deleteSession(s.id)} title="Elimina sessione">
                      <Trash2 size={14} />
                    </button>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {:else if activeTab === "ai"}
          <div class="set-group">
            <h4>Provider AI Attivo</h4>
            <p class="hint">Usa il modello locale leggero Qwen2.5-VL 3B oppure collega direttamente un Agente CLI installato sul tuo PC.</p>
            <div class="seg-row">
              {#each [["local", "Locale Base (Qwen2.5-VL 3B)"], ["cli_agy", "Antigravity CLI (agy)"], ["cli_claude", "Claude Code"], ["cli_codex", "Codex CLI"], ["cli_custom", "CLI Custom"]] as [id, label] (id)}
                <button
                  type="button"
                  class="pill-filter"
                  class:active={themeStore.settings.aiProvider === id}
                  onclick={() => themeStore.updateSettings({ aiProvider: id as AiProvider })}
                >
                  {label}
                </button>
              {/each}
            </div>
          </div>

          <!-- Stato Agenti CLI -->
          <div class="set-group">
            <h4>Agenti CLI Rilevati nel Sistema</h4>
            <div class="order-list">
              {#each cliAgents as cli (cli.id)}
                <div class="order-row">
                  <Terminal size={15} />
                  <div class="sess-det">
                    <strong>{cli.name} (<code>{cli.command || "non impostato"}</code>)</strong>
                    <span>{cli.installed ? `Trovato: ${cli.resolvedPath}` : "Non rilevato nel PATH"}</span>
                  </div>
                  <span class="status-pill" class:ok={cli.installed}>
                    {cli.installed ? "Pronto" : "Assente"}
                  </span>
                </div>
              {/each}
            </div>
            <div class="folder-row">
              <input
                type="text"
                class="soft-input"
                placeholder="Comando CLI Custom (es. my-agent --prompt &#123;prompt&#125;)"
                bind:value={customCliInput}
              />
              <button
                type="button"
                class="pill-btn pill-btn-soft"
                onclick={async () => {
                  await themeStore.updateSettings({ aiCustomCliCommand: customCliInput });
                  await refreshAiState();
                  navStore.showToast("Comando CLI salvato", "success");
                }}
              >
                Salva CLI
              </button>
            </div>
          </div>

          <!-- Modello Locale Base -->
          <div class="set-group">
            <h4>Modello Locale Base (Multimodale + OCR Orario)</h4>
            {#each aiModels as mod (mod.id)}
              <div class="local-model-card">
                <div class="lm-top">
                  <div>
                    <strong>{mod.name}</strong>
                    <p class="hint">{mod.tagline} · ~{mod.sizeGb} GB</p>
                  </div>
                  {#if mod.isDownloaded}
                    <span class="status-pill ok"><CheckCircle2 size={12} /> Scaricato</span>
                  {:else}
                    <span class="status-pill">Non scaricato</span>
                  {/if}
                </div>

                {#if downloadProgress && downloadProgress.modelId === mod.id && !downloadProgress.isCompleted && !downloadProgress.error}
                  <div class="dl-bar">
                    <div class="dl-fill" style:width="{downloadProgress.progressPct}%"></div>
                  </div>
                  <small>{downloadProgress.progressPct}% · {downloadProgress.speedMbPerSec} MB/s</small>
                {/if}

                <div class="lm-actions">
                  {#if !mod.isDownloaded}
                    <button type="button" class="pill-btn pill-btn-primary" onclick={() => ipc.startAiModelDownload(mod.id)}>
                      <Download size={14} /> Scarica Modello
                    </button>
                  {:else}
                    {#if runtimeStatus?.state === "ready"}
                      <button type="button" class="pill-btn pill-btn-soft" onclick={async () => { await ipc.stopAiRuntime(); await refreshAiState(); }}>
                        <Square size={13} /> Ferma Server
                      </button>
                    {:else}
                      <button type="button" class="pill-btn pill-btn-primary" onclick={async () => { await ipc.startAiRuntime(mod.id); await refreshAiState(); }}>
                        <Play size={13} /> Avvia Ora
                      </button>
                    {/if}
                    <button type="button" class="pill-btn pill-btn-soft" onclick={async () => { await ipc.deleteAiModel(mod.id); await refreshAiState(); }}>
                      <Trash2 size={13} /> Rimuovi
                    </button>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 140;
    background: rgba(12, 12, 18, 0.55);
    backdrop-filter: blur(8px);
    display: grid;
    place-items: center;
    padding: 20px;
  }

  .settings-modal {
    width: min(760px, 100%);
    max-height: 88vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .sm-head {
    padding: 20px 24px 12px;
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
  }

  .sm-head h2 {
    margin: 0;
    font-size: 20px;
    font-weight: 800;
  }

  .sm-head p {
    margin: 2px 0 0;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .close-btn {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: var(--bg-surface-muted);
    display: grid;
    place-items: center;
  }

  .sm-tabs {
    padding: 0 24px 14px;
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    border-bottom: 1px solid var(--border-subtle);
  }

  .sm-body {
    padding: 20px 24px 24px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .set-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .set-group h4 {
    margin: 0;
    font-size: 14px;
    font-weight: 700;
  }

  .hint {
    margin: 0;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .seg-row {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .cat-colors-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(135px, 1fr));
    gap: 8px;
  }

  .cat-color-chip {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    border-radius: 12px;
    background: var(--bg-surface-muted);
    font-size: 12.5px;
    font-weight: 600;
    cursor: pointer;
  }

  .cat-color-chip input[type="color"] {
    width: 24px;
    height: 24px;
    border: none;
    background: none;
    padding: 0;
    cursor: pointer;
  }

  .row-between {
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
  }

  .order-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .order-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    border-radius: 14px;
    background: var(--bg-surface-muted);
  }

  .ord-idx {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: var(--bg-surface);
    display: grid;
    place-items: center;
    font-size: 11px;
    font-weight: 700;
  }

  .ord-lbl,
  .sess-det {
    flex: 1;
    font-size: 13px;
  }

  .sess-det {
    display: flex;
    flex-direction: column;
  }

  .sess-det span {
    font-size: 11.5px;
    color: var(--text-secondary);
  }

  .ord-actions {
    display: flex;
    gap: 4px;
  }

  .icon-mini {
    width: 28px;
    height: 28px;
    border-radius: 8px;
    background: var(--bg-surface);
    display: grid;
    place-items: center;
  }

  .icon-mini:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .icon-mini.danger:hover {
    color: var(--danger);
  }

  .folder-row {
    display: flex;
    gap: 8px;
  }

  .soft-input {
    flex: 1;
    padding: 9px 12px;
    border-radius: 12px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface-muted);
    color: var(--text-primary);
    font-size: 12.5px;
  }

  .status-pill {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px 9px;
    border-radius: 999px;
    background: var(--bg-surface);
    font-size: 11px;
    font-weight: 700;
    color: var(--text-secondary);
  }

  .status-pill.ok {
    background: rgba(16, 185, 129, 0.16);
    color: #059669;
  }

  .local-model-card {
    padding: 14px;
    border-radius: 16px;
    background: var(--bg-surface-muted);
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .lm-top {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 10px;
  }

  .lm-actions {
    display: flex;
    gap: 8px;
  }

  .dl-bar {
    height: 7px;
    border-radius: 999px;
    background: var(--bg-surface);
    overflow: hidden;
  }

  .dl-fill {
    height: 100%;
    background: var(--accent);
  }
</style>
