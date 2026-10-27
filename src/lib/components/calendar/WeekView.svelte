<script lang="ts">
  import { themeStore } from "../../stores/theme.svelte";
  import type { Course, Exam } from "../../types/domain";
  import {
    buildWeekGrid,
    formatExamRoundLabel,
    formatItalianDateShort,
    getWeekdayHeaders,
    type DayAggregatedData
  } from "../../utils/calendar";
  import { generateCourseColorTokens } from "../../utils/color";

  interface Props {
    anchorIso: string;
    byDate: Map<string, DayAggregatedData>;
    courseById: Map<string, Course>;
    selectedDate?: string | null;
    selectedCourseId?: string | null;
    todayIso: string;
    onSelectDay?: (isoDate: string) => void;
    onSelectExam?: (exam: Exam, e: MouseEvent) => void;
    onStepWeek?: (deltaWeeks: -1 | 1) => void;
  }

  let {
    anchorIso,
    byDate,
    courseById,
    selectedDate = null,
    selectedCourseId = null,
    todayIso,
    onSelectDay,
    onSelectExam,
    onStepWeek
  }: Props = $props();

  let isDark = $derived(themeStore.effectiveTheme === "dark");
  let startDayOfWeek = $derived(themeStore.settings.startDayOfWeek ?? 1);
  let weeklyHolidayDay = $derived(themeStore.settings.weeklyHolidayDay ?? 0);

  let weekdayHeaders = $derived(getWeekdayHeaders(startDayOfWeek, weeklyHolidayDay));
  let weekCells = $derived(
    buildWeekGrid(anchorIso, startDayOfWeek, weeklyHolidayDay, todayIso)
  );

  let weekRangeLabel = $derived.by(() => {
    if (weekCells.length === 0) return "";
    const first = weekCells[0];
    const last = weekCells[weekCells.length - 1];
    return `${formatItalianDateShort(first.isoDate)} – ${formatItalianDateShort(last.isoDate)} ${last.year}`;
  });
</script>

<div class="week-view-shell" aria-label="Vista Settimanale {weekRangeLabel}">
  <header class="week-top-bar">
    <h2 class="week-heading">Settimana · {weekRangeLabel}</h2>
    <div class="week-actions">
      <button type="button" class="step-btn pressable" onclick={() => onStepWeek?.(-1)}>
        ‹ Sett. Prec.
      </button>
      <button type="button" class="step-btn pressable" onclick={() => onStepWeek?.(1)}>
        Sett. Succ. ›
      </button>
    </div>
  </header>

  <div class="week-columns-grid">
    {#each weekCells as cell, idx (cell.isoDate)}
      {@const wh = weekdayHeaders[idx]}
      {@const dayData = byDate.get(cell.isoDate) ?? null}
      {@const exams = (dayData?.exams ?? []).filter((e) => !selectedCourseId || e.courseId === selectedCourseId)}
      {@const units = (dayData?.studyUnits ?? []).filter((u) => !selectedCourseId || u.courseId === selectedCourseId)}
      {@const events = dayData?.genericEvents ?? []}
      {@const todos = dayData?.todos ?? []}
      {@const conflicts = dayData?.conflicts ?? []}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div
        role="gridcell"
        tabindex="0"
        class="week-day-column"
        class:is-today={cell.isToday}
        class:is-selected={selectedDate === cell.isoDate}
        onclick={() => onSelectDay?.(cell.isoDate)}
      >
        <div class="col-header">
          <span
            class="col-weekday"
            style:color={dayData?.holiday
              ? themeStore.settings.holidayColor
              : wh?.isWeeklyHoliday
                ? themeStore.settings.sundayColor
                : undefined}
          >
            {wh?.shortLabel ?? ""}
          </span>
          <span class="col-day-num" class:today-pill={cell.isToday}>{cell.day}</span>
          {#if dayData?.holiday}
            <span class="col-holiday" style:color={themeStore.settings.holidayColor}>
              {dayData.holiday.name}
            </span>
          {/if}
        </div>

        <div class="col-body">
          {#if conflicts.length > 0}
            <div class="week-conflict-banner">
              {#each conflicts as cf (cf.id)}
                <div class="cf-msg">⚠ {cf.title}: {cf.description}</div>
              {/each}
            </div>
          {/if}

          <!-- Esami -->
          {#each exams as exam (exam.id)}
            {@const course = courseById.get(exam.courseId)}
            {@const palette = generateCourseColorTokens(course?.colorHex ?? "#007AFF", isDark)}
            <button
              type="button"
              class="week-exam-card pressable"
              style:background={palette.bgTint}
              style:border-color={palette.borderTint}
              style:color={palette.textContrast}
              onclick={(e) => {
                e.stopPropagation();
                onSelectExam?.(exam, e);
              }}
            >
              <div class="we-top">
                <span class="we-sigla" style:background={palette.solid} style:color={palette.badgeText}>
                  {course?.sigla ?? "ESAME"}
                </span>
                {#if exam.timeSlot}
                  <span class="we-time">{exam.timeSlot}</span>
                {/if}
              </div>
              <strong class="we-name">{course?.name ?? "Appello"}</strong>
              <span class="we-round">{formatExamRoundLabel(exam.round)}</span>
              {#if exam.classroom}
                <span class="we-room">📍 {exam.classroom}</span>
              {/if}
            </button>
          {/each}

          <!-- Eventi Generici -->
          {#each events as ev (ev.id)}
            <div class="week-generic-card">
              <div class="wg-top">
                <span class="wg-cat">{ev.category}</span>
                {#if ev.timeSlot}
                  <span class="wg-time">{ev.timeSlot}</span>
                {/if}
              </div>
              <strong class="wg-title">{ev.title}</strong>
            </div>
          {/each}

          <!-- Unità Didattiche -->
          {#if units.length > 0}
            <div class="week-section-block">
              <span class="ws-title">UNITÀ DIDATTICHE ({units.length})</span>
              {#each units as u (u.id)}
                {@const c = courseById.get(u.courseId)}
                <div class="week-unit-row" class:done={u.isCompleted}>
                  <span class="wu-badge">{c?.sigla ?? "U"} #{u.unitNumber}</span>
                  <span class="wu-name">{u.title}</span>
                </div>
              {/each}
            </div>
          {/if}

          <!-- To-Do Giornalieri -->
          {#if todos.length > 0}
            <div class="week-section-block">
              <span class="ws-title">TO-DO ({todos.filter((t) => t.done).length}/{todos.length})</span>
              {#each todos as td (td.id)}
                <div class="week-todo-row" class:done={td.done}>
                  <span>{td.done ? "✓" : "○"}</span>
                  <span>{td.text}</span>
                </div>
              {/each}
            </div>
          {/if}
        </div>

        {#if dayData && dayData.workloadHours > 0}
          <footer class="col-footer">
            <span>Studio consigliato: <strong>{dayData.workloadHours.toFixed(1)}h</strong></span>
          </footer>
        {/if}
      </div>
    {/each}
  </div>
</div>

<style>
  .week-view-shell {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 0.75rem 1rem 1rem;
    overflow: hidden;
  }

  .week-top-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.6rem;
    flex-shrink: 0;
  }

  .week-heading {
    margin: 0;
    font-family: var(--font-display);
    font-size: 1.3rem;
    font-weight: 800;
    letter-spacing: -0.024em;
  }

  .week-actions {
    display: flex;
    gap: 0.4rem;
  }

  .step-btn {
    height: 1.85rem;
    padding: 0 0.7rem;
    border-radius: var(--radius-xs);
    border: 0.5px solid var(--hairline-border);
    background: var(--surface-pill);
    color: var(--text-primary);
    font-size: 0.74rem;
    font-weight: 600;
    cursor: pointer;
  }

  .week-columns-grid {
    flex: 1;
    display: grid;
    grid-template-columns: repeat(7, minmax(0, 1fr));
    border-radius: var(--radius-md);
    border: 0.5px solid var(--hairline-border);
    overflow: hidden;
    min-height: 0;
  }

  .week-day-column {
    display: flex;
    flex-direction: column;
    background: var(--calendar-cell-bg);
    border-right: 0.5px solid var(--hairline-border);
    cursor: pointer;
    min-height: 0;
  }

  .week-day-column:last-child {
    border-right: none;
  }

  .week-day-column.is-today {
    background: var(--calendar-cell-today);
  }

  .week-day-column.is-selected {
    box-shadow: inset 0 0 0 1.5px var(--accent-primary);
  }

  .col-header {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.18rem;
    padding: 0.6rem 0.65rem;
    background: var(--calendar-header-bg);
    border-bottom: 0.5px solid var(--hairline-border);
  }

  .col-weekday {
    font-size: 0.66rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-tertiary);
  }

  .col-day-num {
    font-size: 1.15rem;
    font-weight: 800;
    letter-spacing: -0.02em;
  }

  .col-day-num.today-pill {
    padding: 0.1rem 0.48rem;
    border-radius: var(--radius-pill);
    background: var(--accent-primary);
    color: #fff;
  }

  .col-holiday {
    font-size: 0.62rem;
    font-weight: 700;
  }

  .col-body {
    flex: 1;
    overflow-y: auto;
    padding: 0.55rem;
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
  }

  .week-conflict-banner {
    padding: 0.38rem 0.45rem;
    border-radius: var(--radius-xs);
    background: rgba(255, 69, 58, 0.15);
    border: 0.5px solid rgba(255, 69, 58, 0.45);
    color: #ff6961;
    font-size: 0.64rem;
    font-weight: 600;
  }

  .week-exam-card {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    padding: 0.5rem 0.55rem;
    border-radius: var(--radius-sm);
    border: 0.5px solid transparent;
    text-align: left;
    cursor: pointer;
  }

  .we-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .we-sigla {
    font-size: 0.62rem;
    font-weight: 800;
    padding: 0.08rem 0.35rem;
    border-radius: 0.25rem;
  }

  .we-time {
    font-size: 0.62rem;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }

  .we-name {
    font-size: 0.74rem;
    font-weight: 700;
  }

  .we-round,
  .we-room {
    font-size: 0.64rem;
    opacity: 0.88;
  }

  .week-generic-card {
    padding: 0.42rem 0.5rem;
    border-radius: var(--radius-xs);
    background: var(--surface-card);
    border: 0.5px solid var(--hairline-border);
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .wg-top {
    display: flex;
    justify-content: space-between;
    font-size: 0.6rem;
    text-transform: uppercase;
    color: var(--text-tertiary);
    font-weight: 700;
  }

  .wg-title {
    font-size: 0.72rem;
  }

  .week-section-block {
    padding: 0.4rem 0.45rem;
    border-radius: var(--radius-xs);
    background: var(--surface-pill);
    display: flex;
    flex-direction: column;
    gap: 0.22rem;
  }

  .ws-title {
    font-size: 0.58rem;
    font-weight: 700;
    letter-spacing: 0.04em;
    color: var(--text-tertiary);
  }

  .week-unit-row,
  .week-todo-row {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.66rem;
    color: var(--text-secondary);
  }

  .week-unit-row.done,
  .week-todo-row.done {
    text-decoration: line-through;
    opacity: 0.6;
  }

  .wu-badge {
    font-weight: 700;
    color: var(--text-primary);
  }

  .col-footer {
    padding: 0.38rem 0.55rem;
    border-top: 0.5px solid var(--hairline-border);
    font-size: 0.64rem;
    color: var(--text-secondary);
  }
</style>
