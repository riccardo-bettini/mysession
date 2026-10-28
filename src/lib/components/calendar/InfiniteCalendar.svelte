<script lang="ts">
  import {
    elementScroll,
    observeElementOffset,
    observeElementRect,
    Virtualizer,
    type VirtualItem
  } from "@tanstack/virtual-core";
  import { onMount } from "svelte";
  import { themeStore } from "../../stores/theme.svelte";
  import type { Course, Exam } from "../../types/domain";
  import {
    buildSessionVirtualMonthList,
    buildVirtualMonthList,
    parseIsoDate,
    type DayAggregatedData
  } from "../../utils/calendar";
  import MonthGridBlock from "./MonthGridBlock.svelte";

  interface Props {
    byDate: Map<string, DayAggregatedData>;
    courseById: Map<string, Course>;
    selectedDate?: string | null;
    selectedCourseId?: string | null;
    todayIso: string;
    anchorIso?: string;
    sessionStartDate?: string | null;
    sessionEndDate?: string | null;
    lockToSession?: boolean;
    scrollRequestKey?: number;
    scrollTargetIso?: string | null;
    onSelectDay?: (isoDate: string) => void;
    onSelectExam?: (exam: Exam, e: MouseEvent) => void;
    onVisibleMonthChange?: (year: number, month: number, title: string) => void;
  }

  let {
    byDate,
    courseById,
    selectedDate = null,
    selectedCourseId = null,
    todayIso,
    anchorIso = todayIso,
    sessionStartDate = null,
    sessionEndDate = null,
    lockToSession = false,
    scrollRequestKey = 0,
    scrollTargetIso = null,
    onSelectDay,
    onSelectExam,
    onVisibleMonthChange
  }: Props = $props();

  let scrollContainerEl = $state<HTMLDivElement | null>(null);
  let virtualItems = $state<VirtualItem[]>([]);
  let totalSize = $state<number>(0);

  let baseAnchor = $derived(parseIsoDate(anchorIso || todayIso));
  let startDayOfWeek = $derived(themeStore.settings.startDayOfWeek ?? 1);

  let monthsList = $derived.by(() => {
    if (lockToSession && sessionStartDate && sessionEndDate) {
      return buildSessionVirtualMonthList(sessionStartDate, sessionEndDate, startDayOfWeek);
    }
    return buildVirtualMonthList(baseAnchor.year, baseAnchor.month, 60, 120, startDayOfWeek);
  });

  let virtualizerInstance: Virtualizer<HTMLDivElement, HTMLElement> | null = null;

  function estimateMonthHeight(index: number): number {
    const item = monthsList[index];
    const rows = item ? item.rowCount : 5;
    const scale = (themeStore.settings.uiScale || 100) / 100;
    return Math.round((88 + rows * 106) * scale);
  }

  function syncVirtualizerState() {
    if (!virtualizerInstance) return;
    virtualItems = virtualizerInstance.getVirtualItems();
    totalSize = virtualizerInstance.getTotalSize();

    if (virtualItems.length > 0) {
      const topItem = virtualItems[Math.min(1, virtualItems.length - 1)];
      const m = monthsList[topItem.index];
      if (m) {
        onVisibleMonthChange?.(m.year, m.month, m.title);
      }
    }
  }

  export function scrollToIsoDate(isoDate: string) {
    if (!virtualizerInstance || monthsList.length === 0) return;
    const { year, month } = parseIsoDate(isoDate);
    const key = `${year}-${String(month).padStart(2, "0")}`;
    let idx = monthsList.findIndex((m) => m.key === key);
    if (idx < 0) {
      if (lockToSession && sessionStartDate && sessionEndDate) {
        idx = isoDate < sessionStartDate ? 0 : monthsList.length - 1;
      }
    }
    if (idx >= 0) {
      virtualizerInstance.scrollToIndex(idx, { align: "start" });
      syncVirtualizerState();
    }
  }

  $effect(() => {
    const _req = scrollRequestKey;
    const target = scrollTargetIso;
    if (_req && target && virtualizerInstance) {
      scrollToIsoDate(target);
    }
  });

  // Re-sincronizza il Virtualizer quando monthsList o lockToSession cambiano
  $effect(() => {
    const count = monthsList.length;
    if (virtualizerInstance && scrollContainerEl) {
      virtualizerInstance.setOptions({
        count,
        getScrollElement: () => scrollContainerEl,
        estimateSize: (index) => estimateMonthHeight(index),
        overscan: 2,
        observeElementRect,
        observeElementOffset,
        scrollToFn: elementScroll,
        onChange: () => {
          syncVirtualizerState();
        }
      });
      virtualizerInstance._willUpdate();
      syncVirtualizerState();
    }
  });

  onMount(() => {
    if (!scrollContainerEl) return;

    const initialAnchorKey = `${baseAnchor.year}-${String(baseAnchor.month).padStart(2, "0")}`;
    const initialIdx = Math.max(
      0,
      monthsList.findIndex((m) => m.key === initialAnchorKey)
    );

    virtualizerInstance = new Virtualizer<HTMLDivElement, HTMLElement>({
      count: monthsList.length,
      getScrollElement: () => scrollContainerEl,
      estimateSize: (index) => estimateMonthHeight(index),
      overscan: 2,
      observeElementRect,
      observeElementOffset,
      scrollToFn: elementScroll,
      onChange: () => {
        syncVirtualizerState();
      }
    });

    virtualizerInstance._didMount();
    virtualizerInstance._willUpdate();
    syncVirtualizerState();

    queueMicrotask(() => {
      virtualizerInstance?.scrollToIndex(initialIdx, { align: "start" });
      syncVirtualizerState();
    });

    return () => {
      virtualizerInstance?._didMount()();
      virtualizerInstance = null;
    };
  });

  function measureRowAction(node: HTMLElement, index: number) {
    node.setAttribute("data-index", String(index));
    virtualizerInstance?.measureElement(node);
    return {
      update(newIndex: number) {
        node.setAttribute("data-index", String(newIndex));
        virtualizerInstance?.measureElement(node);
      }
    };
  }
</script>

<div
  bind:this={scrollContainerEl}
  class="infinite-calendar-viewport"
  role="region"
  aria-label="Calendario a scorrimento continuo virtualizzato"
>
  <div class="virtual-scroll-runway" style:height="{totalSize}px">
    {#each virtualItems as vItem (vItem.key)}
      {@const monthItem = monthsList[vItem.index]}
      {#if monthItem}
        <div
          use:measureRowAction={vItem.index}
          class="virtual-month-slot"
          style:transform="translateY({vItem.start}px)"
        >
          <MonthGridBlock
            {monthItem}
            {byDate}
            {courseById}
            {selectedDate}
            {selectedCourseId}
            {todayIso}
            {onSelectDay}
            {onSelectExam}
          />
        </div>
      {/if}
    {/each}
  </div>
</div>

<style>
  .infinite-calendar-viewport {
    position: relative;
    flex: 1;
    width: 100%;
    height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
    contain: strict;
  }

  .virtual-scroll-runway {
    position: relative;
    width: 100%;
  }

  .virtual-month-slot {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    will-change: transform;
  }
</style>
