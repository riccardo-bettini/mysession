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

  let visibleEvents = $derived.by(() => {
    const list = dayData?.genericEvents ?? [];
    return list.filter((ev) => {
      if (ev.category === "lezione" && !themeStore.showLessonTimes) {
        return false;
      }
      return true;
    });
  });

  let todos = $derived(dayData?.todos ?? []);
  let todosDone = $derived(todos.filter((t) => t.done).length);
  let conflicts = $derived(dayData?.conflicts ?? []);
  let hasCriticalConflict = $derived(conflicts.some((c) => c.severity === "critical"));
  let hasNote = $derived(Boolean(dayData?.note));
  let workloadHours = $derived(dayData?.workloadHours ?? 0);

  // Riquadri compatti per ciascuna materia da fare nel giorno (derivata da unità didattiche e to-do di corso)
  let daySubjectItems = $derived.by(() => {
    const map = new Map<
      string,
      {
        course: Course;
        unitsCount: number;
        unitsDoneCount: number;
        todosCount: number;
        todosDoneCount: number;
        isAllDone: boolean;
      }
    >();

    for (const u of visibleUnits) {
      const c = courseById.get(u.courseId);
      if (!c || c.isVisible === false) continue;
      const existing = map.get(c.id);
      if (existing) {
        existing.unitsCount++;
        if (u.isCompleted) existing.unitsDoneCount++;
      } else {
        map.set(c.id, {
          course: c,
          unitsCount: 1,
          unitsDoneCount: u.isCompleted ? 1 : 0,
          todosCount: 0,
          todosDoneCount: 0,
          isAllDone: false
        });
      }
    }

    for (const t of todos) {
      if (!t.courseId) continue;
      if (selectedCourseId && t.courseId !== selectedCourseId) continue;
      const c = courseById.get(t.courseId);
      if (!c || c.isVisible === false) continue;
      const existing = map.get(c.id);
      if (existing) {
        existing.todosCount++;
        if (t.done) existing.todosDoneCount++;
      } else {
        map.set(c.id, {
          course: c,
          unitsCount: 0,
          unitsDoneCount: 0,
          todosCount: 1,
          todosDoneCount: t.done ? 1 : 0,
          isAllDone: false
        });
      }
    }

    const res = Array.from(map.values());
    for (const item of res) {
      const total = item.unitsCount + item.todosCount;
      const done = item.unitsDoneCount + item.todosDoneCount;
      item.isAllDone = total > 0 && done === total;
    }
    return res;
  });

  // Quota dinamica: esami ed eventi usano la quota verticale; i chip materie hanno una riga compatta dedicata
  let maxDisplayItems = $derived(compact ? 3 : 4);

  let displayedExams = $derived(visibleExams.slice(0, 3));
  let remainingQuotaAfterExams = $derived(Math.max(0, maxDisplayItems - displayedExams.length));
  let displayedEvents = $derived(visibleEvents.slice(0, remainingQuotaAfterExams));

  let maxSubjectChips = $derived(compact ? 4 : 6);
  let displayedSubjects = $derived(daySubjectItems.slice(0, maxSubjectChips));

  let overflowCount = $derived(
    Math.max(0, visibleExams.length - displayedExams.length) +
      Math.max(0, visibleEvents.length - displayedEvents.length) +
      Math.max(0, daySubjectItems.length - displayedSubjects.length)
  );

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
  data-calendar-day={cell.isoDate}
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

  <!-- Body: Pillole Esami, Impegni Diretti, To-Do List e Unità Didattiche -->
  <div class="day-items-stack">
    <!-- 1. Esami / Appelli -->
    {#each displayedExams as exam (exam.id)}
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

    <!-- 2. Impegni Giornalieri / Lezioni (orario completo se attivo, altrimenti nascoste) -->
    {#each displayedEvents as ev (ev.id)}
      {@const catColor = getCategoryColor(ev.category)}
      <div class="generic-event-chip" title="{ev.title} ({ev.category}){ev.timeSlot ? ` · ${ev.timeSlot}` : ''}">
        <span class="event-dot" style:background={catColor}></span>
        {#if ev.timeSlot}
          <span class="event-time">{ev.timeSlot}</span>
        {/if}
        <span class="event-title">{ev.title}</span>
      </div>
    {/each}

    <!-- 3. Riquadri Compatti delle Materie del Giorno (Sigla + stato completamento) -->
    {#if displayedSubjects.length > 0}
      <div class="subject-chips-row">
        {#each displayedSubjects as item (item.course.id)}
          {@const palette = generateCourseColorTokens(item.course.colorHex, isDark)}
          <div
            class="subject-chip-pill"
            class:all-done={item.isAllDone}
            style:background={palette.bgTint}
            style:border-color={palette.borderTint}
            style:color={palette.textContrast}
            title="{item.course.name} ({item.course.sigla}){item.todosCount > 0 ? ` — ${item.todosDoneCount}/${item.todosCount} task` : ''}"
          >
            <span class="subject-chip-dot" style:background={palette.solid}></span>
            <span class="subject-chip-sigla">{item.course.sigla}</span>
            {#if item.isAllDone}
              <span class="subject-chip-check">✓</span>
            {/if}
          </div>
        {/each}
      </div>
    {/if}

    <!-- 4. Indicatore Overflow (+N altri elementi del giorno) -->
    {#if overflowCount > 0}
      <div class="items-overflow-badge" title="Clicca per aprire tutti gli impegni del giorno">
        +{overflowCount} altri
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
    min-height: 6.6rem;
    padding: 0.42rem 0.5rem;
    background: var(--calendar-cell-bg);
    border-right: 0.5px solid var(--hairline-border);
    border-bottom: 0.5px solid var(--hairline-border);
    cursor: pointer;
    overflow: hidden;
    user-select: none;
    transition: background-color 110ms ease;
  }

  .calendar-day-cell.compact {
    min-height: 5.5rem;
  }

  .calendar-day-cell:hover {
    background: var(--calendar-cell-hover);
  }

  .calendar-day-cell:focus-visible {
    outline: none;
    box-shadow: inset 0 0 0 2px var(--accent-primary);
  }

  .calendar-day-cell.outside-month {
    background: var(--calendar-cell-outside);
    opacity: 0.54;
  }

  .calendar-day-cell.is-today {
    background: var(--calendar-cell-today);
  }

  .calendar-day-cell.is-selected {
    background: var(--calendar-cell-selected);
    box-shadow: inset 0 0 0 1.5px var(--accent-primary);
  }

  .calendar-day-cell.has-conflict {
    box-shadow: inset 0 0 0 1.5px rgba(255, 69, 58, 0.65);
  }

  .day-cell-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.25rem;
    margin-bottom: 0.3rem;
  }

  .day-number-group {
    display: flex;
    align-items: center;
    gap: 0.32rem;
    min-width: 0;
  }

  .day-number-badge {
    display: inline-flex;
    align-items: baseline;
    gap: 0.2rem;
    font-size: 0.8rem;
    font-weight: 700;
    letter-spacing: var(--tracking-normal);
    font-variant-numeric: tabular-nums;
    color: var(--text-primary);
    line-height: 1;
  }

  .is-today .day-number-badge {
    background: var(--accent-primary);
    color: #ffffff;
    padding: 0.16rem 0.44rem;
    border-radius: var(--radius-pill);
    box-shadow: 0 1px 3px rgba(10, 132, 255, 0.35);
  }

  .month-inline-tag {
    font-size: 0.62rem;
    font-weight: 600;
    opacity: 0.85;
    letter-spacing: var(--tracking-normal);
  }

  .holiday-name-label {
    font-size: 0.6rem;
    font-weight: 700;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 6rem;
  }

  .day-meta-indicators {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    flex-shrink: 0;
  }

  .conflict-cell-badge {
    width: 14px;
    height: 14px;
    border-radius: var(--radius-pill);
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
    box-shadow: 0 0 6px rgba(255, 69, 58, 0.5);
  }

  .note-dot {
    width: 6px;
    height: 6px;
    border-radius: var(--radius-pill);
    background: var(--accent-primary);
  }

  .todo-counter-pill {
    font-size: 0.58rem;
    font-weight: 700;
    padding: 0.08rem 0.32rem;
    border-radius: var(--radius-pill);
    background: var(--surface-pill);
    border: 0.5px solid var(--hairline-border);
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
    gap: 0.22rem;
    min-width: 0;
  }

  .exam-cell-pill {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 0.32rem;
    padding: 0.2rem 0.4rem;
    border-radius: var(--radius-xs);
    border: 0.5px solid transparent;
    box-shadow: var(--shadow-pill);
    font-size: 0.68rem;
    font-weight: 700;
    letter-spacing: var(--tracking-normal);
    text-align: left;
    cursor: pointer;
    overflow: hidden;
    will-change: transform;
    transition: transform 120ms var(--spring-press);
  }

  .exam-cell-pill:hover {
    transform: translateX(1px);
  }

  .exam-cell-pill:active {
    transform: scale(0.97);
  }

  .exam-color-bar {
    width: 3px;
    height: 0.8rem;
    border-radius: var(--radius-pill);
    flex-shrink: 0;
  }

  .exam-sigla {
    font-weight: 800;
    letter-spacing: var(--tracking-wide);
    flex-shrink: 0;
  }

  .exam-round-tag {
    font-size: 0.62rem;
    font-weight: 600;
    opacity: 0.9;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .exam-time {
    margin-left: auto;
    font-size: 0.6rem;
    opacity: 0.85;
    font-variant-numeric: tabular-nums;
  }

  .generic-event-chip {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.14rem 0.35rem;
    font-size: 0.63rem;
    color: var(--text-secondary);
    background: rgba(255, 255, 255, 0.035);
    border: 0.5px solid rgba(255, 255, 255, 0.06);
    border-radius: 4px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .event-dot {
    width: 5px;
    height: 5px;
    border-radius: var(--radius-pill);
    flex-shrink: 0;
  }

  .event-time {
    font-size: 0.58rem;
    font-weight: 700;
    opacity: 0.85;
    flex-shrink: 0;
    font-variant-numeric: tabular-nums;
  }

  .event-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* Riquadri Compatti delle Materie del Giorno (affiancati in riga con flex-wrap) */
  .subject-chips-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.24rem;
    margin-top: 0.04rem;
  }

  .subject-chip-pill {
    display: inline-flex;
    align-items: center;
    gap: 0.22rem;
    padding: 0.14rem 0.34rem;
    border-radius: var(--radius-xs);
    border: 0.5px solid;
    font-size: 0.62rem;
    font-weight: 800;
    line-height: 1.1;
    letter-spacing: var(--tracking-wide);
    user-select: none;
    pointer-events: none;
    transition: opacity 160ms ease;
  }

  .subject-chip-pill.all-done {
    opacity: 0.68;
  }

  .subject-chip-dot {
    width: 5px;
    height: 5px;
    border-radius: 999px;
    flex-shrink: 0;
  }

  .subject-chip-sigla {
    white-space: nowrap;
  }

  .subject-chip-check {
    font-size: 0.56rem;
    font-weight: 900;
    line-height: 1;
  }

  .items-overflow-badge {
    font-size: 0.58rem;
    font-weight: 700;
    color: var(--accent-primary, #007aff);
    padding: 0.06rem 0.3rem;
    border-radius: 4px;
    background: rgba(0, 122, 255, 0.1);
    align-self: flex-start;
    margin-top: 0.05rem;
  }
  .items-overflow-badge:hover {
    background: rgba(0, 122, 255, 0.18);
  }

  .workload-bottom-bar {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    height: 2.5px;
    background: transparent;
  }

  .workload-fill {
    height: 100%;
    background: var(--accent-primary);
    opacity: 0.6;
  }
</style>
