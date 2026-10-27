<script lang="ts">
  import { themeStore } from "../../stores/theme.svelte";
  import type { Course, Exam } from "../../types/domain";
  import {
    formatExamRoundLabel,
    MESI_ITALIANI_SHORT,
    type CalendarDayCell,
    type DayAggregatedData
  } from "../../utils/calendar";
  import { generateCourseColorTokens } from "../../utils/color";

  interface Props {
    cell: CalendarDayCell;
    dayData?: DayAggregatedData | null;
    courseById: Map<string, Course>;
    isSelected?: boolean;
    selectedCourseId?: string | null;
    compact?: boolean;
    onSelectDay?: (isoDate: string) => void;
    onSelectExam?: (exam: Exam, e: MouseEvent) => void;
  }

  let {
    cell,
    dayData = null,
    courseById,
    isSelected = false,
    selectedCourseId = null,
    compact = false,
    onSelectDay,
    onSelectExam
  }: Props = $props();

  let isDark = $derived(themeStore.effectiveTheme === "dark");

  let visibleExams = $derived.by(() => {
    const list = dayData?.exams ?? [];
    if (!selectedCourseId) return list;
    return list.filter((e) => e.courseId === selectedCourseId);
  });

  let visibleUnits = $derived.by(() => {
    const list = dayData?.studyUnits ?? [];
    if (!selectedCourseId) return list;
    return list.filter((u) => u.courseId === selectedCourseId);
  });

  let visibleEvents = $derived(dayData?.genericEvents ?? []);
  let todos = $derived(dayData?.todos ?? []);
  let todosDone = $derived(todos.filter((t) => t.done).length);
  let conflicts = $derived(dayData?.conflicts ?? []);
  let hasCriticalConflict = $derived(conflicts.some((c) => c.severity === "critical"));
  let hasNote = $derived(Boolean(dayData?.note));
  let workloadHours = $derived(dayData?.workloadHours ?? 0);

  function getCategoryColor(cat: string): string {
    switch (cat) {
      case "studio":
        return themeStore.settings.catStudioColor || "#007AFF";
      case "scadenza":
        return themeStore.settings.catScadenzaColor || "#FF9500";
      case "lezione":
        return themeStore.settings.catLezioneColor || "#AF52DE";
      default:
        return themeStore.settings.catPersonaleColor || "#34C759";
    }
  }

  function shortRoundTag(round: string): string {
    if (round === "itinere_1") return "Parz. 1";
    if (round === "itinere_2") return "Parz. 2";
    if (round.startsWith("1")) return "1° App";
    if (round.startsWith("2")) return "2° App";
    if (round.startsWith("3")) return "3° App";
    return round;
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  role="gridcell"
  tabindex="0"
  aria-selected={isSelected}
  class="calendar-day-cell"
  class:outside-month={!cell.isCurrentMonth}
  class:is-today={cell.isToday}
  class:is-selected={isSelected}
  class:is-holiday={Boolean(dayData?.holiday)}
  class:is-weekly-rest={cell.isWeeklyHoliday}
  class:has-conflict={conflicts.length > 0}
  class:compact
  onclick={() => onSelectDay?.(cell.isoDate)}
>
  <!-- Top Row: Numero Giorno + Festività / Indicatori Conflitto -->
  <div class="day-cell-header">
    <div class="day-number-group">
      <span
        class="day-number-badge"
        style:color={dayData?.holiday
          ? themeStore.settings.holidayColor
          : cell.isWeeklyHoliday && !cell.isToday
            ? themeStore.settings.sundayColor
            : undefined}
      >
        {cell.day}
        {#if cell.day === 1}
          <small class="month-inline-tag">{MESI_ITALIANI_SHORT[cell.month - 1]}</small>
        {/if}
      </span>

      {#if dayData?.holiday}
        <span
          class="holiday-name-label"
          style:color={themeStore.settings.holidayColor}
          title={dayData.holiday.name}
        >
          {dayData.holiday.name}
        </span>
      {/if}
    </div>

    <div class="day-meta-indicators">
      {#if conflicts.length > 0}
        <span
          class="conflict-cell-badge"
          class:critical={hasCriticalConflict}
          title={conflicts.map((c) => `${c.title}: ${c.description}`).join("\n")}
        >
          !
        </span>
      {/if}
      {#if hasNote}
        <span class="note-dot" title="Nota giornaliera presente"></span>
      {/if}
      {#if todos.length > 0}
        <span class="todo-counter-pill" class:all-done={todosDone === todos.length}>
          ✓ {todosDone}/{todos.length}
        </span>
      {/if}
    </div>
  </div>

  <!-- Body: Pillole Esami, Eventi Generici e Unità Didattiche Pianificate -->
  <div class="day-items-stack">
    {#each visibleExams as exam (exam.id)}
      {@const course = courseById.get(exam.courseId)}
      {@const palette = generateCourseColorTokens(course?.colorHex ?? "#007AFF", isDark)}
      <button
        type="button"
        class="exam-cell-pill"
        style:background={palette.bgTint}
        style:border-color={palette.borderTint}
        style:color={palette.textContrast}
        title="{course?.name ?? 'Esame'} · {formatExamRoundLabel(exam.round)}{exam.timeSlot ? ` (${exam.timeSlot})` : ''}{exam.classroom ? ` — ${exam.classroom}` : ''}"
        onclick={(e) => {
          e.stopPropagation();
          if (onSelectExam) {
            onSelectExam(exam, e);
          } else {
            onSelectDay?.(cell.isoDate);
          }
        }}
      >
        <span class="exam-color-bar" style:background={palette.solid}></span>
        <span class="exam-sigla">{course?.sigla ?? "ESAME"}</span>
        <span class="exam-round-tag">{shortRoundTag(exam.round)}</span>
        {#if exam.timeSlot && !compact}
          <span class="exam-time">{exam.timeSlot}</span>
        {/if}
      </button>
    {/each}

    {#each visibleEvents.slice(0, compact ? 1 : 2) as ev (ev.id)}
      {@const catColor = getCategoryColor(ev.category)}
      <div class="generic-event-chip" title="{ev.title} ({ev.category})">
        <span class="event-dot" style:background={catColor}></span>
        <span class="event-title">{ev.title}</span>
      </div>
    {/each}

    {#if visibleUnits.length > 0}
      <div class="units-chip-row">
        {#each visibleUnits.slice(0, 3) as u (u.id)}
          {@const c = courseById.get(u.courseId)}
          <span
            class="unit-mini-tag"
            class:completed={u.isCompleted}
            title="{c?.sigla ?? ''} · Unità {u.unitNumber}: {u.title}"
          >
            {c?.sigla ?? "U"}#{u.unitNumber}
          </span>
        {/each}
        {#if visibleUnits.length > 3}
          <span class="unit-more">+{visibleUnits.length - 3}</span>
        {/if}
      </div>
    {/if}
  </div>

  <!-- Barra inferiore Carico Studio Giornaliero -->
  {#if workloadHours > 0}
    <div class="workload-bottom-bar" title="Carico studio consigliato: {workloadHours.toFixed(1)}h">
      <div
        class="workload-fill"
        style:width="{Math.min(100, Math.round((workloadHours / 8) * 100))}%"
      ></div>
    </div>
  {/if}
</div>

<style>
  .calendar-day-cell {
    position: relative;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    min-height: 6.4rem;
    padding: 0.42rem 0.48rem;
    background: var(--calendar-cell-bg);
    border-right: 0.5px solid var(--hairline-border);
    border-bottom: 0.5px solid var(--hairline-border);
    cursor: pointer;
    overflow: hidden;
    user-select: none;
    transition: background-color 120ms ease;
  }

  .calendar-day-cell.compact {
    min-height: 5.5rem;
  }

  .calendar-day-cell:hover {
    background: var(--calendar-cell-hover);
  }

  .calendar-day-cell.outside-month {
    background: var(--calendar-cell-outside);
    opacity: 0.56;
  }

  .calendar-day-cell.is-today {
    background: var(--calendar-cell-today);
  }

  .calendar-day-cell.is-selected {
    background: var(--calendar-cell-selected);
    box-shadow: inset 0 0 0 1.5px var(--accent-primary);
  }

  .calendar-day-cell.has-conflict {
    box-shadow: inset 0 0 0 1px rgba(255, 69, 58, 0.55);
  }

  .day-cell-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.25rem;
    margin-bottom: 0.28rem;
  }

  .day-number-group {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    min-width: 0;
  }

  .day-number-badge {
    display: inline-flex;
    align-items: baseline;
    gap: 0.2rem;
    font-size: 0.78rem;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    color: var(--text-primary);
    line-height: 1;
  }

  .is-today .day-number-badge {
    background: var(--accent-primary);
    color: #ffffff;
    padding: 0.18rem 0.42rem;
    border-radius: var(--radius-pill);
  }

  .month-inline-tag {
    font-size: 0.62rem;
    font-weight: 600;
    opacity: 0.85;
  }

  .holiday-name-label {
    font-size: 0.6rem;
    font-weight: 700;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 5.8rem;
  }

  .day-meta-indicators {
    display: inline-flex;
    align-items: center;
    gap: 0.22rem;
    flex-shrink: 0;
  }

  .conflict-cell-badge {
    width: 14px;
    height: 14px;
    border-radius: 999px;
    background: var(--conflict-warning);
    color: #000;
    font-size: 0.62rem;
    font-weight: 800;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .conflict-cell-badge.critical {
    background: var(--conflict-critical);
    color: #fff;
  }

  .note-dot {
    width: 6px;
    height: 6px;
    border-radius: 999px;
    background: var(--accent-primary);
  }

  .todo-counter-pill {
    font-size: 0.58rem;
    font-weight: 700;
    padding: 0.06rem 0.3rem;
    border-radius: var(--radius-pill);
    background: var(--surface-pill);
    color: var(--text-secondary);
    font-variant-numeric: tabular-nums;
  }

  .todo-counter-pill.all-done {
    color: #34c759;
  }

  /* Items Stack */
  .day-items-stack {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    min-width: 0;
  }

  .exam-cell-pill {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 0.28rem;
    padding: 0.18rem 0.36rem;
    border-radius: 0.36rem;
    border: 0.5px solid transparent;
    font-size: 0.66rem;
    font-weight: 700;
    text-align: left;
    cursor: pointer;
    overflow: hidden;
    transition: transform 120ms var(--ease-out);
  }

  .exam-cell-pill:hover {
    transform: translateX(1px);
  }

  .exam-cell-pill:active {
    transform: scale(0.97);
  }

  .exam-color-bar {
    width: 3px;
    height: 0.75rem;
    border-radius: 999px;
    flex-shrink: 0;
  }

  .exam-sigla {
    font-weight: 800;
    letter-spacing: -0.01em;
    flex-shrink: 0;
  }

  .exam-round-tag {
    font-size: 0.6rem;
    font-weight: 600;
    opacity: 0.9;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .exam-time {
    margin-left: auto;
    font-size: 0.58rem;
    opacity: 0.82;
    font-variant-numeric: tabular-nums;
  }

  .generic-event-chip {
    display: flex;
    align-items: center;
    gap: 0.28rem;
    padding: 0.1rem 0.25rem;
    font-size: 0.62rem;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .event-dot {
    width: 5px;
    height: 5px;
    border-radius: 999px;
    flex-shrink: 0;
  }

  .event-title {
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .units-chip-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.18rem;
    margin-top: 0.05rem;
  }

  .unit-mini-tag {
    font-size: 0.56rem;
    font-weight: 700;
    padding: 0.05rem 0.28rem;
    border-radius: 0.25rem;
    background: var(--surface-pill);
    border: 0.5px solid var(--hairline-border);
    color: var(--text-secondary);
  }

  .unit-mini-tag.completed {
    text-decoration: line-through;
    opacity: 0.6;
  }

  .unit-more {
    font-size: 0.56rem;
    color: var(--text-tertiary);
  }

  .workload-bottom-bar {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    height: 2px;
    background: transparent;
  }

  .workload-fill {
    height: 100%;
    background: var(--accent-primary);
    opacity: 0.55;
  }
</style>
