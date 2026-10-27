<script lang="ts">
  import type { ConflictItem } from "../../types/domain";
  import { formatItalianDateLong } from "../../utils/calendar";

  interface Props {
    conflicts: ConflictItem[];
    onSelectDate?: (isoDate: string) => void;
    onClose?: () => void;
  }

  let {
    conflicts = [],
    onSelectDate,
    onClose
  }: Props = $props();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="modal-scrim"
  role="dialog"
  aria-modal="true"
  aria-label="Analisi Conflitti & Buffer CFU"
  tabindex="-1"
  onclick={(e) => {
    if (e.target === e.currentTarget) onClose?.();
  }}
>
  <div class="modal-sheet">
    <header class="sheet-header">
      <div>
        <span class="sheet-eyebrow">CONFLICT ENGINE IN TEMPO REALE (RUST)</span>
        <h2>Analisi Collisioni & Buffer CFU ({conflicts.length})</h2>
      </div>
      <button type="button" class="sheet-close-btn" onclick={() => onClose?.()}>✕</button>
    </header>

    <div class="sheet-body">
      {#if conflicts.length === 0}
        <div class="no-conflicts-state">
          <strong>✓ Nessuna collisione rilevata</strong>
          <p>Tutti gli appelli pianificati rispettano il buffer minimo di giorni calcolato sui CFU e sulle unità rimanenti.</p>
        </div>
      {:else}
        <div class="conflicts-list">
          {#each conflicts as cf (cf.id)}
            <div class="conflict-card" class:critical={cf.severity === "critical"}>
              <div class="cf-top">
                <span class="cf-severity-badge">
                  {cf.severity === "critical" ? "CRITICO · SOVRAPPOSIZIONE DIRETTA" : "AVVISO · BUFFER CFU INSUFFICIENTE"}
                </span>
                <span class="cf-siglas">{cf.courseSiglas.join(" ↔ ")}</span>
              </div>

              <h3 class="cf-title">{cf.title}</h3>
              <p class="cf-desc">{cf.description}</p>

              <div class="cf-metrics-row">
                <span>Distanza attuale: <strong>{cf.daysGap} gg</strong></span>
                <span>Buffer minimo consigliato: <strong>{cf.recommendedMinDays} gg</strong></span>
              </div>

              <div class="cf-suggestion">💡 {cf.suggestion}</div>

              <div class="cf-dates-actions">
                <button
                  type="button"
                  class="jump-btn"
                  onclick={() => {
                    onSelectDate?.(cf.date);
                    onClose?.();
                  }}
                >
                  Salta a {formatItalianDateLong(cf.date)}
                </button>
                {#if cf.secondaryDate && cf.secondaryDate !== cf.date}
                  <button
                    type="button"
                    class="jump-btn"
                    onclick={() => {
                      if (cf.secondaryDate) onSelectDate?.(cf.secondaryDate);
                      onClose?.();
                    }}
                  >
                    Salta a {formatItalianDateLong(cf.secondaryDate)}
                  </button>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .modal-scrim {
    position: fixed;
    inset: 0;
    z-index: 420;
    background: rgba(0, 0, 0, 0.45);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1.5rem;
  }

  .modal-sheet {
    width: 100%;
    max-width: 38rem;
    max-height: 85vh;
    border-radius: var(--radius-xl);
    background: var(--popover-bg);
    backdrop-filter: blur(28px) saturate(195%);
    -webkit-backdrop-filter: blur(28px) saturate(195%);
    border: 0.5px solid var(--hairline-border-strong);
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.45);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .sheet-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    padding: 1.1rem 1.35rem 0.85rem;
    border-bottom: 0.5px solid var(--hairline-border);
  }

  .sheet-eyebrow {
    font-size: 0.62rem;
    font-weight: 700;
    letter-spacing: 0.06em;
    color: var(--text-tertiary);
  }

  .sheet-header h2 {
    margin: 0.15rem 0 0;
    font-size: 1.12rem;
    font-weight: 800;
    letter-spacing: -0.022em;
  }

  .sheet-close-btn {
    width: 1.75rem;
    height: 1.75rem;
    border-radius: 999px;
    border: 0.5px solid var(--hairline-border);
    background: var(--surface-pill);
    color: var(--text-secondary);
    cursor: pointer;
  }

  .sheet-body {
    flex: 1;
    overflow-y: auto;
    padding: 1rem 1.35rem 1.25rem;
  }

  .no-conflicts-state {
    padding: 1.5rem;
    border-radius: var(--radius-md);
    background: rgba(52, 199, 89, 0.12);
    border: 0.5px solid rgba(52, 199, 89, 0.35);
    text-align: center;
  }

  .conflicts-list {
    display: flex;
    flex-direction: column;
    gap: 0.7rem;
  }

  .conflict-card {
    padding: 0.85rem 1rem;
    border-radius: var(--radius-md);
    background: rgba(255, 159, 10, 0.12);
    border: 0.5px solid rgba(255, 159, 10, 0.4);
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .conflict-card.critical {
    background: rgba(255, 69, 58, 0.14);
    border-color: rgba(255, 69, 58, 0.45);
  }

  .cf-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 0.64rem;
    font-weight: 800;
  }

  .cf-severity-badge {
    color: var(--conflict-warning);
  }

  .conflict-card.critical .cf-severity-badge {
    color: var(--conflict-critical);
  }

  .cf-siglas {
    padding: 0.1rem 0.45rem;
    border-radius: var(--radius-pill);
    background: var(--surface-pill);
  }

  .cf-title {
    margin: 0;
    font-size: 0.92rem;
    font-weight: 700;
  }

  .cf-desc {
    margin: 0;
    font-size: 0.76rem;
    color: var(--text-secondary);
  }

  .cf-metrics-row {
    display: flex;
    gap: 1rem;
    font-size: 0.72rem;
    color: var(--text-secondary);
  }

  .cf-suggestion {
    font-size: 0.72rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .cf-dates-actions {
    display: flex;
    gap: 0.45rem;
    flex-wrap: wrap;
    margin-top: 0.25rem;
  }

  .jump-btn {
    padding: 0.28rem 0.65rem;
    border-radius: var(--radius-xs);
    border: 0.5px solid var(--hairline-border-strong);
    background: var(--surface-card);
    color: var(--text-primary);
    font-size: 0.7rem;
    font-weight: 600;
    cursor: pointer;
  }
</style>
