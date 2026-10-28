<script lang="ts">
  import { onMount } from "svelte";
  import { themeStore, UI_SCALE_MAX, UI_SCALE_MIN } from "../../stores/theme.svelte";
  import type {
    AcademicSession,
    BackdropMaterial,
    CalendarViewMode,
    ThemeChoice,
    WindowControlsStyle
  } from "../../types/domain";

  interface Props {
    sessions?: AcademicSession[];
    activeSession?: AcademicSession | null;
    conflictCount?: number;
    isVoiceListening?: boolean;
    onSelectSession?: (sessionId: string) => void;
    onNewSession?: () => void;
    onEditSession?: () => void;
    onOpenConflicts?: () => void;
    onNavigateToday?: () => void;
    onNavigateStep?: (direction: -1 | 1) => void;
    onOpenCommandPalette?: (startWithVoice?: boolean) => void;
    onOpenSettings?: () => void;
  }

  let {
    sessions = [],
    activeSession = null,
    conflictCount = 0,
    isVoiceListening = false,
    onSelectSession,
    onNewSession,
    onEditSession,
    onOpenConflicts,
    onNavigateToday,
    onNavigateStep,
    onOpenCommandPalette,
    onOpenSettings
  }: Props = $props();

  let sessionMenuOpen = $state(false);
  let appearancePopoverOpen = $state(false);
  let appleMaxBtnEl = $state<HTMLButtonElement | null>(null);
  let winMaxBtnEl = $state<HTMLButtonElement | null>(null);

  const views: { id: CalendarViewMode; label: string; shortLabel: string }[] = [
    { id: "continuous", label: "Continua", shortLabel: "Infinita" },
    { id: "month", label: "Mese", shortLabel: "Mese" },
    { id: "week", label: "Settimana", shortLabel: "Sett." },
    { id: "agenda", label: "Agenda", shortLabel: "Appelli" }
  ];

  const materials: { id: BackdropMaterial; label: string; desc: string }[] = [
    { id: "mica", label: "Mica", desc: "Nativo Windows 11" },
    { id: "acrylic", label: "Acrylic", desc: "Vetro sfocato profondo" },
    { id: "tabbed", label: "Mica Alt", desc: "Superficie Tabbed" },
    { id: "solid", label: "Solido", desc: "Opaco ad alto contrasto" }
  ];

  const themes: { id: ThemeChoice; label: string }[] = [
    { id: "auto", label: "Auto (Win11)" },
    { id: "dark", label: "Scuro" },
    { id: "light", label: "Chiaro" }
  ];

  function syncMaxButtonBounds() {
    const activeEl =
      themeStore.settings.windowControlsStyle === "macos-left" ? appleMaxBtnEl : winMaxBtnEl;
    if (activeEl) {
      themeStore.registerMaximizeButtonElement(activeEl);
    }
  }

  $effect(() => {
    const _style = themeStore.settings.windowControlsStyle;
    const _scale = themeStore.settings.uiScale;
    if (_style && _scale) {
      queueMicrotask(syncMaxButtonBounds);
    }
  });

  onMount(() => {
    syncMaxButtonBounds();
    const handleResize = () => syncMaxButtonBounds();
    const handleGlobalClick = (e: MouseEvent) => {
      const target = e.target as HTMLElement | null;
      if (!target) return;
      if (!target.closest(".session-dropdown-anchor")) {
        sessionMenuOpen = false;
      }
      if (!target.closest(".appearance-dropdown-anchor")) {
        appearancePopoverOpen = false;
      }
    };

    window.addEventListener("resize", handleResize);
    window.addEventListener("pointerdown", handleGlobalClick);
    return () => {
      window.removeEventListener("resize", handleResize);
      window.removeEventListener("pointerdown", handleGlobalClick);
    };
  });

  function handleSelectSession(id: string) {
    sessionMenuOpen = false;
    onSelectSession?.(id);
  }

  function handleViewChange(view: CalendarViewMode) {
    themeStore.setActiveView(view);
  }

  function handleScaleSlider(e: Event) {
    const val = parseFloat((e.currentTarget as HTMLInputElement).value);
    if (!Number.isNaN(val)) {
      themeStore.setUiScale(val);
    }
  }

  function handleControlStyleChange(style: WindowControlsStyle) {
    themeStore.setWindowControlsStyle(style);
  }
</script>

<header
  class="unified-toolbar"
  role="toolbar"
  aria-label="Barra degli strumenti unificata MySession"
  tabindex="-1"
  data-tauri-drag-region
  ondblclick={(e) => {
    if (e.target === e.currentTarget) {
      themeStore.toggleMaximizeWindow();
    }
  }}
>
  <!-- SEZIONE SINISTRA: Semaforo Apple (se selezionato) + Selettore Sessione + Navigazione Temporale -->
  <div class="toolbar-left" data-tauri-drag-region>
    {#if themeStore.settings.windowControlsStyle === "macos-left"}
      <div class="apple-traffic-lights" role="group" aria-label="Controlli finestra stile macOS">
        <button
          type="button"
          class="traffic-btn close"
          title="Chiudi MySession"
          aria-label="Chiudi finestra"
          onclick={() => themeStore.closeWindow()}
        >
          <svg viewBox="0 0 10 10" aria-hidden="true">
            <path d="M2.2 2.2L7.8 7.8M7.8 2.2L2.2 7.8" stroke="currentColor" stroke-width="1.35" stroke-linecap="round" />
          </svg>
        </button>
        <button
          type="button"
          class="traffic-btn minimize"
          title="Riduci a icona"
          aria-label="Riduci a icona"
          onclick={() => themeStore.minimizeWindow()}
        >
          <svg viewBox="0 0 10 10" aria-hidden="true">
            <path d="M2 5H8" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
          </svg>
        </button>
        <button
          bind:this={appleMaxBtnEl}
          type="button"
          class="traffic-btn maximize"
          title={themeStore.isMaximized ? "Ripristina dimensioni (Snap Layouts)" : "Massimizza (Snap Layouts)"}
          aria-label="Massimizza o ripristina finestra"
          onclick={() => themeStore.toggleMaximizeWindow()}
        >
          <svg viewBox="0 0 10 10" aria-hidden="true">
            <path d="M2.4 5H7.6M5 2.4V7.6" stroke="currentColor" stroke-width="1.35" stroke-linecap="round" />
          </svg>
        </button>
      </div>
      <div class="toolbar-divider" aria-hidden="true"></div>
    {/if}

    <!-- Selettore Rapido Sessione Accademica -->
    <div class="session-dropdown-anchor">
      <button
        type="button"
        class="session-selector-pill pressable"
        onclick={() => (sessionMenuOpen = !sessionMenuOpen)}
        aria-expanded={sessionMenuOpen}
        title="Cambia Sessione Accademica attiva"
      >
        <span class="session-dot" class:has-conflict={conflictCount > 0}></span>
        <span class="session-title-wrap">
          <span class="session-name">
            {activeSession ? activeSession.name : "Nessuna Sessione"}
          </span>
          {#if activeSession?.academicYear}
            <span class="session-year">{activeSession.academicYear}</span>
          {/if}
        </span>
        <svg class="chevron-icon" class:rotated={sessionMenuOpen} viewBox="0 0 16 16" fill="none">
          <path d="M4.5 6.25L8 9.75L11.5 6.25" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </button>

      {#if sessionMenuOpen}
        <div class="popover-menu session-popover" role="menu">
          <div class="popover-section-header">Sessioni Accademiche</div>
          {#if sessions.length === 0}
            <div class="popover-empty">Nessuna sessione configurata</div>
          {:else}
            {#each sessions as s (s.id)}
              <button
                type="button"
                role="menuitem"
                class="popover-item"
                class:active={activeSession?.id === s.id}
                onclick={() => handleSelectSession(s.id)}
              >
                <span class="item-main">
                  <span class="item-name">{s.name}</span>
                  <span class="item-sub">{s.academicYear} · {s.sessionType}</span>
                </span>
                {#if activeSession?.id === s.id}
                  <span class="check-badge">Attiva</span>
                {/if}
              </button>
            {/each}
          {/if}
          <div class="popover-separator"></div>
          {#if activeSession}
            <button
              type="button"
              class="popover-item"
              onclick={() => {
                sessionMenuOpen = false;
                onEditSession?.();
              }}
            >
              <span>✎ Modifica Sessione Attiva…</span>
            </button>
          {/if}
          <button
            type="button"
            class="popover-item action-item"
            onclick={() => {
              sessionMenuOpen = false;
              onNewSession?.();
            }}
          >
            <span>+ Nuova Sessione Accademica…</span>
          </button>
        </div>
      {/if}
    </div>

    <!-- Navigazione Temporale Rapida -->
    <div class="temporal-nav-group" role="group" aria-label="Navigazione temporale calendario">
      <button
        type="button"
        class="nav-icon-btn pressable"
        title="Periodo precedente"
        aria-label="Periodo precedente"
        onclick={() => onNavigateStep?.(-1)}
      >
        <svg viewBox="0 0 16 16" fill="none">
          <path d="M9.75 4.25L6 8L9.75 11.75" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </button>
      <button
        type="button"
        class="nav-today-btn pressable"
        title="Salta alla data odierna (T)"
        onclick={() => onNavigateToday?.()}
      >
        Oggi
      </button>
      <button
        type="button"
        class="nav-icon-btn pressable"
        title="Periodo successivo"
        aria-label="Periodo successivo"
        onclick={() => onNavigateStep?.(1)}
      >
        <svg viewBox="0 0 16 16" fill="none">
          <path d="M6.25 4.25L10 8L6.25 11.75" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </button>
    </div>
  </div>

  <!-- SEZIONE CENTRALE: Area di trascinamento libera della finestra desktop nativa -->
  <div class="toolbar-center" data-tauri-drag-region></div>

  <!-- SEZIONE DESTRA: Segmented Control Viste + Controllo Aspetto/Zoom + Controlli Windows 11 (se selezionati) -->
  <div class="toolbar-right" data-tauri-drag-region>
    {#if conflictCount > 0}
      <button
        type="button"
        class="conflict-pill-indicator pressable"
        title="Ispeziona {conflictCount} conflitti o buffer critici rilevati"
        onclick={() => onOpenConflicts?.()}
      >
        <span class="conflict-pulse"></span>
        <span>{conflictCount} {conflictCount === 1 ? "Conflitto" : "Conflitti"}</span>
      </button>
    {/if}

    <!-- Segmented Control 4 Viste Ortogonali -->
    <div class="view-segmented-control" role="tablist" aria-label="Vista Calendario">
      {#each views as v (v.id)}
        <button
          type="button"
          role="tab"
          aria-selected={themeStore.settings.defaultView === v.id}
          class="segment-btn pressable"
          class:active={themeStore.settings.defaultView === v.id}
          onclick={() => handleViewChange(v.id)}
        >
          {v.label}
        </button>
      {/each}
    </div>

    <!-- Popover Aspetto, Materiali DWM & Zoom Scalabile -->
    <div class="appearance-dropdown-anchor">
      <button
        type="button"
        class="toolbar-utility-btn pressable"
        class:active={appearancePopoverOpen}
        title="Aspetto, Materiale DWM (Mica/Acrylic) e Zoom ({themeStore.uiScalePercent}%)"
        onclick={() => (appearancePopoverOpen = !appearancePopoverOpen)}
      >
        <svg viewBox="0 0 16 16" fill="none">
          <circle cx="8" cy="8" r="5.75" stroke="currentColor" stroke-width="1.35" />
          <path d="M8 2.25V13.75" stroke="currentColor" stroke-width="1.35" />
        </svg>
        <span class="scale-readout">{themeStore.uiScalePercent}%</span>
      </button>

      {#if appearancePopoverOpen}
        <div class="popover-menu appearance-popover" role="dialog" aria-label="Impostazioni rapide finestra e tema">
          <!-- Tema Chiaro / Scuro / Auto -->
          <div class="popover-section-header">Tema Sistema (Win11 Sync)</div>
          <div class="mini-segmented-row">
            {#each themes as t (t.id)}
              <button
                type="button"
                class="mini-seg-btn pressable"
                class:active={themeStore.settings.theme === t.id}
                onclick={() => themeStore.setThemeMode(t.id)}
              >
                {t.label}
              </button>
            {/each}
          </div>

          <!-- Materiale DWM Nativo -->
          <div class="popover-section-header">Materiale DWM (Win32 Backdrop)</div>
          <div class="material-grid">
            {#each materials as m (m.id)}
              <button
                type="button"
                class="material-card-btn pressable"
                class:active={themeStore.settings.backdropMaterial === m.id}
                onclick={() => themeStore.setWindowMaterial(m.id)}
              >
                <span class="mat-name">{m.label}</span>
                <span class="mat-desc">{m.desc}</span>
              </button>
            {/each}
          </div>

          <!-- Stile Controlli Finestra -->
          <div class="popover-section-header">Stile Controlli Finestra</div>
          <div class="mini-segmented-row">
            <button
              type="button"
              class="mini-seg-btn pressable"
              class:active={themeStore.settings.windowControlsStyle === "macos-left"}
              onclick={() => handleControlStyleChange("macos-left")}
            >
              Apple Semaforo (Sx)
            </button>
            <button
              type="button"
              class="mini-seg-btn pressable"
              class:active={themeStore.settings.windowControlsStyle === "win11-right"}
              onclick={() => handleControlStyleChange("win11-right")}
            >
              Windows 11 (Dx)
            </button>
          </div>

          <!-- Zoom Scalabile (80% - 150%) -->
          <div class="popover-section-header">
            <span>Zoom Interfaccia (80% – 150%)</span>
            <span class="zoom-badge">{themeStore.uiScalePercent}%</span>
          </div>
          <div class="zoom-controls-row">
            <button
              type="button"
              class="zoom-step-btn pressable"
              onclick={() => themeStore.zoomOut()}
              disabled={themeStore.settings.uiScale <= UI_SCALE_MIN}
              aria-label="Riduci zoom"
            >
              −
            </button>
            <input
              type="range"
              class="zoom-slider"
              min={UI_SCALE_MIN}
              max={UI_SCALE_MAX}
              step="5"
              value={themeStore.settings.uiScale}
              oninput={handleScaleSlider}
            />
            <button
              type="button"
              class="zoom-step-btn pressable"
              onclick={() => themeStore.zoomIn()}
              disabled={themeStore.settings.uiScale >= UI_SCALE_MAX}
              aria-label="Aumenta zoom"
            >
              +
            </button>
            <button
              type="button"
              class="zoom-reset-btn pressable"
              onclick={() => themeStore.resetZoom()}
              title="Ripristina 100% (Ctrl+0)"
            >
              100%
            </button>
          </div>
        </div>
      {/if}
    </div>

    <!-- Pulsante Impostazioni Complete -->
    <button
      type="button"
      class="toolbar-icon-only-btn pressable"
      title="Impostazioni MySession"
      aria-label="Apri impostazioni"
      onclick={() => onOpenSettings?.()}
    >
      <svg viewBox="0 0 16 16" fill="none">
        <circle cx="8" cy="8" r="2.1" stroke="currentColor" stroke-width="1.35" />
        <path
          d="M8 1.75V3.1M8 12.9V14.25M1.75 8H3.1M12.9 8H14.25M3.58 3.58L4.54 4.54M11.46 11.46L12.42 12.42M12.42 3.58L11.46 4.54M4.54 11.46L3.58 12.42"
          stroke="currentColor"
          stroke-width="1.35"
          stroke-linecap="round"
        />
      </svg>
    </button>

    <!-- Controlli Finestra Stile Windows 11 a Destra (se selezionato) -->
    {#if themeStore.settings.windowControlsStyle === "win11-right"}
      <div class="win11-window-controls" role="group" aria-label="Controlli finestra Windows 11">
        <button
          type="button"
          class="win11-btn minimize"
          title="Riduci a icona"
          aria-label="Riduci a icona"
          onclick={() => themeStore.minimizeWindow()}
        >
          <svg width="10" height="10" viewBox="0 0 10 10">
            <path d="M1 5H9" stroke="currentColor" stroke-width="1" />
          </svg>
        </button>
        <button
          bind:this={winMaxBtnEl}
          type="button"
          class="win11-btn maximize"
          title={themeStore.isMaximized ? "Ripristina (Snap Layouts)" : "Massimizza (Snap Layouts)"}
          aria-label="Massimizza o ripristina finestra"
          onclick={() => themeStore.toggleMaximizeWindow()}
        >
          {#if themeStore.isMaximized}
            <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
              <rect x="1.5" y="3" width="5.5" height="5.5" stroke="currentColor" stroke-width="1" />
              <path d="M3 3V1.5H8.5V7H7" stroke="currentColor" stroke-width="1" />
            </svg>
          {:else}
            <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
              <rect x="1.5" y="1.5" width="7" height="7" stroke="currentColor" stroke-width="1" />
            </svg>
          {/if}
        </button>
        <button
          type="button"
          class="win11-btn close"
          title="Chiudi MySession"
          aria-label="Chiudi finestra"
          onclick={() => themeStore.closeWindow()}
        >
          <svg width="10" height="10" viewBox="0 0 10 10">
            <path d="M1.5 1.5L8.5 8.5M8.5 1.5L1.5 8.5" stroke="currentColor" stroke-width="1.1" />
          </svg>
        </button>
      </div>
    {/if}
  </div>
</header>

<style>
  .unified-toolbar {
    position: relative;
    z-index: 100;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.75rem;
    height: 3.25rem;
    padding: 0 0.85rem;
    background: var(--toolbar-bg);
    backdrop-filter: blur(28px) saturate(180%);
    -webkit-backdrop-filter: blur(28px) saturate(180%);
    border-bottom: 0.5px solid var(--hairline-border);
    box-shadow: inset 0 0.5px 0 rgba(255, 255, 255, 0.08);
    user-select: none;
    -webkit-user-select: none;
  }

  :global([data-controls="win11-right"]) .unified-toolbar {
    padding-right: 0;
  }

  .toolbar-left,
  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }

  .toolbar-right {
    justify-content: flex-end;
  }

  .toolbar-center {
    display: flex;
    justify-content: center;
    align-items: center;
    min-width: 0;
  }

  /* Micro-interazioni a molla istantanee Apple HIG (Emil Kowalski feedback) */
  .pressable {
    will-change: transform;
    transition:
      transform 120ms var(--spring-press),
      background-color 130ms ease,
      border-color 130ms ease,
      box-shadow 130ms ease,
      color 130ms ease;
  }

  .pressable:active {
    transform: scale(0.97);
  }

  /* Focus ring rifinito macOS per accessibilità WCAG */
  button:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px var(--accent-primary), 0 0 0 4px var(--accent-soft);
  }

  /* ==========================================================================
     1. Apple Traffic Lights (macOS Native Left Controls)
     ========================================================================== */
  .apple-traffic-lights {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.25rem 0.2rem;
    flex-shrink: 0;
  }

  .traffic-btn {
    position: relative;
    width: 12px;
    height: 12px;
    padding: 0;
    border-radius: var(--radius-pill);
    border: 0.5px solid rgba(0, 0, 0, 0.16);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    color: rgba(0, 0, 0, 0.65);
    transition: transform 120ms var(--spring-press), filter 120ms ease;
  }

  .traffic-btn:active {
    transform: scale(0.92);
  }

  .traffic-btn svg {
    width: 8px;
    height: 8px;
    opacity: 0;
    transition: opacity 120ms var(--ease-out);
  }

  .apple-traffic-lights:hover .traffic-btn svg {
    opacity: 1;
  }

  .traffic-btn.close {
    background: #ff5f56;
  }

  .traffic-btn.minimize {
    background: #ffbd2e;
  }

  .traffic-btn.maximize {
    background: #27c93f;
  }

  .toolbar-divider {
    width: 0.5px;
    height: 1.15rem;
    background: var(--hairline-border);
    margin: 0 0.2rem;
    flex-shrink: 0;
  }

  /* ==========================================================================
     2. Session Selector & Temporal Navigation
     ========================================================================== */
  .session-dropdown-anchor,
  .appearance-dropdown-anchor {
    position: relative;
  }

  .session-selector-pill {
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
    height: 2.05rem;
    padding: 0 0.75rem;
    border-radius: var(--radius-sm);
    background: var(--surface-pill);
    border: 0.5px solid var(--hairline-border);
    box-shadow: var(--shadow-pill);
    color: var(--text-primary);
    font-size: 0.8125rem;
    font-weight: 600;
    letter-spacing: var(--tracking-normal);
    cursor: pointer;
    max-width: 15rem;
  }

  .session-selector-pill:hover {
    background: var(--surface-pill-hover);
    border-color: var(--hairline-border-strong);
  }

  .session-dot {
    width: 7px;
    height: 7px;
    border-radius: var(--radius-pill);
    background: var(--accent-primary);
    box-shadow: 0 0 8px var(--accent-glow);
    flex-shrink: 0;
  }

  .session-dot.has-conflict {
    background: #ef4444;
    box-shadow: 0 0 8px rgba(239, 68, 68, 0.55);
    animation: conflictPing 2s infinite cubic-bezier(0, 0, 0.2, 1);
  }

  @keyframes conflictPing {
    0%, 100% {
      transform: scale(1);
      opacity: 1;
    }
    50% {
      transform: scale(1.18);
      opacity: 0.85;
    }
  }

  .session-title-wrap {
    display: flex;
    align-items: baseline;
    gap: 0.35rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .session-name {
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .session-year {
    font-size: 0.7rem;
    font-weight: 500;
    color: var(--text-secondary);
    opacity: 0.85;
  }

  .chevron-icon {
    width: 0.85rem;
    height: 0.85rem;
    color: var(--text-secondary);
    flex-shrink: 0;
    transition: transform 180ms var(--spring-snappy);
  }

  .chevron-icon.rotated {
    transform: rotate(180deg);
  }

  .temporal-nav-group {
    display: inline-flex;
    align-items: center;
    background: var(--surface-pill);
    border: 0.5px solid var(--hairline-border);
    box-shadow: var(--shadow-pill);
    border-radius: var(--radius-sm);
    padding: 0.12rem;
    gap: 0.1rem;
    flex-shrink: 0;
  }

  .nav-icon-btn {
    width: 1.75rem;
    height: 1.75rem;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    border-radius: var(--radius-xs);
    cursor: pointer;
  }

  .nav-icon-btn:hover {
    background: var(--surface-pill-hover);
    color: var(--text-primary);
  }

  .nav-icon-btn:active {
    transform: scale(0.94);
  }

  .nav-icon-btn svg {
    width: 0.9rem;
    height: 0.9rem;
  }

  .nav-today-btn {
    height: 1.75rem;
    padding: 0 0.65rem;
    border: none;
    background: transparent;
    color: var(--text-primary);
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: var(--tracking-wide);
    border-radius: var(--radius-xs);
    cursor: pointer;
  }

  .nav-today-btn:hover {
    background: var(--surface-pill-hover);
  }

  .nav-today-btn:active {
    transform: scale(0.96);
  }

  

  /* ==========================================================================
     4. Right Controls: Conflict Badge, Segmented Views, Appearance Popover, Win11 Controls
     ========================================================================== */
  .conflict-pill-indicator {
    display: inline-flex;
    align-items: center;
    gap: 0.38rem;
    height: 1.95rem;
    padding: 0 0.65rem;
    border-radius: var(--radius-pill);
    background: rgba(239, 68, 68, 0.12);
    border: 0.5px solid rgba(239, 68, 68, 0.35);
    color: #f87171;
    font-size: 0.73rem;
    font-weight: 600;
    letter-spacing: var(--tracking-normal);
    flex-shrink: 0;
  }

  .conflict-pulse {
    width: 6px;
    height: 6px;
    border-radius: var(--radius-pill);
    background: #ef4444;
  }

  .view-segmented-control {
    display: inline-flex;
    align-items: center;
    background: var(--surface-pill);
    border: 0.5px solid var(--hairline-border);
    box-shadow: var(--shadow-pill);
    border-radius: var(--radius-sm);
    padding: 0.15rem;
    gap: 0.12rem;
    flex-shrink: 0;
  }

  .segment-btn {
    height: 1.7rem;
    padding: 0 0.62rem;
    border: none;
    border-radius: var(--radius-xs);
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.74rem;
    font-weight: 500;
    letter-spacing: var(--tracking-normal);
    cursor: pointer;
  }

  .segment-btn:hover {
    color: var(--text-primary);
  }

  .segment-btn.active {
    background: var(--segment-active-bg);
    color: var(--text-primary);
    font-weight: 600;
    box-shadow: var(--shadow-segment-active);
  }

  .toolbar-utility-btn,
  .toolbar-icon-only-btn {
    height: 2.05rem;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.35rem;
    padding: 0 0.65rem;
    border-radius: var(--radius-sm);
    background: var(--surface-pill);
    border: 0.5px solid var(--hairline-border);
    box-shadow: var(--shadow-pill);
    color: var(--text-secondary);
    font-size: 0.73rem;
    font-weight: 600;
    letter-spacing: var(--tracking-normal);
    cursor: pointer;
    flex-shrink: 0;
  }

  .toolbar-icon-only-btn {
    width: 2.05rem;
    padding: 0;
  }

  .toolbar-utility-btn:hover,
  .toolbar-utility-btn.active,
  .toolbar-icon-only-btn:hover {
    background: var(--surface-pill-hover);
    color: var(--text-primary);
    border-color: var(--hairline-border-strong);
  }

  .toolbar-utility-btn svg,
  .toolbar-icon-only-btn svg {
    width: 0.92rem;
    height: 0.92rem;
  }

  .scale-readout {
    font-variant-numeric: tabular-nums;
  }

  /* ==========================================================================
     5. Origin-Aware Floating Popovers (Apple HIG Vibrancy)
     ========================================================================== */
  .popover-menu {
    position: absolute;
    top: calc(100% + 0.5rem);
    min-width: 16rem;
    padding: 0.5rem;
    border-radius: var(--radius-lg);
    background: var(--popover-bg);
    backdrop-filter: blur(32px) saturate(200%);
    -webkit-backdrop-filter: blur(32px) saturate(200%);
    border: 0.5px solid var(--hairline-border-strong);
    box-shadow: var(--shadow-popover);
    z-index: 300;
    animation: popoverEnter 150ms var(--spring-snappy);
  }

  .session-popover {
    left: 0;
    transform-origin: top left;
  }

  .appearance-popover {
    right: 0;
    width: 19rem;
    padding: 0.75rem;
    transform-origin: top right;
  }

  @keyframes popoverEnter {
    from {
      opacity: 0;
      transform: scale(0.96);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .popover-section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 0.68rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: var(--tracking-caps);
    color: var(--text-tertiary);
    padding: 0.35rem 0.45rem 0.35rem;
  }

  .popover-item {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.48rem 0.65rem;
    border: none;
    border-radius: var(--radius-xs);
    background: transparent;
    color: var(--text-primary);
    text-align: left;
    cursor: pointer;
    font-size: 0.78rem;
  }

  .popover-item:hover {
    background: var(--surface-pill-hover);
  }

  .popover-item.active {
    background: var(--accent-soft);
  }

  .item-main {
    display: flex;
    flex-direction: column;
    gap: 0.12rem;
  }

  .item-name {
    font-weight: 600;
  }

  .item-sub {
    font-size: 0.68rem;
    color: var(--text-secondary);
  }

  .check-badge {
    font-size: 0.66rem;
    font-weight: 600;
    padding: 0.12rem 0.45rem;
    border-radius: var(--radius-pill);
    background: var(--accent-primary);
    color: #fff;
    letter-spacing: var(--tracking-wide);
  }

  .popover-separator {
    height: 0.5px;
    background: var(--hairline-border);
    margin: 0.4rem 0;
  }

  .action-item {
    color: var(--accent-primary);
    font-weight: 600;
  }

  .mini-segmented-row {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(70px, 1fr));
    gap: 0.22rem;
    padding: 0.16rem;
    background: var(--surface-pill);
    border: 0.5px solid var(--hairline-border);
    border-radius: var(--radius-sm);
    margin-bottom: 0.55rem;
  }

  .mini-seg-btn {
    height: 1.68rem;
    border: none;
    border-radius: var(--radius-xs);
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.72rem;
    font-weight: 600;
    cursor: pointer;
  }

  .mini-seg-btn:active {
    transform: scale(0.96);
  }

  .mini-seg-btn.active {
    background: var(--segment-active-bg);
    color: var(--text-primary);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.12);
  }

  .material-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.35rem;
    margin-bottom: 0.55rem;
  }

  .material-card-btn {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.12rem;
    padding: 0.48rem 0.6rem;
    border-radius: var(--radius-sm);
    background: var(--surface-pill);
    border: 0.5px solid var(--hairline-border);
    color: var(--text-primary);
    cursor: pointer;
    text-align: left;
  }

  .material-card-btn:active {
    transform: scale(0.97);
  }

  .material-card-btn.active {
    border-color: var(--accent-primary);
    background: var(--accent-soft);
  }

  .mat-name {
    font-size: 0.75rem;
    font-weight: 600;
  }

  .mat-desc {
    font-size: 0.64rem;
    color: var(--text-secondary);
  }

  .zoom-badge {
    color: var(--accent-primary);
    font-variant-numeric: tabular-nums;
  }

  .zoom-controls-row {
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }

  .zoom-step-btn,
  .zoom-reset-btn {
    height: 1.68rem;
    padding: 0 0.52rem;
    border-radius: var(--radius-xs);
    border: 0.5px solid var(--hairline-border);
    background: var(--surface-pill);
    color: var(--text-primary);
    font-size: 0.74rem;
    font-weight: 600;
    cursor: pointer;
  }

  .zoom-step-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .zoom-slider {
    flex: 1;
    accent-color: var(--accent-primary);
    cursor: pointer;
  }

  /* ==========================================================================
     6. Windows 11 Native Right Window Controls
     ========================================================================== */
  .win11-window-controls {
    display: inline-flex;
    align-items: stretch;
    height: 3.25rem;
    margin-left: 0.25rem;
    flex-shrink: 0;
  }

  .win11-btn {
    width: 46px;
    height: 100%;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: background-color 110ms ease, color 110ms ease;
  }

  .win11-btn:hover {
    background: var(--surface-pill-hover);
    color: var(--text-primary);
  }

  .win11-btn.close:hover {
    background: #e81123;
    color: #ffffff;
  }
</style>
