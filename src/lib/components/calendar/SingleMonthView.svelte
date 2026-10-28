<script lang="ts">
  import { themeStore } from "../../stores/theme.svelte";
  import type { Course, Exam } from "../../types/domain";
  import {
    buildMonthGrid,
    getWeekdayHeaders,
    MESI_ITALIANI,
    type DayAggregatedData
  } from "../../utils/calendar";
  import DayCell from "./DayCell.svelte";

  interface Props {
    year: number;
    month: number; // 1..12
    byDate: Map<string, DayAggregatedData>;
    courseById: Map<string, Course>;
    selectedDate?: string | null;
    selectedCourseId?: string | null;
    todayIso: string;
    onSelectDay?: (isoDate: string) => void;
    onSelectExam?: (exam: Exam, e: MouseEvent) => void;
    onStepMonth?: (delta: -1 | 1) => void;
  }

  let {
    year,
    month,
    byDate,
    courseById,
    selectedDate = null,
    selectedCourseId = null,
    todayIso,
    onSelectDay,
    onSelectExam,
    onStepMonth
  }: Props = $props();

  let startDayOfWeek = $derived(themeStore.settings.startDayOfWeek ?? 1);
  let weeklyHolidayDay = $derived(themeStore.settings.weeklyHolidayDay ?? 0);

  let weekdayHeaders = $derived(getWeekdayHeaders(startDayOfWeek, weeklyHolidayDay));
  let cells = $derived(
    buildMonthGrid(year, month, startDayOfWeek, weeklyHolidayDay, todayIso)
  );
  let rowCount = $derived(Math.ceil(cells.length / 7));
  let monthTitle = $derived(`${MESI_ITALIANI[month - 1] ?? ""} ${year}`);
</script>

<div class="single-month-view" aria-label="Vista Mensile {monthTitle}">
  <header class="single-month-bar">
    <div class="bar-left">
      <h2 class="month-heading">{monthTitle}</h2>
    </div>
    <div class="bar-right">
      <button
        type="button"
        class="step-btn pressable"
        title="Mese precedente"
        onclick={() => onStepMonth?.(-1)}
      >
        ‹ Mese Prec.
      </button>
      <button
        type="button"
        class="step-btn pressable"
        title="Mese successivo"
        onclick={() => onStepMonth?.(1)}
      >
        Mese Succ. ›
      </button>
    </div>
  </header>

  <div class="month-grid-frame">
    <div class="weekday-row" role="row">
      {#each weekdayHeaders as wh (wh.jsDay)}
        <div
          class="weekday-cell"
          role="columnheader"
          style:color={wh.isWeeklyHoliday ? themeStore.settings.sundayColor : undefined}
        >
          {wh.longLabel}
        </div>
      {/each}
    </div>

    <div
      class="days-matrix"
      role="grid"
      style:grid-template-rows="repeat({rowCount}, minmax(0, 1fr))"
    >
      {#each cells as cell (cell.isoDate)}
        <DayCell
          {cell}
          dayData={byDate.get(cell.isoDate) ?? null}
          {courseById}
          isSelected={selectedDate === cell.isoDate}
          {selectedCourseId}
          {onSelectDay}
          {onSelectExam}
        />
      {/each}
    </div>
  </div>
</div>

<style>
  .single-month-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 0.85rem 1.15rem 1.15rem;
    overflow: hidden;
  }

  .single-month-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.75rem;
    flex-shrink: 0;
  }

  .month-heading {
    margin: 0;
    font-family: var(--font-display);
    font-size: 1.4rem;
    font-weight: 800;
    letter-spacing: var(--tracking-tight);
    color: var(--text-primary);
  }

  .bar-right {
    display: flex;
    gap: 0.45rem;
  }

  .step-btn {
    height: 1.95rem;
    padding: 0 0.75rem;
    border-radius: var(--radius-sm);
    border: 0.5px solid var(--hairline-border);
    background: var(--surface-pill);
    box-shadow: var(--shadow-pill);
    color: var(--text-primary);
    font-size: 0.74rem;
    font-weight: 600;
    letter-spacing: var(--tracking-normal);
    cursor: pointer;
    will-change: transform;
    transition: transform 120ms var(--spring-press), background-color 130ms ease;
  }

  .step-btn:hover {
    background: var(--surface-pill-hover);
    border-color: var(--hairline-border-strong);
  }

  .step-btn:active {
    transform: scale(0.96);
  }

  .month-grid-frame {
    flex: 1;
    display: flex;
    flex-direction: column;
    border-radius: var(--radius-lg);
    border-top: 0.5px solid var(--hairline-border);
    border-left: 0.5px solid var(--hairline-border);
    box-shadow: var(--shadow-pill);
    overflow: hidden;
  }

  .weekday-row {
    display: grid;
    grid-template-columns: repeat(7, minmax(0, 1fr));
    background: var(--calendar-header-bg);
    border-right: 0.5px solid var(--hairline-border);
    border-bottom: 0.5px solid var(--hairline-border);
    flex-shrink: 0;
  }

  .weekday-cell {
    padding: 0.5rem 0.75rem;
    font-size: 0.68rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: var(--tracking-caps);
    color: var(--text-tertiary);
  }

  .days-matrix {
    flex: 1;
    display: grid;
    grid-template-columns: repeat(7, minmax(0, 1fr));
    min-height: 0;
  }
</style>
