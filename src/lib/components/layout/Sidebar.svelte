<script lang="ts">
  import { APP_AUTHOR, APP_VERSION } from "../../constants/version";
  import { Eye, EyeOff, Sparkles } from "lucide-svelte";
  import { ipc } from "../../api/ipc";
  import { planStore } from "../../stores/plan.svelte";
  import { sessionStore } from "../../stores/session.svelte";
  import { themeStore } from "../../stores/theme.svelte";
  import type { Course, SessionSnapshot } from "../../types/domain";
  import { formatExamRoundLabel, formatItalianDateShort } from "../../utils/calendar";
  import { generateCourseColorTokens } from "../../utils/color";

  interface Props {
    snapshot: SessionSnapshot | null;
    selectedCourseId?: string | null;
    onSelectCourseFilter?: (courseId: string | null) => void;
    onJumpToDate?: (isoDate: string) => void;
    onOpenNewCourse?: () => void;
    onEditCourse?: (course: Course) => void;
    onOpenNewExam?: () => void;
    onOpenNewEvent?: () => void;
    onOpenTimetable?: () => void;
  }

  let {
    snapshot = null,
    selectedCourseId = null,
    onSelectCourseFilter,
    onJumpToDate,
    onOpenNewCourse,
    onEditCourse,
    onOpenNewExam,
    onOpenNewEvent,
    onOpenTimetable
  }: Props = $props();

  let isCompact = $derived(themeStore.settings.sidebarState === "compact");
  let isDark = $derived(themeStore.effectiveTheme === "dark");

  let telemetry = $derived(snapshot?.telemetry ?? null);
  let courses = $derived(snapshot?.courses ?? []);
  let workloadsByCourse = $derived.by(() => {
    const map = new Map<string, { progressPct: number; dailyHours: number; criticality: number }>();
    for (const w of telemetry?.courseWorkloads ?? []) {
      map.set(w.courseId, {
        progressPct: Math.round(w.completionPct),
        dailyHours: w.recommendedDailyHours,
        criticality: w.criticalityScore
      });
    }
    return map;
  });

  function toggleSidebarMode() {
    themeStore.settings.sidebarState = isCompact ? "expanded" : "compact";
    themeStore.persistAndSync();
  }

  function handleCourseClick(courseId: string) {
    if (selectedCourseId === courseId) {
      onSelectCourseFilter?.(null);
    } else {
      onSelectCourseFilter?.(courseId);
    }
  }

  async function toggleCourseVisibility(course: Course, e: MouseEvent) {
    e.stopPropagation();
    const sessionId = snapshot?.session.id;
    if (!sessionId) return;
    try {
      const snap = await ipc.setCourseVisibility(sessionId, course.id, !course.isVisible);
      sessionStore.activeSnapshot = snap;
    } catch (err) {
      console.error("Errore toggle visibilità corso:", err);
    }
  }
  let shortcutsCollapsed = $state<boolean>(
    typeof localStorage !== "undefined" && localStorage.getItem("mysession_shortcuts_collapsed") === "true"
  );

  function toggleShortcuts() {
    shortcutsCollapsed = !shortcutsCollapsed;
    try {
      localStorage.setItem("mysession_shortcuts_collapsed", String(shortcutsCollapsed));
    } catch (e) {
      console.warn("Impossibile salvare stato scorciatoie:", e);
    }
  }
</script>

<aside class="app-sidebar" class:compact={isCompact} aria-label="Pannello Telemetria e Corsi">
  <!-- Header Sidebar + Collapse Toggle -->
  <div class="sidebar-top-row">
    {#if !isCompact}
      <div class="sidebar-session-meta">
        <span class="meta-eyebrow">SESSIONE ATTIVA</span>
        <h2 class="meta-title" title={snapshot?.session.name}>{snapshot?.session.name ?? "Nessuna Sessione"}</h2>
      </div>
    {/if}
    <button
      type="button"
      class="collapse-btn pressable"
      title={isCompact ? "Espandi barra laterale" : "Compatta barra laterale"}
      aria-label="Commuta larghezza barra laterale"
      onclick={toggleSidebarMode}
    >
      <svg viewBox="0 0 16 16" fill="none">
        <rect x="2" y="2.5" width="12" height="11" rx="2" stroke="currentColor" stroke-width="1.3" />
        <path d="M6 2.5V13.5" stroke="currentColor" stroke-width="1.3" />
      </svg>
    </button>
  </div>

  {#if !isCompact}
    <!-- 1. HERO CARD COMPATTA: PROSSIMO TARGET D'ESAME & COUNTDOWN -->
    <div class="target-hero-card">
      {#if telemetry?.nextTargetExam && telemetry?.nextTargetCourse}
        {@const targetPalette = generateCourseColorTokens(telemetry.nextTargetCourse.colorHex, isDark)}
        <button
          type="button"
          class="target-hero-trigger pressable"
          onclick={() => {
            if (telemetry?.nextTargetExam) {
              onJumpToDate?.(telemetry.nextTargetExam.date);
            }
          }}
          title="Salta alla data del prossimo appello nel calendario"
        >
          <div class="target-top">
            <div class="target-left-group">
              <span
                class="target-sigla-badge"
                style:background={targetPalette.solid}
                style:color={targetPalette.badgeText}
              >
                {telemetry.nextTargetCourse.sigla}
              </span>
              <strong class="target-course-name">{telemetry.nextTargetCourse.name}</strong>
            </div>
            <span class="target-date-pill">
              {formatItalianDateShort(telemetry.nextTargetExam.date)}
            </span>
          </div>

          <div class="target-countdown-row">
            <div class="countdown-number-wrap">
              <span class="countdown-val">
                {telemetry.daysToNextTarget !== null ? Math.max(0, telemetry.daysToNextTarget) : "—"}
              </span>
              <span class="countdown-unit">
                {telemetry.daysToNextTarget === 1 ? "giorno" : "giorni"} all'appello
              </span>
            </div>
            <span class="target-round-label">
              {formatExamRoundLabel(telemetry.nextTargetExam.round)} · {telemetry.nextTargetCourse.cfu} CFU
            </span>
          </div>
        </button>
      {:else}
        <div class="target-empty-state">
          <span class="empty-title">Nessun appello imminente</span>
          <span class="empty-sub">Pianifica un esame per attivare il calcolo carico.</span>
        </div>
      {/if}
    </div>

    <!-- 2. GRIGLIA TELEMETRIA DETERMINISTICA COMPATTA -->
    <div class="telemetry-kpi-grid">
      <div class="kpi-box">
        <span class="kpi-label">CFU</span>
        <strong class="kpi-value">{telemetry?.totalCfu ?? 0}</strong>
        <span class="kpi-sub">{telemetry?.coursesCount ?? 0} materie</span>
      </div>
      <div class="kpi-box">
        <span class="kpi-label">Appelli</span>
        <strong class="kpi-value">{telemetry?.examsCount ?? 0}</strong>
        <span class="kpi-sub">
          {snapshot?.conflicts.length ?? 0} {snapshot?.conflicts.length === 1 ? "conflitto" : "conflitti"}
        </span>
      </div>
      <div class="kpi-box">
        <span class="kpi-label">Oggi</span>
        <strong class="kpi-value">{(telemetry?.recommendedHoursToday ?? 0).toFixed(1)}h</strong>
        <span class="kpi-sub">studio cons.</span>
      </div>
      <div class="kpi-box">
        <span class="kpi-label">Progresso</span>
        <strong class="kpi-value">{Math.round(telemetry?.overallProgressPct ?? 0)}%</strong>
        <span class="kpi-sub">
          {telemetry?.completedUnitsTotal ?? 0}/{telemetry?.unitsTotal ?? 0} unità
        </span>
      </div>
    </div>

    <!-- 3. AZIONI RAPIDE OPERATIVE -->
    <div class="quick-actions-row">
      <button type="button" class="action-chip primary pressable" onclick={() => onOpenNewCourse?.()}>
        + Corso
      </button>
      <button type="button" class="action-chip pressable" onclick={() => onOpenNewExam?.()}>
        + Appello
      </button>
      <button type="button" class="action-chip pressable" onclick={() => onOpenNewEvent?.()}>
        + Impegno
      </button>
      <button type="button" class="action-chip pressable" onclick={() => onOpenTimetable?.()}>
        📅 Orario
      </button>
    </div>

    <!-- BANNER PIANI IN ATTESA -->
    {#if planStore.pendingPlans.length > 0}
      <button
        type="button"
        class="plan-alert-pill pressable"
        onclick={() => planStore.openPreview(planStore.pendingPlans[0])}
      >
        <div style="display: flex; align-items: center; gap: 0.4rem;">
          <Sparkles size={13} />
          <span style="font-weight: 600;">
            {planStore.pendingPlans.length} {planStore.pendingPlans.length === 1 ? "Piano rilevato" : "Piani rilevati"}
          </span>
        </div>
        <span style="font-size: 0.72rem; opacity: 0.85;">Esamina →</span>
      </button>
    {/if}

    <!-- 4. ELENCO CORSI & WORKLOAD ENGINE -->
    <div class="courses-section-header">
      <span>MATERIE IN SESSIONE ({courses.length})</span>
      {#if selectedCourseId}
        <button
          type="button"
          class="clear-filter-btn"
          onclick={() => onSelectCourseFilter?.(null)}
        >
          Mostra tutte
        </button>
      {/if}
    </div>

    <div class="courses-scroll-list">
      {#if courses.length === 0}
        <div class="courses-empty-card">
          <p>Nessuna materia registrata in questa sessione.</p>
          <button type="button" class="empty-cta-btn pressable" onclick={() => onOpenNewCourse?.()}>
            Aggiungi la prima materia
          </button>
        </div>
      {:else}
        {#each courses as course (course.id)}
          {@const palette = generateCourseColorTokens(course.colorHex, isDark)}
          {@const metric = workloadsByCourse.get(course.id)}
          {@const isSelected = selectedCourseId === course.id}
          <div
            class="course-row-card"
            class:selected={isSelected}
            class:dimmed={!course.isVisible}
            style:--course-solid={palette.solid}
            style:--course-tint={palette.bgTint}
            style:--course-border={palette.borderTint}
            style:--course-text={palette.textContrast}
          >
            <button
              type="button"
              class="course-main-click"
              onclick={() => handleCourseClick(course.id)}
              title="Filtra calendario per {course.name}"
            >
              <div class="course-row-top">
                <span
                  class="course-sigla-pill"
                  style:background={palette.solid}
                  style:color={palette.badgeText}
                >
                  {course.sigla}
                </span>
                <span class="course-title">{course.name}</span>
                <span class="course-cfu-badge">{course.cfu} CFU</span>
              </div>

              <div class="course-progress-track">
                <div
                  class="course-progress-fill"
                  style:width="{metric?.progressPct ?? 0}%"
                  style:background={palette.solid}
                ></div>
              </div>

              <div class="course-row-meta">
                <span>Avanzamento: {metric?.progressPct ?? 0}%</span>
                {#if metric && metric.dailyHours > 0}
                  <span>{metric.dailyHours.toFixed(1)} h/g</span>
                {/if}
              </div>
            </button>

            <button
              type="button"
              class="course-edit-icon-btn pressable"
              title={course.isVisible ? "Nascondi materia dal calendario" : "Mostra materia nel calendario"}
              aria-label="Toggle visibilità {course.name}"
              onclick={(e) => toggleCourseVisibility(course, e)}
            >
              {#if course.isVisible}
                <Eye size={13} />
              {:else}
                <EyeOff size={13} style="opacity: 0.4;" />
              {/if}
            </button>

            <button
              type="button"
              class="course-edit-icon-btn pressable"
              title="Modifica {course.name}"
              aria-label="Modifica corso {course.name}"
              onclick={() => onEditCourse?.(course)}
            >
              <svg viewBox="0 0 16 16" fill="none">
                <path
                  d="M11.2 2.5L13.5 4.8L5.3 13H3V10.7L11.2 2.5Z"
                  stroke="currentColor"
                  stroke-width="1.3"
                  stroke-linejoin="round"
                />
              </svg>
            </button>
          </div>
        {/each}
      {/if}
    </div>
  {:else}
    <!-- VISTA COMPATTA: Icone & Sigle Corsi -->
    <div class="compact-courses-rail">
      {#each courses as course (course.id)}
        {@const palette = generateCourseColorTokens(course.colorHex, isDark)}
        <button
          type="button"
          class="compact-course-avatar pressable"
          class:selected={selectedCourseId === course.id}
          style:background={palette.solid}
          style:color={palette.badgeText}
          title="{course.sigla} — {course.name} ({course.cfu} CFU)"
          onclick={() => handleCourseClick(course.id)}
        >
          {course.sigla.slice(0, 3)}
        </button>
      {/each}
      <button
        type="button"
        class="compact-add-btn pressable"
        title="Nuovo Corso"
        onclick={() => onOpenNewCourse?.()}
      >
        +
      </button>
    </div>
  {/if}

  <!-- Sezione Scorciatoie da Tastiera con Minimizzazione a Fisarmonica -->
  {#if !isCompact}
    <div class="sidebar-shortcuts-card" class:collapsed={shortcutsCollapsed}>
      <button
        type="button"
        class="shortcuts-header-btn pressable"
        onclick={toggleShortcuts}
        title={shortcutsCollapsed ? "Espandi scorciatoie da tastiera" : "Minimizza scorciatoie da tastiera"}
        aria-expanded={!shortcutsCollapsed}
      >
        <div class="shortcuts-title-wrap">
          <svg viewBox="0 0 16 16" fill="none" class="shortcuts-mini-icon">
            <rect x="2" y="3.5" width="12" height="9" rx="2" stroke="currentColor" stroke-width="1.2" />
            <path d="M4.5 6.5H5.5M7.5 6.5H8.5M10.5 6.5H11.5M4.5 9H11.5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" />
          </svg>
          <span class="shortcuts-title">SCORCIATOIE DA TASTIERA</span>
        </div>
        <svg class="shortcuts-chevron" class:rotated={!shortcutsCollapsed} viewBox="0 0 16 16" fill="none">
          <path d="M4 6L8 10L12 6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </button>

      {#if !shortcutsCollapsed}
        <div class="shortcuts-grid">
          <div class="shortcut-item">
            <kbd class="shortcut-kbd">T</kbd>
            <span class="shortcut-label">Salta ad Oggi</span>
          </div>
          <div class="shortcut-item">
            <div class="kbd-combo">
              <kbd class="shortcut-kbd">←</kbd>
              <kbd class="shortcut-kbd">→</kbd>
            </div>
            <span class="shortcut-label">Naviga Periodo</span>
          </div>
          <div class="shortcut-item">
            <div class="kbd-combo">
              <kbd class="shortcut-kbd">Spazio</kbd>
              <span class="kbd-sep">/</span>
              <kbd class="shortcut-kbd">Ctrl+K</kbd>
            </div>
            <span class="shortcut-label">Command Bar</span>
          </div>
          <div class="shortcut-item">
            <kbd class="shortcut-kbd">Esc</kbd>
            <span class="shortcut-label">Chiudi Pannelli</span>
          </div>
        </div>
      {/if}
    </div>
  {:else}
    <div
      class="compact-shortcuts-btn"
      title="Scorciatoie Tastiera:&#10;• T: Salta ad Oggi&#10;• ← / →: Naviga Periodo&#10;• Spazio / Ctrl+K: Command Bar&#10;• Esc: Chiudi"
    >
      <svg viewBox="0 0 16 16" fill="none" class="shortcut-icon">
        <rect x="2" y="3.5" width="12" height="9" rx="2" stroke="currentColor" stroke-width="1.25" />
        <path d="M4.5 6.5H5.5M7.5 6.5H8.5M10.5 6.5H11.5M4.5 9H11.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
      </svg>
    </div>
  {/if}

  <!-- Footer Autore e Versione in basso alla barra laterale sinistra -->
  <div class="sidebar-footer-meta">
    {#if !isCompact}
      <div class="footer-app-row">
        <span class="footer-brand">MySession</span>
        <span class="footer-version-tag">v{APP_VERSION}</span>
      </div>
      <div class="footer-author-row">
        <span class="footer-author-label">Autore:</span>
        <span class="footer-author-name">{APP_AUTHOR}</span>
      </div>
    {:else}
      <span class="footer-compact-badge" title="MySession v{APP_VERSION} · {APP_AUTHOR}">v{APP_VERSION}</span>
    {/if}
  </div>
</aside>

<style>
  .app-sidebar {
    width: 17.5rem;
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding: 0.85rem;
    background: var(--sidebar-bg);
    backdrop-filter: blur(28px) saturate(180%);
    -webkit-backdrop-filter: blur(28px) saturate(180%);
    border-right: 0.5px solid var(--hairline-border);
    flex-shrink: 0;
    user-select: none;
    overflow: hidden;
    transition: width 220ms var(--spring-snappy), padding 220ms var(--spring-snappy);
  }

  .app-sidebar.compact {
    width: 3.6rem;
    padding: 0.75rem 0.45rem;
    align-items: center;
  }

  .sidebar-top-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .sidebar-session-meta {
    min-width: 0;
  }

  .meta-eyebrow {
    display: block;
    font-size: 0.62rem;
    font-weight: 700;
    letter-spacing: var(--tracking-caps);
    color: var(--text-tertiary);
  }

  .meta-title {
    margin: 0.1rem 0 0;
    font-size: 0.95rem;
    font-weight: 700;
    letter-spacing: var(--tracking-tight);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .collapse-btn {
    width: 1.85rem;
    height: 1.85rem;
    border-radius: var(--radius-xs);
    border: 0.5px solid var(--hairline-border);
    background: var(--surface-pill);
    color: var(--text-secondary);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    flex-shrink: 0;
  }

  .collapse-btn:hover {
    background: var(--surface-pill-hover);
    color: var(--text-primary);
  }

  .collapse-btn svg {
    width: 0.92rem;
    height: 0.92rem;
  }

  /* Hero Target Card Compatta Apple HIG */
  .target-hero-card {
    border-radius: var(--radius-md);
    background: var(--surface-elevated);
    border: 0.5px solid var(--hairline-border-strong);
    box-shadow: var(--shadow-pill);
    overflow: hidden;
  }

  .target-hero-trigger {
    width: 100%;
    padding: 0.55rem 0.7rem;
    border: none;
    background: transparent;
    color: inherit;
    text-align: left;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .target-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.4rem;
  }

  .target-left-group {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    min-width: 0;
    overflow: hidden;
  }

  .target-sigla-badge {
    font-size: 0.62rem;
    font-weight: 700;
    padding: 0.1rem 0.36rem;
    border-radius: var(--radius-xs);
    letter-spacing: var(--tracking-wide);
    flex-shrink: 0;
  }

  .target-date-pill {
    font-size: 0.65rem;
    font-weight: 600;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .target-countdown-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.4rem;
  }

  .countdown-number-wrap {
    display: inline-flex;
    align-items: baseline;
    gap: 0.28rem;
    padding: 0.18rem 0.42rem;
    border-radius: var(--radius-xs);
    background: var(--accent-soft);
    color: var(--accent-primary);
    border: 0.5px solid var(--hairline-border);
    flex-shrink: 0;
  }

  .countdown-val {
    font-size: 0.95rem;
    font-weight: 800;
    line-height: 1;
    font-variant-numeric: tabular-nums;
  }

  .countdown-unit {
    font-size: 0.6rem;
    font-weight: 600;
    text-transform: none;
    letter-spacing: var(--tracking-normal);
  }

  .target-course-name {
    font-size: 0.78rem;
    font-weight: 700;
    letter-spacing: var(--tracking-normal);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .target-round-label {
    font-size: 0.65rem;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .target-empty-state {
    padding: 0.55rem 0.7rem;
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .empty-title {
    font-size: 0.74rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .empty-sub {
    font-size: 0.65rem;
    color: var(--text-secondary);
    line-height: 1.35;
  }

  /* Telemetry KPI Grid Compatta */
  .telemetry-kpi-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.35rem;
  }

  .kpi-box {
    padding: 0.38rem 0.52rem;
    border-radius: var(--radius-xs);
    background: var(--surface-card);
    border: 0.5px solid var(--hairline-border);
    box-shadow: var(--shadow-pill);
    display: flex;
    flex-direction: column;
    gap: 0.05rem;
  }

  .kpi-label {
    font-size: 0.58rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: var(--tracking-caps);
    color: var(--text-tertiary);
  }

  .kpi-value {
    font-size: 0.88rem;
    font-weight: 700;
    letter-spacing: var(--tracking-tight);
    font-variant-numeric: tabular-nums;
    line-height: 1.15;
  }

  .kpi-sub {
    font-size: 0.6rem;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  /* Quick Actions */
  .quick-actions-row {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 0.28rem;
  }

  .action-chip {
    height: 1.75rem;
    padding: 0 0.2rem;
    border-radius: var(--radius-xs);
    border: 0.5px solid var(--hairline-border);
    background: var(--surface-pill);
    box-shadow: var(--shadow-pill);
    color: var(--text-primary);
    font-size: 0.68rem;
    font-weight: 600;
    letter-spacing: var(--tracking-normal);
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .action-chip:hover {
    background: var(--surface-pill-hover);
    border-color: var(--hairline-border-strong);
  }

  .action-chip.primary {
    background: var(--accent-primary);
    border-color: transparent;
    color: #ffffff;
    box-shadow: 0 1px 3px rgba(10, 132, 255, 0.3);
  }

  /* Courses List */
  .courses-section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 0.64rem;
    font-weight: 700;
    letter-spacing: var(--tracking-caps);
    color: var(--text-tertiary);
    padding: 0.15rem 0.1rem 0;
  }

  .clear-filter-btn {
    border: none;
    background: transparent;
    color: var(--accent-primary);
    font-size: 0.66rem;
    font-weight: 600;
    cursor: pointer;
    padding: 0;
  }

  .courses-scroll-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.42rem;
    padding-right: 0.15rem;
  }

  .courses-empty-card {
    padding: 1rem 0.75rem;
    border-radius: var(--radius-sm);
    background: var(--surface-card);
    border: 0.5px dashed var(--hairline-border-strong);
    text-align: center;
    font-size: 0.74rem;
    color: var(--text-secondary);
  }

  .courses-empty-card p {
    margin: 0 0 0.65rem;
  }

  .empty-cta-btn {
    padding: 0.38rem 0.75rem;
    border-radius: var(--radius-xs);
    border: none;
    background: var(--accent-primary);
    color: #fff;
    font-size: 0.72rem;
    font-weight: 600;
    cursor: pointer;
  }

  .plan-alert-pill {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 0.5rem 0.75rem;
    border-radius: var(--radius-sm);
    background: rgba(245, 158, 11, 0.12);
    border: 0.5px solid rgba(245, 158, 11, 0.3);
    color: #f59e0b;
    font-size: 0.8rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .plan-alert-pill:hover {
    background: rgba(245, 158, 11, 0.18);
    border-color: rgba(245, 158, 11, 0.45);
  }

  .course-row-card {
    position: relative;
    display: flex;
    align-items: stretch;
    border-radius: var(--radius-sm);
    background: var(--surface-card);
    border: 0.5px solid var(--hairline-border);
    box-shadow: var(--shadow-pill);
    will-change: transform;
    transition:
      transform 120ms var(--spring-press),
      border-color 130ms ease,
      background-color 130ms ease;
  }

  .course-row-card:hover {
    border-color: var(--course-border);
  }

  .course-row-card.selected {
    background: var(--course-tint);
    border-color: var(--course-solid);
  }

  .course-row-card.dimmed {
    opacity: 0.45;
    filter: grayscale(45%);
  }

  .course-main-click {
    flex: 1;
    padding: 0.44rem 0.55rem;
    border: none;
    background: transparent;
    color: inherit;
    text-align: left;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 0.26rem;
    min-width: 0;
  }

  .course-row-top {
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }

  .course-sigla-pill {
    font-size: 0.62rem;
    font-weight: 700;
    padding: 0.08rem 0.34rem;
    border-radius: 0.25rem;
    flex-shrink: 0;
    letter-spacing: var(--tracking-wide);
  }

  .course-title {
    flex: 1;
    font-size: 0.74rem;
    font-weight: 600;
    letter-spacing: var(--tracking-normal);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .course-cfu-badge {
    font-size: 0.62rem;
    font-weight: 600;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .course-progress-track {
    width: 100%;
    height: 2.5px;
    border-radius: var(--radius-pill);
    background: var(--surface-pill);
    overflow: hidden;
  }

  .course-progress-fill {
    height: 100%;
    border-radius: var(--radius-pill);
    transition: width 260ms var(--ease-out);
  }

  .course-row-meta {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 0.64rem;
    color: var(--text-secondary);
  }

  .course-edit-icon-btn {
    width: 1.8rem;
    border: none;
    background: transparent;
    color: var(--text-tertiary);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    border-left: 0.5px solid var(--hairline-border);
  }

  .course-edit-icon-btn:hover {
    color: var(--text-primary);
    background: var(--surface-pill-hover);
  }

  .course-edit-icon-btn svg {
    width: 0.8rem;
    height: 0.8rem;
  }

  /* Compact Rail */
  .compact-courses-rail {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.45rem;
    width: 100%;
    overflow-y: auto;
  }

  .compact-course-avatar {
    width: 2.2rem;
    height: 2.2rem;
    border-radius: var(--radius-sm);
    border: 1.5px solid transparent;
    font-size: 0.65rem;
    font-weight: 700;
    cursor: pointer;
  }

  .compact-course-avatar.selected {
    border-color: var(--text-primary);
  }

  .compact-add-btn {
    width: 2.2rem;
    height: 2.2rem;
    border-radius: var(--radius-sm);
    border: 0.5px dashed var(--hairline-border-strong);
    background: var(--surface-pill);
    color: var(--text-primary);
    font-size: 1rem;
    cursor: pointer;
  }

  /* Sezione Scorciatoie da Tastiera con Minimizzazione a Fisarmonica */
  .sidebar-shortcuts-card {
    margin-top: auto;
    border-radius: var(--radius-sm);
    background: var(--surface-pill);
    border: 0.5px solid var(--hairline-border);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    overflow: hidden;
    transition: background 140ms ease, border-color 140ms ease;
  }

  .sidebar-shortcuts-card.collapsed {
    background: transparent;
    border-color: transparent;
  }

  .shortcuts-header-btn {
    width: 100%;
    padding: 0.35rem 0.45rem;
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: transparent;
    border: none;
    cursor: pointer;
    border-radius: var(--radius-xs);
    color: var(--text-tertiary);
    transition: color 140ms ease, background 140ms ease;
  }

  .shortcuts-header-btn:hover {
    color: var(--text-primary);
    background: var(--surface-pill-hover);
  }

  .shortcuts-title-wrap {
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }

  .shortcuts-mini-icon {
    width: 0.85rem;
    height: 0.85rem;
    opacity: 0.7;
  }

  .shortcuts-title {
    font-size: 0.58rem;
    font-weight: 700;
    letter-spacing: var(--tracking-caps);
  }

  .shortcuts-chevron {
    width: 0.82rem;
    height: 0.82rem;
    transition: transform 180ms var(--spring-snappy);
    opacity: 0.7;
  }

  .shortcuts-chevron.rotated {
    transform: rotate(180deg);
  }

  .shortcuts-grid {
    padding: 0.1rem 0.5rem 0.45rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    animation: backdropFade 140ms ease;
  }

  .shortcut-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.4rem;
  }

  .shortcut-label {
    font-size: 0.68rem;
    color: var(--text-secondary);
    font-weight: 500;
  }

  .kbd-combo {
    display: inline-flex;
    align-items: center;
    gap: 0.2rem;
  }

  .kbd-sep {
    font-size: 0.62rem;
    color: var(--text-tertiary);
  }

  .shortcut-kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0.1rem 0.34rem;
    font-size: 0.62rem;
    font-family: var(--font-mono);
    font-weight: 700;
    color: var(--text-primary);
    background: var(--surface-pill, rgba(128, 128, 128, 0.12));
    border: 0.5px solid var(--hairline-border-strong);
    border-radius: var(--radius-xs);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.08);
  }

  .compact-shortcuts-btn {
    width: 2.2rem;
    height: 2.2rem;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    background: rgba(128, 128, 128, 0.06);
    border: 0.5px solid var(--hairline-border);
    margin-top: auto;
    cursor: help;
  }

  .shortcut-icon {
    width: 1.1rem;
    height: 1.1rem;
  }

  /* Footer Autore & Versione in basso */
  .sidebar-footer-meta {
    margin-top: 0;
    padding-top: 0.45rem;
    border-top: 0.5px solid var(--hairline-border);
    display: flex;
    flex-direction: column;
    gap: 0.18rem;
    flex-shrink: 0;
  }

  .footer-app-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .footer-brand {
    font-size: 0.72rem;
    font-weight: 700;
    letter-spacing: -0.01em;
    color: var(--text-primary);
  }

  .footer-version-tag {
    font-size: 0.64rem;
    font-weight: 600;
    padding: 0.08rem 0.35rem;
    border-radius: var(--radius-xs);
    background: var(--surface-pill);
    border: 0.5px solid var(--hairline-border);
    color: var(--text-secondary);
  }

  .footer-author-row {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.65rem;
    color: var(--text-tertiary);
  }

  .footer-author-label {
    font-weight: 500;
  }

  .footer-author-name {
    font-weight: 600;
    color: var(--text-secondary);
  }

  .footer-compact-badge {
    font-size: 0.6rem;
    font-weight: 700;
    color: var(--text-tertiary);
    text-align: center;
    padding: 0.15rem 0.25rem;
  }
</style>
