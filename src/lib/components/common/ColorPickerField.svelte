<script lang="ts">
  import { themeStore } from "../../stores/theme.svelte";
  import {
    generateCourseColorTokens,
    hexToHsl,
    hueToHex,
    normalizeHex,
    UNIFIED_STANDARD_PALETTE
  } from "../../utils/color";

  interface Props {
    value: string;
    label?: string;
    previewText?: string;
    onChange?: (newHex: string) => void;
  }

  let {
    value = $bindable("#007AFF"),
    label = "Colore Identificativo",
    previewText = "ANTEPRIMA",
    onChange
  }: Props = $props();

  let isDark = $derived(themeStore.effectiveTheme === "dark");
  let normalized = $derived(normalizeHex(value));
  let hsl = $derived(hexToHsl(normalized));
  let tokens = $derived(generateCourseColorTokens(normalized, isDark));

  function selectColor(hex: string) {
    const clean = normalizeHex(hex);
    value = clean;
    onChange?.(clean);
  }

  function handleHueInput(e: Event) {
    const hue = parseInt((e.currentTarget as HTMLInputElement).value, 10);
    if (!Number.isNaN(hue)) {
      const nextHex = hueToHex(hue, 82, 52);
      selectColor(nextHex);
    }
  }

  function handleHexInput(e: Event) {
    const raw = (e.currentTarget as HTMLInputElement).value;
    if (/^#?[0-9a-fA-F]{6}$/.test(raw.trim())) {
      selectColor(raw);
    }
  }
</script>

<div class="unified-color-picker">
  <div class="picker-top-row">
    <span class="picker-label">{label}</span>
    <div class="wcag-preview-group">
      <span
        class="preview-pill"
        style:background={tokens.bgTint}
        style:border-color={tokens.borderTint}
        style:color={tokens.textContrast}
      >
        <span class="preview-dot" style:background={tokens.solid}></span>
        {previewText}
      </span>
      <span class="wcag-ratio" title="Rapporto di contrasto WCAG 2.1">
        WCAG {tokens.contrastRatio}:1
      </span>
    </div>
  </div>

  <!-- 12 Colori Standard Unificati -->
  <div class="palette-swatches" role="radiogroup" aria-label={label}>
    {#each UNIFIED_STANDARD_PALETTE as swatch (swatch)}
      <button
        type="button"
        class="swatch-btn pressable"
        class:active={normalized.toUpperCase() === swatch.toUpperCase()}
        style:background={swatch}
        title={swatch}
        aria-label="Seleziona colore {swatch}"
        onclick={() => selectColor(swatch)}
      >
        {#if normalized.toUpperCase() === swatch.toUpperCase()}
          <svg viewBox="0 0 12 12" fill="none">
            <path d="M2.5 6.2L5 8.7L9.5 3.5" stroke="#fff" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        {/if}
      </button>
    {/each}
  </div>

  <!-- Slider Tonalità 360° + Input Hex Diretto -->
  <div class="hue-and-hex-row">
    <input
      type="range"
      class="hue-slider-360"
      min="0"
      max="360"
      step="1"
      value={hsl.h}
      aria-label="Tonalità 360 gradi"
      oninput={handleHueInput}
    />
    <input
      type="text"
      class="hex-input"
      maxlength="7"
      value={normalized}
      aria-label="Codice colore esadecimale"
      oninput={handleHexInput}
    />
  </div>
</div>

<style>
  .unified-color-picker {
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
    padding: 0.65rem 0.75rem;
    border-radius: var(--radius-sm);
    background: var(--surface-card);
    border: 0.5px solid var(--hairline-border);
  }

  .picker-top-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .picker-label {
    font-size: 0.72rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .wcag-preview-group {
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
  }

  .preview-pill {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.14rem 0.5rem;
    border-radius: var(--radius-pill);
    border: 0.5px solid transparent;
    font-size: 0.65rem;
    font-weight: 800;
  }

  .preview-dot {
    width: 6px;
    height: 6px;
    border-radius: 999px;
  }

  .wcag-ratio {
    font-size: 0.62rem;
    font-weight: 600;
    color: var(--text-tertiary);
    font-variant-numeric: tabular-nums;
  }

  .palette-swatches {
    display: grid;
    grid-template-columns: repeat(12, 1fr);
    gap: 0.32rem;
  }

  .swatch-btn {
    aspect-ratio: 1;
    width: 100%;
    border-radius: 999px;
    border: 1.5px solid rgba(255, 255, 255, 0.18);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    padding: 0;
    transition: transform 140ms cubic-bezier(0.34, 1.56, 0.64, 1), box-shadow 140ms ease;
  }

  .swatch-btn:hover {
    transform: scale(1.12);
  }

  .swatch-btn:active {
    transform: scale(0.92);
  }

  .swatch-btn.active {
    border-color: var(--text-primary);
    box-shadow: 0 0 0 2px var(--surface-card), 0 0 0 4px var(--accent-primary);
  }

  .swatch-btn svg {
    width: 0.7rem;
    height: 0.7rem;
  }

  .hue-and-hex-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .hue-slider-360 {
    flex: 1;
    height: 8px;
    appearance: none;
    -webkit-appearance: none;
    border-radius: 999px;
    background: linear-gradient(
      to right,
      hsl(0, 82%, 52%),
      hsl(60, 82%, 52%),
      hsl(120, 82%, 52%),
      hsl(180, 82%, 52%),
      hsl(240, 82%, 52%),
      hsl(300, 82%, 52%),
      hsl(360, 82%, 52%)
    );
    outline: none;
    cursor: pointer;
  }

  .hex-input {
    width: 5.2rem;
    height: 1.75rem;
    padding: 0 0.45rem;
    border-radius: var(--radius-xs);
    border: 0.5px solid var(--hairline-border-strong);
    background: var(--surface-pill);
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 0.72rem;
    font-weight: 600;
    text-align: center;
    transition: border-color 140ms ease, box-shadow 140ms ease;
  }

  .hex-input:focus-visible {
    outline: none;
    border-color: var(--accent-primary);
    box-shadow: 0 0 0 2.5px var(--accent-soft);
  }
</style>
