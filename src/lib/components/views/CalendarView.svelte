<script lang="ts">
  import { CalendarClock, ChevronLeft, ChevronRight, ExternalLink, Plus, Sparkles } from "lucide-svelte";
  import { navStore } from "../../stores/navigation.svelte";
  import { sessionStore } from "../../stores/session.svelte";
  import { themeStore } from "../../stores/theme.svelte";
  import {
    addMonths,
    buildMonthGrid,
    formatItalianDateLong,
    getTodayIso,
    getWeekdayHeaders,
    indexSnapshotByDate,
    MESI_ITALIANI,
    parseIsoDate
  } from "../../utils/calendar";
  import { getCategoryColor, getCategoryLabel } from "../../utils/categories";

  let snap = $derived(sessionStore.activeSnapshot);
  let todayIso = $derived(getTodayIso());

  let initial = parseIsoDate(navStore.calendarSelectedDate || getTodayIso());
  let viewYear = $state<number>(initial.year);
  let viewMonth = $state<number>(initial.month);
  let calendarMode = $state<"month" | "continuous">("month");
  let filterMode = $state<"all" | "tasks" | "exams" | "lessons">("all");
  let quickEventTitle = $state("");
  let noteDraft = $state("");

  let indexed = $derived(indexSnapshotByDate(snap, sessionStore.holidays));
  let weekdayHeaders = $derived(getWeekdayHeaders(themeStore.settings.startDayOfWeek));
  let cells = $derived(
    buildMonthGrid(viewYear, viewMonth, themeStore.settings.startDayOfWeek, todayIso)
  );
  let continuousMonths = $derived.by(() => {
    if (!snap) return [];
    const s = parseIsoDate(snap.session.startDate);
    const e = parseIsoDate(snap.session.endDate);
    const total = Math.max(1, (e.year - s.year) * 12 + (e.month - s.month) + 1);
    const out = [];
    for (let i = 0; i < Math.min(total, 18); i++) {
      const { year, month } = addMonths(s.year, s.month, i);
      out.push({
        key: `${year}-${month}`,
        title: `${MESI_ITALIANI[month - 1]} ${year}`,
        cells: buildMonthGrid(year, month, themeStore.settings.startDayOfWeek, todayIso)
      });
    }
    return out;
  });
  let selectedDayData = $derived(indexed.byDate.get(navStore.calendarSelectedDate));

  $effect(() => {
    noteDraft = selectedDayData?.note?.contentMarkdown ?? "";
  });

  function shiftMonth(delta: number) {
    const next = addMonths(viewYear, viewMonth, delta);
    viewYear = next.year;
    viewMonth = next.month;
  }

  function jumpToToday() {
    const p = parseIsoDate(todayIso);
    viewYear = p.year;
    viewMonth = p.month;
    navStore.calendarSelectedDate = todayIso;
  }

  async function addQuickEvent() {
    if (!snap || !quickEventTitle.trim()) return;
    await sessionStore.upsertCalendarEvent({
      sessionId: snap.session.id,
      date: navStore.calendarSelectedDate,
      title: quickEventTitle.trim(),
      category: "personale"
    });
    quickEventTitle = "";
  }

  async function saveNote() {
    await sessionStore.saveDailyNote(navStore.calendarSelectedDate, noteDraft);
    navStore.showToast("Nota giornaliera salvata", "success");
  }
</script>

{#if snap}
  <div class="cal-layout bento-enter">
    <!-- Griglia Mensile Bento -->
    <div class="bento-card cal-main">
      <div class="cal-header">
        <div class="cal-title-row">
          <h2>{MESI_ITALIANI[viewMonth - 1]} {viewYear}</h2>
          <div class="nav-arrows">
            <button type="button" class="icon-pill" onclick={() => shiftMonth(-1)} aria-label="Mese precedente">
              <ChevronLeft size={16} />
            </button>
            <button type="button" class="pill-btn pill-btn-soft btn-xs" onclick={jumpToToday}>Oggi</button>
            <button type="button" class="icon-pill" onclick={() => shiftMonth(1)} aria-label="Mese successivo">
              <ChevronRight size={16} />
            </button>
          </div>
        </div>

        <div class="cal-tools">
          <div class="filter-pills">
            <button type="button" class="pill-filter" class:active={calendarMode === "month"} onclick={() => (calendarMode = "month")}>Mese</button>
            <button type="button" class="pill-filter" class:active={calendarMode === "continuous"} onclick={() => (calendarMode = "continuous")}>Continuo</button>
          </div>
          <div class="filter-pills">
            <button type="button" class="pill-filter" class:active={filterMode === "all"} onclick={() => (filterMode = "all")}>Tutti</button>
            <button type="button" class="pill-filter" class:active={filterMode === "tasks"} onclick={() => (filterMode = "tasks")}>Studio</button>
            <button type="button" class="pill-filter" class:active={filterMode === "exams"} onclick={() => (filterMode = "exams")}>Esami</button>
            <button type="button" class="pill-filter" class:active={filterMode === "lessons"} onclick={() => (filterMode = "lessons")}>Lezioni</button>
          </div>

          <button type="button" class="pill-btn pill-btn-primary btn-xs" onclick={() => (navStore.isTimetableModalOpen = true)}>
            <CalendarClock size={14} />
            Orario Lezioni
          </button>
        </div>
      </div>

      <div class="weekdays-row">
        {#each weekdayHeaders as wh (wh.jsDay)}
          <div class="wh-cell">{wh.shortLabel}</div>
        {/each}
      </div>

      {#if calendarMode === "month"}
        <div class="month-grid">
          {#each cells as cell (cell.isoDate)}
            {@const d = indexed.byDate.get(cell.isoDate)}
            {@const isSelected = navStore.calendarSelectedDate === cell.isoDate}
            {@const tasks = filterMode === "all" || filterMode === "tasks" ? (d?.tasks ?? []) : []}
            {@const exams = filterMode === "all" || filterMode === "exams" ? (d?.exams ?? []) : []}
            {@const lessons = filterMode === "all" || filterMode === "lessons" ? (d?.events.filter((e) => e.category === "lezione") ?? []) : []}
            {@const extraEvents = filterMode === "all" ? (d?.events.filter((e) => e.category !== "lezione") ?? []) : []}
            <button
              type="button"
              class="day-cell"
              class:muted={!cell.isCurrentMonth}
              class:today={cell.isToday}
              class:selected={isSelected}
              onclick={() => (navStore.calendarSelectedDate = cell.isoDate)}
            >
              <div class="cell-top">
                <span class="day-num">{cell.day}</span>
                {#if d && d.plannedHours > 0}
                  <span class="hours-badge">{d.plannedHours}h</span>
                {/if}
              </div>

              {#if d?.holiday}
                <div class="pill-tag holiday">{d.holiday.name}</div>
              {/if}

              <div class="cell-items">
                {#each exams as ex (ex.id)}
                  <div class="pill-tag exam">{ex.title}</div>
                {/each}

                {#each tasks.slice(0, 3) as t (t.id)}
                  {@const c = indexed.courseById.get(t.courseId ?? "")}
                  <div
                    class="pill-tag task"
                    class:done={t.done}
                    style:--dot={c?.colorHex ?? getCategoryColor(t.category, themeStore.settings)}
                  >
                    <span class="dot"></span>
                    <span class="lbl">{c ? c.sigla : ""} {t.title}</span>
                  </div>
                {/each}

                {#if tasks.length > 3}
                  <span class="more-lbl">+{tasks.length - 3} attività</span>
                {/if}

                {#each lessons.slice(0, 2) as l (l.id)}
                  <div class="pill-tag lesson">{l.timeSlot} {l.title}</div>
                {/each}

                {#each extraEvents.slice(0, 1) as ev (ev.id)}
                  <div class="pill-tag personal">{ev.title}</div>
                {/each}
              </div>
            </button>
          {/each}
        </div>
      {:else}
        <div class="continuous-scroll">
          {#each continuousMonths as mBlock (mBlock.key)}
            <div class="cont-month-block">
              <div class="cont-month-title">{mBlock.title}</div>
              <div class="month-grid cont">
                {#each mBlock.cells as cell (cell.isoDate)}
                  {@const d = indexed.byDate.get(cell.isoDate)}
                  {@const isSelected = navStore.calendarSelectedDate === cell.isoDate}
                  {@const tasks = filterMode === "all" || filterMode === "tasks" ? (d?.tasks ?? []) : []}
                  {@const exams = filterMode === "all" || filterMode === "exams" ? (d?.exams ?? []) : []}
                  {@const lessons = filterMode === "all" || filterMode === "lessons" ? (d?.events.filter((e) => e.category === "lezione") ?? []) : []}
                  <button
                    type="button"
                    class="day-cell"
                    class:muted={!cell.isCurrentMonth}
                    class:today={cell.isToday}
                    class:selected={isSelected}
                    onclick={() => (navStore.calendarSelectedDate = cell.isoDate)}
                  >
                    <div class="cell-top">
                      <span class="day-num">{cell.day}</span>
                      {#if d && d.plannedHours > 0}
                        <span class="hours-badge">{d.plannedHours}h</span>
                      {/if}
                    </div>
                    <div class="cell-items">
                      {#each exams as ex (ex.id)}
                        <div class="pill-tag exam">{ex.title}</div>
                      {/each}
                      {#each tasks.slice(0, 2) as t (t.id)}
                        {@const c = indexed.courseById.get(t.courseId ?? "")}
                        <div
                          class="pill-tag task"
                          class:done={t.done}
                          style:--dot={c?.colorHex ?? getCategoryColor(t.category, themeStore.settings)}
                        >
                          <span class="dot"></span>
                          <span class="lbl">{c ? c.sigla : ""} {t.title}</span>
                        </div>
                      {/each}
                      {#each lessons.slice(0, 1) as l (l.id)}
                        <div class="pill-tag lesson">{l.timeSlot} {l.title}</div>
                      {/each}
                    </div>
                  </button>
                {/each}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Pannello Ispettore Giornaliero sulla destra -->
    <aside class="bento-card day-panel">
      <div class="panel-head">
        <div>
          <span class="panel-eyebrow">Ispettore Giorno</span>
          <h3>{formatItalianDateLong(navStore.calendarSelectedDate)}</h3>
        </div>
        <button
          type="button"
          class="pill-btn pill-btn-primary btn-xs"
          onclick={() => navStore.openDayInTodayView(navStore.calendarSelectedDate)}
        >
          <ExternalLink size={13} />
          Espandi
        </button>
      </div>

      {#if selectedDayData?.operationalPlan}
        <div class="op-preview">
          <div class="op-tag"><Sparkles size={12} /> Piano Operativo</div>
          <p>{selectedDayData.operationalPlan.dailyGoal}</p>
        </div>
      {/if}

      <div class="panel-section">
        <h4>Attività ({selectedDayData?.tasks.length ?? 0})</h4>
        {#if !selectedDayData || selectedDayData.tasks.length === 0}
          <p class="empty-note">Nessuna task in questa data.</p>
        {:else}
          <div class="panel-list">
            {#each selectedDayData.tasks as t (t.id)}
              {@const c = indexed.courseById.get(t.courseId ?? "")}
              <button
                type="button"
                class="panel-task"
                class:done={t.done}
                onclick={() => sessionStore.toggleTask(t.id, !t.done)}
              >
                <span class="cat-badge" style:background={getCategoryColor(t.category, themeStore.settings)}>
                  {getCategoryLabel(t.category)}
                </span>
                <div class="pt-body">
                  <strong>{c ? `${c.sigla} · ` : ""}{t.title}</strong>
                  <span>{t.chapter}{t.pages ? ` · ${t.pages}` : ""} · {t.plannedHours}h</span>
                </div>
              </button>
            {/each}
          </div>
        {/if}
      </div>

      {#if selectedDayData && (selectedDayData.exams.length > 0 || selectedDayData.events.length > 0)}
        <div class="panel-section">
          <h4>Esami & Lezioni</h4>
          <div class="panel-list">
            {#each selectedDayData.exams as ex (ex.id)}
              <div class="panel-event exam">{ex.title} {ex.timeSlot ? `(${ex.timeSlot})` : ""}</div>
            {/each}
            {#each selectedDayData.events as ev (ev.id)}
              <div class="panel-event">{ev.timeSlot ? `${ev.timeSlot} · ` : ""}{ev.title}</div>
            {/each}
          </div>
        </div>
      {/if}

      {#if selectedDayData && selectedDayData.quizzes.length > 0}
        <div class="panel-section">
          <h4>Quiz del giorno ({selectedDayData.quizzes.length})</h4>
          <div class="panel-list">
            {#each selectedDayData.quizzes as q (q.id)}
              <button
                type="button"
                class="panel-quiz"
                onclick={() => {
                  navStore.activeQuizId = q.id;
                  navStore.navigate("quiz");
                }}
              >
                <span>{q.title}</span>
                <strong>{q.latestAttempt ? `${q.latestAttempt.scorePct}%` : "Avvia"}</strong>
              </button>
            {/each}
          </div>
        </div>
      {/if}

      <div class="panel-section">
        <h4>Promemoria rapido</h4>
        <div class="quick-row">
          <input
            type="text"
            class="soft-input"
            placeholder="Aggiungi evento personale..."
            bind:value={quickEventTitle}
            onkeydown={(e) => e.key === "Enter" && addQuickEvent()}
          />
          <button type="button" class="pill-btn pill-btn-soft btn-xs" onclick={addQuickEvent}>
            <Plus size={14} />
          </button>
        </div>
      </div>

      <div class="panel-section note-section">
        <h4>Note della giornata</h4>
        <textarea class="soft-textarea" rows="3" bind:value={noteDraft} placeholder="Appunti o annotazioni..."></textarea>
        <button type="button" class="pill-btn pill-btn-soft btn-xs self-end" onclick={saveNote}>Salva nota</button>
      </div>
    </aside>
  </div>
{/if}

<style>
  .cal-layout {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 340px;
    gap: 18px;
    height: calc(100vh - 32px);
  }

  .cal-main {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    overflow: hidden;
  }

  .cal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 12px;
  }

  .cal-title-row {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .cal-title-row h2 {
    margin: 0;
    font-size: 20px;
    font-weight: 800;
  }

  .nav-arrows {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .icon-pill {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: var(--bg-surface-muted);
  }

  .cal-tools {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }

  .filter-pills {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .weekdays-row {
    display: grid;
    grid-template-columns: repeat(7, minmax(0, 1fr));
    gap: 6px;
  }

  .wh-cell {
    text-align: center;
    font-size: 11.5px;
    font-weight: 700;
    color: var(--text-secondary);
    padding: 4px 0;
  }

  .month-grid {
    flex: 1;
    display: grid;
    grid-template-columns: repeat(7, minmax(0, 1fr));
    grid-auto-rows: minmax(92px, 1fr);
    gap: 6px;
    overflow-y: auto;
  }

  .continuous-scroll {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 18px;
    padding-right: 4px;
  }

  .cont-month-block {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .cont-month-title {
    font-size: 14px;
    font-weight: 800;
    color: var(--accent);
    padding-left: 2px;
  }

  .month-grid.cont {
    overflow: visible;
    flex: none;
    grid-auto-rows: minmax(82px, auto);
  }

  .day-cell {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 7px;
    border-radius: 14px;
    background: var(--bg-surface-muted);
    border: 1.5px solid transparent;
    text-align: left;
    overflow: hidden;
    transition: border-color 0.14s ease, background-color 0.14s ease;
  }

  .day-cell:hover {
    border-color: var(--border-strong);
  }

  .day-cell.muted {
    opacity: 0.48;
  }

  .day-cell.selected {
    border-color: var(--accent);
    background: var(--bg-surface);
  }

  .day-cell.today .day-num {
    background: var(--bg-surface-dark);
    color: #fff;
  }

  :global(html[data-theme="dark"]) .day-cell.today .day-num {
    background: var(--accent);
  }

  .cell-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .day-num {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    font-size: 11.5px;
    font-weight: 700;
  }

  .hours-badge {
    font-size: 10px;
    font-weight: 700;
    color: var(--text-secondary);
  }

  .cell-items {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }

  .pill-tag {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 2px 6px;
    border-radius: 6px;
    font-size: 10.5px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .pill-tag.task {
    background: var(--bg-surface);
    color: var(--text-primary);
  }

  .pill-tag.task.done {
    opacity: 0.52;
    text-decoration: line-through;
  }

  .pill-tag.task .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--dot);
    flex-shrink: 0;
  }

  .pill-tag.task .lbl {
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .pill-tag.exam {
    background: rgba(239, 68, 68, 0.14);
    color: #DC2626;
  }

  .pill-tag.lesson {
    background: var(--accent-soft);
    color: var(--accent);
  }

  .pill-tag.holiday {
    background: rgba(245, 158, 11, 0.14);
    color: #D97706;
  }

  .pill-tag.personal {
    background: rgba(16, 185, 129, 0.14);
    color: #059669;
  }

  .more-lbl {
    font-size: 10px;
    color: var(--text-muted);
    padding-left: 4px;
  }

  .day-panel {
    padding: 18px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    overflow-y: auto;
  }

  .panel-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 8px;
  }

  .panel-eyebrow {
    font-size: 11px;
    font-weight: 700;
    color: var(--accent);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .panel-head h3 {
    margin: 2px 0 0;
    font-size: 15.5px;
    font-weight: 800;
  }

  .op-preview {
    padding: 10px 12px;
    border-radius: 14px;
    background: var(--accent-soft);
  }

  .op-tag {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 11px;
    font-weight: 700;
    color: var(--accent);
  }

  .op-preview p {
    margin: 4px 0 0;
    font-size: 12.5px;
  }

  .panel-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .panel-section h4 {
    margin: 0;
    font-size: 12.5px;
    font-weight: 700;
    color: var(--text-secondary);
  }

  .panel-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .panel-task {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 9px 10px;
    border-radius: 12px;
    background: var(--bg-surface-muted);
    text-align: left;
  }

  .panel-task.done {
    opacity: 0.55;
  }

  .cat-badge {
    align-self: flex-start;
    padding: 2px 7px;
    border-radius: 999px;
    color: #fff;
    font-size: 10px;
    font-weight: 700;
  }

  .pt-body {
    display: flex;
    flex-direction: column;
    font-size: 12.5px;
  }

  .pt-body span {
    font-size: 11px;
    color: var(--text-secondary);
  }

  .panel-event {
    padding: 7px 10px;
    border-radius: 10px;
    background: var(--bg-surface-muted);
    font-size: 12px;
    font-weight: 600;
  }

  .panel-event.exam {
    background: rgba(239, 68, 68, 0.14);
    color: #DC2626;
  }

  .panel-quiz {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 10px;
    border-radius: 12px;
    background: var(--accent-soft);
    font-size: 12px;
    font-weight: 700;
    color: var(--accent);
  }

  .quick-row {
    display: flex;
    gap: 6px;
  }

  .soft-input,
  .soft-textarea {
    width: 100%;
    padding: 8px 11px;
    border-radius: 12px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface-muted);
    color: var(--text-primary);
    font-size: 12.5px;
    resize: vertical;
  }

  .self-end {
    align-self: flex-end;
  }

  .empty-note {
    margin: 0;
    font-size: 12px;
    color: var(--text-muted);
  }

  .btn-xs {
    padding: 6px 10px;
    font-size: 12px;
  }

  @media (max-width: 1140px) {
    .cal-layout {
      grid-template-columns: 1fr;
      height: auto;
    }
  }
</style>
