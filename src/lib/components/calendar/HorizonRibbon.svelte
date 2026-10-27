<script lang="ts">
  import { themeStore } from "../../stores/theme.svelte";
  import type { TelemetrySummary } from "../../types/domain";
  import { formatExamRoundLabel, formatItalianDateShort } from "../../utils/calendar";
  import { generateCourseColorTokens } from "../../utils/color";

  interface Props {
    telemetry: TelemetrySummary | null;
    selectedCourseId?: string | null;
    onSelectDate?: (isoDate: string) => void;
  }

  let {
    telemetry = null,
    selectedCourseId = null,
    onSelectDate
  }: Props = $props();

  let isDark = $derived(themeStore.effectiveTheme === "dark");
  let pins = $derived.by(() => {
    const raw = telemetry?.horizonPins ?? [];
    if (!selectedCourseId) return raw;
    return raw.filter((p) => p.courseId === selectedCourseId);
  });
  let ticks = $derived(telemetry?.horizonTicks ?? []);
  let todayPct = $derived(telemetry?.horizonTodayPct ?? null);
</script>

<section class="horizon-ribbon-bar" aria-label="Orizzonte temporale degli appelli di sessione">
  <div class="ribbon-header-col">
    <span class="ribbon-title">ORIZZONTE SESSIONE</span>
    {#if telemetry?.horizonStartDate && telemetry?.horizonEndDate}
      <span class="ribbon-range">
        {formatItalianDateShort(telemetry.horizonStartDate)} → {formatItalianDateShort(telemetry.horizonEndDate)}
      </span>
    {/if}
  </div>

  <div class="ribbon-track-container">
    <div class="ribbon-axis-line"></div>

    <!-- Tacche Mensili -->
    {#each ticks as tick (`${tick.year}-${tick.month}`)}
      <div class="ribbon-month-tick" style:left="{tick.positionPct}%">
        <span class="tick-notch"></span>
        <span class="tick-label">{tick.label}</span>
      </div>
    {/each}

    <!-- Indicatore Data Odierna -->
    {#if todayPct !== null && todayPct >= 0 && todayPct <= 100}
      <button
        type="button"
        class="ribbon-today-marker pressable"
        style:left="{todayPct}%"
        title="Oggi ({telemetry?.referenceToday})"
        onclick={() => {
          if (telemetry?.referenceToday) onSelectDate?.(telemetry.referenceToday);
        }}
      >
        <span class="today-dot"></span>
        <span class="today-tag">Oggi</span>
      </button>
    {/if}

    <!-- Pin Appelli d'Esame -->
    {#each pins as pin (pin.examId)}
      {@const palette = generateCourseColorTokens(pin.colorHex, isDark)}
      <button
        type="button"
        class="ribbon-exam-pin pressable"
        class:conflict={pin.hasConflict}
        style:left="{pin.positionPct}%"
        style:--pin-bg={palette.solid}
        style:--pin-fg={palette.badgeText}
        title="{pin.sigla} · {formatExamRoundLabel(pin.round)} ({formatItalianDateShort(pin.date)}){pin.hasConflict ? ' — CONFLITTO RILEVATO' : ''}"
        onclick={() => onSelectDate?.(pin.date)}
      >
        <span class="pin-pill">{pin.sigla}</span>
        {#if pin.hasConflict}
          <span class="pin-conflict-dot" aria-label="Conflitto"></span>
        {/if}
      </button>
    {/each}
  </div>
</section>

<style>
  .horizon-ribbon-bar {
    display: flex;
    align-items: center;
    gap: 1rem;
    height: 3.1rem;
    padding: 0 1rem;
    background: var(--calendar-header-bg);
    backdrop-filter: blur(20px) saturate(180%);
    -webkit-backdrop-filter: blur(20px) saturate(180%);
    border-bottom: 0.5px solid var(--hairline-border);
    flex-shrink: 0;
    user-select: none;
  }

  .ribbon-header-col {
    display: flex;
    flex-direction: column;
    gap: 0.08rem;
    min-width: 8.5rem;
    flex-shrink: 0;
  }

  .ribbon-title {
    font-size: 0.62rem;
    font-weight: 700;
    letter-spacing: 0.06em;
    color: var(--text-tertiary);
  }

  .ribbon-range {
    font-size: 0.72rem;
    font-weight: 600;
    color: var(--text-secondary);
    font-variant-numeric: tabular-nums;
  }

  .ribbon-track-container {
    position: relative;
    flex: 1;
    height: 2.3rem;
    display: flex;
    align-items: center;
    margin: 0 0.75rem;
  }

  .ribbon-axis-line {
    position: absolute;
    left: 0;
    right: 0;
    top: 50%;
    height: 1.5px;
    background: var(--hairline-border-strong);
    border-radius: 999px;
    transform: translateY(-50%);
  }

  .ribbon-month-tick {
    position: absolute;
    top: 50%;
    transform: translate(-50%, -50%);
    display: flex;
    flex-direction: column;
    align-items: center;
    pointer-events: none;
  }

  .tick-notch {
    width: 1px;
    height: 8px;
    background: var(--hairline-border-strong);
  }

  .tick-label {
    margin-top: 0.25rem;
    font-size: 0.6rem;
    font-weight: 600;
    color: var(--text-tertiary);
    white-space: nowrap;
  }

  .ribbon-today-marker {
    position: absolute;
    top: 50%;
    transform: translate(-50%, -50%);
    border: none;
    background: transparent;
    padding: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    cursor: pointer;
    z-index: 2;
  }

  .today-dot {
    width: 8px;
    height: 8px;
    border-radius: 999px;
    background: var(--accent-primary);
    box-shadow: 0 0 8px var(--accent-glow);
  }

  .today-tag {
    position: absolute;
    bottom: calc(100% + 2px);
    font-size: 0.58rem;
    font-weight: 700;
    color: var(--accent-primary);
    white-space: nowrap;
  }

  .ribbon-exam-pin {
    position: absolute;
    top: 50%;
    transform: translate(-50%, -50%);
    border: none;
    background: transparent;
    padding: 0;
    cursor: pointer;
    z-index: 3;
    transition: transform 140ms var(--ease-out);
  }

  .ribbon-exam-pin:hover {
    transform: translate(-50%, -58%) scale(1.08);
    z-index: 10;
  }

  .ribbon-exam-pin:active {
    transform: translate(-50%, -50%) scale(0.95);
  }

  .pin-pill {
    display: inline-block;
    padding: 0.14rem 0.42rem;
    border-radius: var(--radius-pill);
    background: var(--pin-bg);
    color: var(--pin-fg);
    font-size: 0.64rem;
    font-weight: 700;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.25);
    white-space: nowrap;
  }

  .ribbon-exam-pin.conflict .pin-pill {
    outline: 1.5px solid var(--conflict-critical);
  }

  .pin-conflict-dot {
    position: absolute;
    top: -3px;
    right: -3px;
    width: 7px;
    height: 7px;
    border-radius: 999px;
    background: var(--conflict-critical);
    border: 1px solid #fff;
  }
</style>
