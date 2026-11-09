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
    ignoredConflictCount?: number;
    isVoiceListening?: boolean;
    isSessionLocked?: boolean;
    onSelectSession?: (sessionId: string) => void;
    onNewSession?: () => void;
    onEditSession?: () => void;
    onManageSessions?: () => void;
    onOpenConflicts?: () => void;
    onToggleSessionLock?: () => void;
    onNavigateToday?: () => void;
    onNavigateStep?: (direction: -1 | 1) => void;
    onOpenCommandPalette?: (startWithVoice?: boolean) => void;
    onOpenSettings?: () => void;
    onOpenTimetable?: () => void;
  }

  let {
    sessions = [],
    activeSession = null,
    conflictCount = 0,
    ignoredConflictCount = 0,
    isSessionLocked = false,
    onSelectSession,
    onNewSession,
    onEditSession,
    onManageSessions,
    onOpenConflicts,
    onToggleSessionLock,
    onNavigateToday,
    onNavigateStep,
    onOpenSettings,
    onOpenTimetable
  }: Props = $props();

  let toolbarEl = $state<HTMLElement | null>(null);
  let toolbarWidth = $state<number>(1200);
  let remUnit = $state<number>(16);

  let sessionMenuOpen = $state(false);
  let appearancePopoverOpen = $state(false);
  let overflowMenuOpen = $state(false);
  let appleMaxBtnEl = $state<HTMLButtonElement | null>(null);
  let winMaxBtnEl = $state<HTMLButtonElement | null>(null);

  function updateToolbarDimensions() {
    if (!toolbarEl) return;
    toolbarWidth = toolbarEl.clientWidth;
    if (typeof document !== "undefined") {
      const rootFs = parseFloat(getComputedStyle(document.documentElement).fontSize);
      if (rootFs > 0) {
        remUnit = rootFs;
      }
    }
  }

  let widthInRem = $derived<number>(toolbarWidth / (remUnit > 0 ? remUnit : 16));

  let overflowTier = $derived.by<number>(() => {
    const w = widthInRem;
    const isWinControlsRight = themeStore.settings.windowControlsStyle === "win11-right";
    const winBuffer = isWinControlsRight ? 8.5 : 0;

    // Livello 0: Spazio ampio (> 1300px a scala standard), tutto esteso
    if (w >= 74 + winBuffer) return 0;
    // Livello 1: Compattazione testi (anno sessione, testo conflitti/blocco, % zoom)
    if (w >= 62 + winBuffer) return 1;
    // Livello 2: Viste a icone SVG Apple HIG
    if (w >= 52 + winBuffer) return 2;
    // Livello 3: Utility (Zoom, Orario, Impostazioni) spostate nel menu a tendina
    if (w >= 41 + winBuffer) return 3;
    // Livello 4: Blocco sessione e Conflitti spostati anch'essi nel menu a tendina
    return 4;
  });

  $effect(() => {
    if (overflowTier < 3) {
      overflowMenuOpen = false;
    }
  });

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
    if ((_style || _scale) && toolbarEl) {
      queueMicrotask(() => {
        updateToolbarDimensions();
        syncMaxButtonBounds();
      });
    }
  });

  onMount(() => {
    updateToolbarDimensions();
    syncMaxButtonBounds();
    const handleResize = () => {
      updateToolbarDimensions();
      syncMaxButtonBounds();
    };
    const handleGlobalClick = (e: MouseEvent) => {
      const target = e.target as HTMLElement | null;
      if (!target) return;
      if (!target.closest(".session-dropdown-anchor")) {
        sessionMenuOpen = false;
      }
      if (!target.closest(".appearance-dropdown-anchor")) {
        appearancePopoverOpen = false;
      }
      if (!target.closest(".overflow-dropdown-anchor")) {
        overflowMenuOpen = false;
      }
    };

    window.addEventListener("resize", handleResize);
    window.addEventListener("pointerdown", handleGlobalClick);

    let ro: ResizeObserver | null = null;
    if (toolbarEl && typeof ResizeObserver !== "undefined") {
      ro = new ResizeObserver(() => {
        updateToolbarDimensions();
        syncMaxButtonBounds();
      });
      ro.observe(toolbarEl);
    }

    return () => {
      window.removeEventListener("resize", handleResize);
      window.removeEventListener("pointerdown", handleGlobalClick);
      if (ro) ro.disconnect();
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

  let lastWheelTime = 0;
  const WHEEL_THROTTLE_MS = 150;

  function handleSessionWheel(e: WheelEvent) {
    if (sessions.length <= 1) return;
    e.preventDefault();

    const now = performance.now();
    if (now - lastWheelTime < WHEEL_THROTTLE_MS) return;
    lastWheelTime = now;

    const currentIndex = sessions.findIndex((s) => s.id === activeSession?.id);
    const total = sessions.length;

    if (e.deltaY > 0) {
      // Rotella in giù -> sessione successiva (ciclica)
      const nextIndex = currentIndex === -1 ? 0 : (currentIndex + 1) % total;
      onSelectSession?.(sessions[nextIndex].id);
    } else if (e.deltaY < 0) {
      // Rotella in su -> sessione precedente (ciclica)
      const prevIndex = currentIndex === -1 ? 0 : (currentIndex - 1 + total) % total;
      onSelectSession?.(sessions[prevIndex].id);
    }
  }
</script>

<header
  bind:this={toolbarEl}
  class="unified-toolbar tier-{overflowTier}"
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
        onwheel={handleSessionWheel}
        aria-expanded={sessionMenuOpen}
        title="Cambia Sessione Accademica attiva (scorri con la rotella del mouse per passare rapidamente da una sessione all'altra)"
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
              onOpenTimetable?.();
            }}
          >
            <span>📅 Orario Settimanale Lezioni…</span>
          </button>
          <button
            type="button"
            class="popover-item action-item"
            onclick={() => {
              sessionMenuOpen = false;
              onManageSessions?.();
            }}
          >
            <span>⚙ Gestisci tutte le sessioni…</span>
          </button>
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

  <!-- SEZIONE DESTRA: Segmented Control Viste + Controllo Aspetto/Zoom + Overflow Dropdown + Controlli Finestra -->
  <div class="toolbar-right" data-tauri-drag-region>
    {#if overflowTier < 4}
      {#if conflictCount > 0}
        <button
          type="button"
          class="conflict-pill-indicator pressable"
          title="Ispeziona {conflictCount} {conflictCount === 1 ? 'conflitto attivo' : 'conflitti attivi'} rilevati"
          onclick={() => onOpenConflicts?.()}
        >
          <span class="conflict-pulse"></span>
          <span class="conflict-full-text">{conflictCount} {conflictCount === 1 ? "Conflitto" : "Conflitti"}</span>
          <span class="conflict-compact-badge">{conflictCount}</span>
        </button>
      {:else if ignoredConflictCount > 0}
        <button
          type="button"
          class="conflict-pill-indicator ignored-pill pressable"
          title="Nessun conflitto attivo ({ignoredConflictCount} {ignoredConflictCount === 1 ? 'conflitto ignorato' : 'conflitti ignorati'}). Clicca per visualizzare o ripristinare."
          onclick={() => onOpenConflicts?.()}
        >
          <span class="ignored-icon">🙈</span>
          <span class="conflict-full-text">0 Conflitti · {ignoredConflictCount} {ignoredConflictCount === 1 ? "ignorato" : "ignorati"}</span>
          <span class="conflict-compact-badge">{ignoredConflictCount}</span>
        </button>
      {/if}

      <!-- Toggle Blocco Periodo Sessione / Scorrimento Infinito con Lucchetto Animato -->
      <button
        type="button"
        class="session-lock-toggle pressable"
        class:is-locked={isSessionLocked}
        class:has-no-session={!activeSession}
        disabled={!activeSession}
        aria-label={isSessionLocked ? "Blocco periodo sessione attivo" : "Scorrimento infinito sbloccato"}
        title={!activeSession
          ? "Nessuna sessione attiva"
          : isSessionLocked
            ? "Blocco sessione ATTIVO: visualizzazione limitata al periodo della sessione. Clicca per sbloccare lo scorrimento infinito."
            : "Blocco sessione DISATTIVO: puoi scorrere all'infinito nel calendario. Clicca per bloccare il calendario sul periodo della sessione."}
        onclick={() => onToggleSessionLock?.()}
      >
        <div class="lock-icon-container" aria-hidden="true">
          <svg class="lock-svg" viewBox="0 0 16 16" fill="none">
            <!-- Archetto lucchetto animato con fisica a molla -->
            <path
              class="lock-shackle"
              d="M4.75 6.5V4.25C4.75 2.45 6.2 1.25 8 1.25C9.8 1.25 11.25 2.45 11.25 4.25V6.5"
              stroke="currentColor"
              stroke-width="1.45"
              stroke-linecap="round"
            />
            <!-- Corpo lucchetto stile squircle Apple HIG -->
            <rect
              class="lock-body"
              x="3"
              y="6.25"
              width="10"
              height="8"
              rx="2.2"
              stroke="currentColor"
              stroke-width="1.35"
            />
            <!-- Dettaglio serratura centrale -->
            <circle cx="8" cy="9.6" r="0.9" fill="currentColor" />
            <path d="M8 10.5V11.8" stroke="currentColor" stroke-width="1.15" stroke-linecap="round" />
          </svg>
        </div>
        <span class="lock-status-badge">
          {isSessionLocked ? "Sessione Bloccata" : "Scorrimento Libero"}
        </span>
      </button>
    {/if}

    <!-- Segmented Control 4 Viste Ortogonali (Sempre visibile sulla barra principale) -->
    <div class="view-segmented-control" role="tablist" aria-label="Vista Calendario">
      {#each views as v (v.id)}
        <button
          type="button"
          role="tab"
          aria-selected={themeStore.settings.defaultView === v.id}
          class="segment-btn pressable"
          class:active={themeStore.settings.defaultView === v.id}
          title={`Vista ${v.label}`}
          onclick={() => handleViewChange(v.id)}
        >
          <span class="view-label-text">{v.label}</span>
          <span class="view-icon" aria-hidden="true">
            {#if v.id === "continuous"}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M18.178 8c5.096 0 5.096 8 0 8-2.695 0-4.73-2.617-6.178-4.667C10.552 9.283 8.517 8 5.822 8c-5.096 0-5.096 8 0 8 2.695 0 4.73-2.617 6.178-4.667 1.448 2.05 3.483 3.333 6.178 3.333z" />
              </svg>
            {:else if v.id === "month"}
              <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round">
                <rect x="2" y="2.5" width="12" height="11" rx="2" />
                <line x1="2" y1="6" x2="14" y2="6" />
                <circle cx="5" cy="8.5" r="0.75" fill="currentColor" />
                <circle cx="8" cy="8.5" r="0.75" fill="currentColor" />
                <circle cx="11" cy="8.5" r="0.75" fill="currentColor" />
                <circle cx="5" cy="11" r="0.75" fill="currentColor" />
                <circle cx="8" cy="11" r="0.75" fill="currentColor" />
                <circle cx="11" cy="11" r="0.75" fill="currentColor" />
              </svg>
            {:else if v.id === "week"}
              <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round">
                <rect x="2" y="2.5" width="12" height="11" rx="2" />
                <line x1="6" y1="2.5" x2="6" y2="13.5" />
                <line x1="10" y1="2.5" x2="10" y2="13.5" />
                <line x1="2" y1="6" x2="14" y2="6" />
              </svg>
            {:else if v.id === "agenda"}
              <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.35" stroke-linecap="round" stroke-linejoin="round">
                <line x1="6" y1="4.5" x2="13.5" y2="4.5" />
                <line x1="6" y1="8" x2="13.5" y2="8" />
                <line x1="6" y1="11.5" x2="13.5" y2="11.5" />
                <circle cx="3.2" cy="4.5" r="0.8" fill="currentColor" />
                <circle cx="3.2" cy="8" r="0.8" fill="currentColor" />
                <circle cx="3.2" cy="11.5" r="0.8" fill="currentColor" />
              </svg>
            {/if}
          </span>
        </button>
      {/each}
    </div>

    <!-- CONTROLLI UTILITY SULLA BARRA (Mostrati solo quando c'è spazio sufficiente: overflowTier < 3) -->
    {#if overflowTier < 3}
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

      <!-- Toggle Visualizzazione Orario Lezioni nel Calendario -->
      <button
        type="button"
        class="toolbar-icon-only-btn pressable"
        class:active={themeStore.showLessonTimes}
        title={themeStore.showLessonTimes
          ? "Nascondi orario lezioni nel calendario (vista rapida)"
          : "Mostra orario lezioni nel calendario (vista rapida)"}
        aria-label="Toggle orario lezioni nel calendario"
        onclick={() => themeStore.toggleShowLessonTimes()}
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="10" />
          <polyline points="12 6 12 12 16 14" />
        </svg>
      </button>

      <!-- Pulsante Impostazioni Complete (Icona Ingranaggio / Gear) -->
      <button
        type="button"
        class="toolbar-icon-only-btn pressable"
        title="Impostazioni MySession"
        aria-label="Apri impostazioni"
        onclick={() => onOpenSettings?.()}
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
          <circle cx="12" cy="12" r="3" />
        </svg>
      </button>
    {/if}

    <!-- MENU A TENDINA OVERFLOW (Attivo quando la finestra si restringe: overflowTier >= 3) -->
    {#if overflowTier >= 3}
      <div class="overflow-dropdown-anchor">
        <button
          type="button"
          class="overflow-btn pressable"
          class:active={overflowMenuOpen}
          title="Altre opzioni e strumenti"
          aria-label="Altre opzioni e strumenti"
          aria-expanded={overflowMenuOpen}
          onclick={() => (overflowMenuOpen = !overflowMenuOpen)}
        >
          <svg class="chevron-icon" class:rotated={overflowMenuOpen} viewBox="0 0 16 16" fill="none">
            <path d="M4 6L8 10L12 6" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
          {#if overflowTier >= 4 && conflictCount > 0}
            <span class="overflow-badge-dot" title="{conflictCount} {conflictCount === 1 ? 'conflitto attivo' : 'conflitti attivi'}"></span>
          {/if}
        </button>

        {#if overflowMenuOpen}
          <div class="popover-menu overflow-popover" role="dialog" aria-label="Menu opzioni e strumenti">
            {#if overflowTier >= 4}
              <!-- Sezione Stato Sessione e Conflitti (se collassati dalla barra) -->
              <div class="popover-section-header">Stato Sessione</div>
              {#if conflictCount > 0}
                <button
                  type="button"
                  class="popover-item conflict-row"
                  onclick={() => {
                    overflowMenuOpen = false;
                    onOpenConflicts?.();
                  }}
                >
                  <span class="row-left">
                    <span class="conflict-pulse"></span>
                    <span class="conflict-text">{conflictCount} {conflictCount === 1 ? "Conflitto attivo" : "Conflitti attivi"}</span>
                  </span>
                  <span class="check-badge danger">{conflictCount}</span>
                </button>
              {:else if ignoredConflictCount > 0}
                <button
                  type="button"
                  class="popover-item"
                  onclick={() => {
                    overflowMenuOpen = false;
                    onOpenConflicts?.();
                  }}
                >
                  <span class="row-left">
                    <span class="ignored-icon">🙈</span>
                    <span>0 Conflitti ({ignoredConflictCount} ignorati)</span>
                  </span>
                </button>
              {/if}

              <button
                type="button"
                class="popover-item"
                disabled={!activeSession}
                onclick={() => onToggleSessionLock?.()}
              >
                <span class="row-left">
                  <svg class="popover-row-icon" viewBox="0 0 16 16" fill="none">
                    <path d="M4.75 6.5V4.25C4.75 2.45 6.2 1.25 8 1.25C9.8 1.25 11.25 2.45 11.25 4.25V6.5" stroke="currentColor" stroke-width="1.45" stroke-linecap="round" />
                    <rect x="3" y="6.25" width="10" height="8" rx="2.2" stroke="currentColor" stroke-width="1.35" />
                  </svg>
                  <span>Blocco Periodo Sessione</span>
                </span>
                <span class="check-badge" class:locked={isSessionLocked}>
                  {isSessionLocked ? "Attivo" : "Disattivo"}
                </span>
              </button>
              <div class="popover-separator"></div>
            {/if}

            <!-- Sezione Visualizzazione -->
            <div class="popover-section-header">Visualizzazione</div>
            <button
              type="button"
              class="popover-item"
              onclick={() => themeStore.toggleShowLessonTimes()}
            >
              <span class="row-left">
                <svg class="popover-row-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
                  <circle cx="12" cy="12" r="10" />
                  <polyline points="12 6 12 12 16 14" />
                </svg>
                <span>Orario Settimanale Lezioni</span>
              </span>
              <span class="check-badge" class:locked={themeStore.showLessonTimes}>
                {themeStore.showLessonTimes ? "Visibile" : "Nascosto"}
              </span>
            </button>

            <div class="popover-separator"></div>

            <!-- Sezione Aspetto & Zoom -->
            <div class="popover-section-header">
              <span>Zoom Interfaccia</span>
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

            <div class="popover-section-header" style="margin-top: 0.5rem;">Tema Sistema</div>
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

            <div class="popover-section-header">Materiale DWM</div>
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

            <div class="popover-separator"></div>

            <!-- Sezione Impostazioni Globali -->
            <button
              type="button"
              class="popover-item action-item"
              onclick={() => {
                overflowMenuOpen = false;
                onOpenSettings?.();
              }}
            >
              <span class="row-left">
                <svg class="popover-row-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
                  <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
                  <circle cx="12" cy="12" r="3" />
                </svg>
                <span>Tutte le Impostazioni…</span>
              </span>
            </button>
          </div>
        {/if}
      </div>
    {/if}

    <!-- Controlli Finestra Stile Windows 11 a Destra (Sempre visibile all'estrema destra) -->
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
    container-type: inline-size;
    container-name: toolbar;
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

  .toolbar-left {
    flex-shrink: 1;
  }

  .toolbar-right {
    justify-content: flex-end;
    flex-shrink: 0;
    margin-left: auto;
  }

  .toolbar-center {
    display: flex;
    justify-content: center;
    align-items: center;
    min-width: 12px;
    flex: 1 1 auto;
    height: 100%;
  }

  /* Micro-interazioni a molla istantanee Apple HIG (Emil Kowalski feedback) */
  .pressable {
    transition:
      background-color 120ms ease,
      border-color 120ms ease,
      color 120ms ease,
      transform 100ms ease;
  }

  .pressable:active {
    transform: scale(0.98);
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
  .session-dropdown-anchor {
    position: relative;
    display: inline-flex;
    min-width: 0;
    max-width: 32rem;
    flex-shrink: 1;
  }

  .appearance-dropdown-anchor {
    position: relative;
    display: inline-flex;
    flex-shrink: 0;
  }

  .session-selector-pill {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    height: 2.05rem;
    padding: 0 0.8rem;
    border-radius: var(--radius-sm);
    background: var(--surface-card);
    border: 0.5px solid var(--hairline-border);
    box-shadow: inset 0 0.5px 0 rgba(255, 255, 255, 0.06);
    color: var(--text-primary);
    font-size: 0.8125rem;
    font-weight: 600;
    letter-spacing: var(--tracking-normal);
    cursor: pointer;
    width: auto;
    max-width: 32rem;
    min-width: 0;
    flex-shrink: 1;
    overflow: hidden;
    transition:
      background-color 130ms ease,
      border-color 130ms ease,
      color 130ms ease;
  }

  .session-selector-pill:hover {
    background: var(--surface-elevated);
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
    display: inline-flex;
    align-items: baseline;
    gap: 0.4rem;
    overflow: hidden;
    white-space: nowrap;
    min-width: 0;
    flex-shrink: 1;
  }

  .session-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 3.5rem;
    flex-shrink: 1;
  }

  .session-year {
    font-size: 0.72rem;
    font-weight: 500;
    color: var(--text-tertiary);
    flex-shrink: 0;
    display: inline-block;
    overflow: hidden;
    white-space: nowrap;
    max-width: 8rem;
    opacity: 1;
    transition: max-width 180ms ease, opacity 140ms ease;
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
    background: var(--surface-card);
    border: 0.5px solid var(--hairline-border);
    box-shadow: inset 0 0.5px 0 rgba(255, 255, 255, 0.06);
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
    transition: background-color 110ms ease, color 110ms ease;
  }

  .nav-icon-btn:hover {
    background: var(--surface-elevated);
    color: var(--text-primary);
  }

  .nav-icon-btn:active {
    transform: scale(0.96);
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
    transition: background-color 110ms ease;
  }

  .nav-today-btn:hover {
    background: var(--surface-elevated);
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
    background: rgba(239, 68, 68, 0.14);
    border: 0.5px solid rgba(239, 68, 68, 0.38);
    box-shadow: none;
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

  .conflict-pill-indicator.ignored-pill {
    background: var(--surface-card);
    border: 0.5px solid var(--hairline-border);
    box-shadow: inset 0 0.5px 0 rgba(255, 255, 255, 0.06);
    color: var(--text-secondary);
  }

  .conflict-pill-indicator.ignored-pill:hover {
    background: var(--surface-elevated);
    color: var(--text-primary);
  }

  .conflict-pill-indicator.ignored-pill .ignored-icon {
    font-size: 0.76rem;
    line-height: 1;
  }

  .conflict-full-text {
    display: inline-block;
    max-width: 14rem;
    overflow: hidden;
    white-space: nowrap;
    opacity: 1;
    transition: max-width 180ms ease, opacity 140ms ease;
  }

  .conflict-compact-badge {
    display: inline-block;
    max-width: 0;
    overflow: hidden;
    white-space: nowrap;
    opacity: 0;
    font-size: 0.72rem;
    font-weight: 700;
    transition: max-width 180ms ease, opacity 140ms ease;
  }

  /* ==========================================================================
     Session Lock Toggle (Solido, stile Apple HIG)
     ========================================================================== */
  .session-lock-toggle {
    display: inline-flex;
    align-items: center;
    gap: 0.42rem;
    height: 1.95rem;
    padding: 0 0.65rem;
    border-radius: var(--radius-sm);
    background: var(--surface-card);
    border: 0.5px solid var(--hairline-border);
    box-shadow: inset 0 0.5px 0 rgba(255, 255, 255, 0.06);
    color: var(--text-secondary);
    font-size: 0.73rem;
    font-weight: 500;
    letter-spacing: var(--tracking-normal);
    cursor: pointer;
    user-select: none;
    flex-shrink: 0;
    transition:
      background-color 120ms ease,
      border-color 120ms ease,
      color 120ms ease;
  }

  .session-lock-toggle:hover:not(:disabled) {
    background: var(--surface-elevated);
    color: var(--text-primary);
    border-color: var(--hairline-border-strong);
  }

  .session-lock-toggle:active:not(:disabled) {
    transform: scale(0.98);
  }

  .session-lock-toggle:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .lock-status-badge {
    display: inline-block;
    max-width: 9rem;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: clip;
    opacity: 1;
    transition: max-width 180ms ease, opacity 140ms ease;
  }

  .lock-icon-container {
    position: relative;
    width: 15px;
    height: 15px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .lock-svg {
    width: 15px;
    height: 15px;
    overflow: visible;
  }

  .lock-shackle {
    transform-origin: 4.75px 6.5px;
    transition: transform 0.25s cubic-bezier(0.34, 1.56, 0.64, 1),
                stroke 0.2s ease;
  }

  .lock-body {
    transition: fill 0.2s ease, fill-opacity 0.2s ease, stroke 0.2s ease;
  }

  /* STATO ATTIVO: Lucchetto chiuso (Blocco periodo sessione) */
  .session-lock-toggle.is-locked {
    background: rgba(0, 122, 255, 0.16);
    border-color: rgba(0, 122, 255, 0.45);
    box-shadow: inset 0 0.5px 0 rgba(255, 255, 255, 0.08);
    color: var(--accent, #007AFF);
  }

  .session-lock-toggle.is-locked:hover:not(:disabled) {
    background: rgba(0, 122, 255, 0.22);
    border-color: rgba(0, 122, 255, 0.6);
  }

  .session-lock-toggle.is-locked .lock-shackle {
    transform: translateY(0) rotate(0deg);
  }

  .session-lock-toggle.is-locked .lock-body {
    fill: currentColor;
    fill-opacity: 0.12;
  }

  /* STATO DISATTIVO: Lucchetto aperto (Scorrimento infinito libero) */
  .session-lock-toggle:not(.is-locked) .lock-shackle {
    transform: translateY(-2.4px) rotate(-35deg);
  }

  .view-segmented-control {
    display: inline-flex;
    align-items: center;
    background: var(--surface-card);
    border: 0.5px solid var(--hairline-border);
    box-shadow: inset 0 0.5px 0 rgba(255, 255, 255, 0.06);
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
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 0;
    transition:
      background-color 110ms ease,
      color 110ms ease,
      padding 140ms ease;
  }

  .segment-btn:hover {
    color: var(--text-primary);
  }

  .segment-btn.active {
    background: var(--surface-elevated);
    color: var(--text-primary);
    font-weight: 600;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.25);
  }

  .view-label-text {
    display: inline-block;
    max-width: 6rem;
    overflow: hidden;
    white-space: nowrap;
    opacity: 1;
    transition: max-width 180ms ease, opacity 140ms ease;
  }

  .view-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 1;
    max-width: 0;
    overflow: hidden;
    opacity: 0;
    transform: scale(0.85);
    transition: max-width 180ms ease, opacity 140ms ease, transform 180ms ease;
  }

  .view-icon svg {
    width: 0.95rem;
    height: 0.95rem;
  }

  /* ==========================================================================
     Responsive Compaction & Fluid Transitions (Tier 0 -> Tier 4)
     ========================================================================== */

  /* Tier 1+: Compattazione testi */
  .unified-toolbar.tier-1 .session-year,
  .unified-toolbar.tier-2 .session-year,
  .unified-toolbar.tier-3 .session-year,
  .unified-toolbar.tier-4 .session-year {
    max-width: 0;
    opacity: 0;
    margin: 0;
    pointer-events: none;
  }

  .unified-toolbar.tier-1 .conflict-full-text,
  .unified-toolbar.tier-2 .conflict-full-text,
  .unified-toolbar.tier-3 .conflict-full-text,
  .unified-toolbar.tier-4 .conflict-full-text {
    max-width: 0;
    opacity: 0;
    margin: 0;
    pointer-events: none;
  }

  .unified-toolbar.tier-1 .conflict-compact-badge,
  .unified-toolbar.tier-2 .conflict-compact-badge,
  .unified-toolbar.tier-3 .conflict-compact-badge,
  .unified-toolbar.tier-4 .conflict-compact-badge {
    max-width: 2.5rem;
    opacity: 1;
  }

  .unified-toolbar.tier-1 .conflict-pill-indicator,
  .unified-toolbar.tier-2 .conflict-pill-indicator,
  .unified-toolbar.tier-3 .conflict-pill-indicator,
  .unified-toolbar.tier-4 .conflict-pill-indicator {
    padding: 0 0.45rem;
  }

  .unified-toolbar.tier-1 .lock-status-badge,
  .unified-toolbar.tier-2 .lock-status-badge,
  .unified-toolbar.tier-3 .lock-status-badge,
  .unified-toolbar.tier-4 .lock-status-badge {
    max-width: 0;
    opacity: 0;
    margin: 0;
    pointer-events: none;
  }

  .unified-toolbar.tier-1 .session-lock-toggle,
  .unified-toolbar.tier-2 .session-lock-toggle,
  .unified-toolbar.tier-3 .session-lock-toggle,
  .unified-toolbar.tier-4 .session-lock-toggle {
    padding: 0 0.45rem;
  }

  .unified-toolbar.tier-1 .scale-readout,
  .unified-toolbar.tier-2 .scale-readout,
  .unified-toolbar.tier-3 .scale-readout,
  .unified-toolbar.tier-4 .scale-readout {
    max-width: 0;
    opacity: 0;
    margin: 0;
    pointer-events: none;
  }

  /* Tier 2+: Viste a sole icone */
  .unified-toolbar.tier-2 .view-label-text,
  .unified-toolbar.tier-3 .view-label-text,
  .unified-toolbar.tier-4 .view-label-text {
    max-width: 0;
    opacity: 0;
    margin: 0;
    pointer-events: none;
  }

  .unified-toolbar.tier-2 .view-icon,
  .unified-toolbar.tier-3 .view-icon,
  .unified-toolbar.tier-4 .view-icon {
    max-width: 1.5rem;
    opacity: 1;
    transform: scale(1);
  }

  .unified-toolbar.tier-2 .segment-btn,
  .unified-toolbar.tier-3 .segment-btn,
  .unified-toolbar.tier-4 .segment-btn {
    padding: 0 0.45rem;
    min-width: 1.85rem;
  }

  .unified-toolbar.tier-2 .toolbar-left,
  .unified-toolbar.tier-2 .toolbar-right,
  .unified-toolbar.tier-3 .toolbar-left,
  .unified-toolbar.tier-3 .toolbar-right,
  .unified-toolbar.tier-4 .toolbar-left,
  .unified-toolbar.tier-4 .toolbar-right {
    gap: 0.35rem;
  }

  .unified-toolbar.tier-3 .session-selector-pill,
  .unified-toolbar.tier-4 .session-selector-pill {
    max-width: 16rem;
  }

  /* Fallback Container Queries (pre-idratazione) */
  @container toolbar (max-width: 1200px) {
    .session-year {
      max-width: 0;
      opacity: 0;
    }
  }

  @container toolbar (max-width: 1040px) {
    .conflict-full-text {
      max-width: 0;
      opacity: 0;
    }
    .conflict-compact-badge {
      max-width: 2.5rem;
      opacity: 1;
    }
    .conflict-pill-indicator {
      padding: 0 0.45rem;
    }
    .view-label-text {
      max-width: 0;
      opacity: 0;
    }
    .view-icon {
      max-width: 1.5rem;
      opacity: 1;
      transform: scale(1);
    }
    .segment-btn {
      padding: 0 0.45rem;
      min-width: 1.85rem;
    }
    .lock-status-badge {
      max-width: 0;
      opacity: 0;
    }
    .session-lock-toggle {
      padding: 0 0.45rem;
    }
  }

  @container toolbar (max-width: 900px) {
    .toolbar-left,
    .toolbar-right {
      gap: 0.35rem;
    }
    .toolbar-center {
      min-width: 16px;
    }
    .scale-readout {
      max-width: 0;
      opacity: 0;
    }
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
    background: var(--surface-card);
    border: 0.5px solid var(--hairline-border);
    box-shadow: inset 0 0.5px 0 rgba(255, 255, 255, 0.06);
    color: var(--text-secondary);
    font-size: 0.73rem;
    font-weight: 600;
    letter-spacing: var(--tracking-normal);
    cursor: pointer;
    flex-shrink: 0;
    transition:
      background-color 120ms ease,
      border-color 120ms ease,
      color 120ms ease;
  }

  .toolbar-icon-only-btn {
    width: 2.05rem;
    padding: 0;
  }

  .toolbar-utility-btn:hover,
  .toolbar-utility-btn.active,
  .toolbar-icon-only-btn:hover {
    background: var(--surface-elevated);
    color: var(--text-primary);
    border-color: var(--hairline-border-strong);
  }

  .toolbar-icon-only-btn.active {
    background: var(--surface-elevated);
    color: var(--accent-primary, #007AFF);
    border-color: var(--accent-primary, #007AFF);
  }

  .toolbar-utility-btn svg,
  .toolbar-icon-only-btn svg {
    width: 0.92rem;
    height: 0.92rem;
  }

  .scale-readout {
    display: inline-block;
    max-width: 4rem;
    overflow: hidden;
    white-space: nowrap;
    opacity: 1;
    font-variant-numeric: tabular-nums;
    transition: max-width 180ms ease, opacity 140ms ease;
  }

  /* ==========================================================================
     Overflow Dropdown Button & Popover (Apple HIG)
     ========================================================================== */
  .overflow-dropdown-anchor {
    position: relative;
    display: inline-flex;
    align-items: center;
    flex-shrink: 0;
  }

  .overflow-btn {
    position: relative;
    width: 2.05rem;
    height: 2.05rem;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-sm);
    background: var(--surface-card);
    border: 0.5px solid var(--hairline-border);
    box-shadow: inset 0 0.5px 0 rgba(255, 255, 255, 0.06);
    color: var(--text-secondary);
    cursor: pointer;
    flex-shrink: 0;
    transition:
      background-color 120ms ease,
      border-color 120ms ease,
      color 120ms ease;
  }

  .overflow-btn:hover,
  .overflow-btn.active {
    background: var(--surface-elevated);
    color: var(--text-primary);
    border-color: var(--hairline-border-strong);
  }

  .overflow-badge-dot {
    position: absolute;
    top: 3px;
    right: 3px;
    width: 7px;
    height: 7px;
    border-radius: var(--radius-pill);
    background: #ef4444;
    box-shadow: 0 0 6px rgba(239, 68, 68, 0.75);
    animation: conflictPing 2s infinite cubic-bezier(0, 0, 0.2, 1);
  }

  .overflow-popover {
    right: 0;
    width: 20rem;
    padding: 0.75rem;
    transform-origin: top right;
    max-height: calc(100vh - 4.5rem);
    overflow-y: auto;
  }

  .popover-item .row-left {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }

  .popover-row-icon {
    width: 1rem;
    height: 1rem;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .popover-item.conflict-row {
    background: rgba(239, 68, 68, 0.08);
    border-radius: var(--radius-xs);
  }

  .popover-item.conflict-row:hover {
    background: rgba(239, 68, 68, 0.16);
  }

  .conflict-text {
    color: #f87171;
    font-weight: 600;
  }

  .check-badge.danger {
    background: #ef4444;
    color: #ffffff;
  }

  .check-badge.locked {
    background: var(--accent-primary, #007AFF);
    color: #ffffff;
  }

  .check-badge:not(.locked):not(.danger) {
    background: var(--surface-pill);
    color: var(--text-tertiary);
    border: 0.5px solid var(--hairline-border);
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
