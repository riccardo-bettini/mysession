<script lang="ts">
  import { themeStore } from "../../stores/theme.svelte";
  import type { Course, Exam } from "../../types/domain";
  import {
    buildMonthGrid,
    getWeekdayHeaders,
    type DayAggregatedData,
    type VirtualMonthItem
  } from "../../utils/calendar";
  import DayCell from "./DayCell.svelte";

  interface Props {
    monthItem: VirtualMonthItem;
    byDate: Map<string, DayAggregatedData>;
    courseById: Map<string, Course>;
    selectedDate?: string | null;
    selectedCourseId?: string | null;
    todayIso: string;
    onSelectDay?: (isoDate: string) => void;
    onSelectExam?: (exam: Exam, e: MouseEvent) => void;
  }

  let {
    monthItem,
    byDate,
    courseById,
    selectedDate = null,
    selectedCourseId = null,
    todayIso,
    onSelectDay,
    onSelectExam
  }: Props = $props();

  let startDayOfWeek = $derived(themeStore.settings.startDayOfWeek ?? 1);
  let weeklyHolidayDay = $derived(themeStore.settings.weeklyHolidayDay ?? 0);

  let weekdayHeaders = $derived(getWeekdayHeaders(startDayOfWeek, weeklyHolidayDay));
  let cells = $derived(
    buildMonthGrid(
      monthItem.year,
      monthItem.month,
      startDayOfWeek,
      weeklyHolidayDay,
      todayIso
    )
  );

  let monthStats = $derived.by(() => {
    let examsCount = 0;
    let conflictsCount = 0;
    for (const c of cells) {
      if (!c.isCurrentMonth) continue;
      const d = byDate.get(c.isoDate);
      if (!d) continue;
      examsCount += selectedCourseId
        ? d.exams.filter((e) => e.courseId === selectedCourseId).length
        : d.exams.length;
      conflictsCount += d.conflicts.length;
    }
    return { examsCount, conflictsCount };
  });
</script>

<section class="month-grid-block" aria-label={monthItem.title}>
  <header class="month-block-header">
    <div class="month-title-group">
      <h3 class="month-title">{monthItem.title}</h3>
      {#if monthStats.examsCount > 0}
        <span class="month-exam-badge">
          {monthStats.examsCount} {monthStats.examsCount === 1 ? "appello" : "appelli"}
        </span>
      {/if}
      {#if monthStats.conflictsCount > 0}
        <span class="month-conflict-badge">
          {monthStats.conflictsCount} {monthStats.conflictsCount === 1 ? "collisione" : "collisioni"}
        </span>
      {/if}
    </div>
  </header>

  <div class="weekday-header-row" role="row">
    {#each weekdayHeaders as wh (wh.jsDay)}
      <div
        class="weekday-col-label"
        role="columnheader"
        style:color={wh.isWeeklyHoliday ? themeStore.settings.sundayColor : undefined}
      >
        {wh.shortLabel}
      </div>
    {/each}
  </div>

  <div class="month-cells-grid" role="grid">
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
</section>

<style>
  .month-grid-block {
    display: flex;
    flex-direction: column;
    background: transparent;
    padding: 0 1rem 1.35rem;
  }

  .month-block-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.85rem 0.25rem 0.55rem;
  }

  .month-title-group {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .month-title {
    margin: 0;
    font-family: var(--font-display);
    font-size: 1.2rem;
    font-weight: 800;
    letter-spacing: var(--tracking-tight);
    color: var(--text-primary);
  }

  .month-exam-badge {
    font-size: 0.66rem;
    font-weight: 700;
    padding: 0.14rem 0.52rem;
    border-radius: var(--radius-pill);
    background: var(--accent-soft);
    border: 0.5px solid var(--hairline-border);
    color: var(--accent-primary);
    letter-spacing: var(--tracking-normal);
  }

  .month-conflict-badge {
    font-size: 0.66rem;
    font-weight: 700;
    padding: 0.14rem 0.52rem;
    border-radius: var(--radius-pill);
    background: rgba(255, 69, 58, 0.16);
    border: 0.5px solid rgba(255, 69, 58, 0.35);
    color: var(--conflict-critical);
    letter-spacing: var(--tracking-normal);
  }

  .weekday-header-row {
    display: grid;
    grid-template-columns: repeat(7, minmax(0, 1fr));
    background: var(--calendar-header-bg);
    border-top: 0.5px solid var(--hairline-border);
    border-left: 0.5px solid var(--hairline-border);
    border-right: 0.5px solid var(--hairline-border);
    border-top-left-radius: var(--radius-lg);
    border-top-right-radius: var(--radius-lg);
  }

  .weekday-col-label {
    padding: 0.45rem 0.6rem;
    font-size: 0.66rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: var(--tracking-caps);
    color: var(--text-tertiary);
    text-align: left;
  }

  .month-cells-grid {
    display: grid;
    grid-template-columns: repeat(7, minmax(0, 1fr));
    border-top: 0.5px solid var(--hairline-border);
    border-left: 0.5px solid var(--hairline-border);
    border-bottom-left-radius: var(--radius-lg);
    border-bottom-right-radius: var(--radius-lg);
    box-shadow: var(--shadow-pill);
    overflow: hidden;
  }
</style>
