<script lang="ts">
  import type { ConflictItem } from "../../types/domain";
  import { formatItalianDateLong } from "../../utils/calendar";

  interface Props {
    conflicts?: ConflictItem[];
    ignoredConflicts?: ConflictItem[];
    onIgnoreConflict?: (id: string) => void;
    onRestoreConflict?: (id: string) => void;
    onIgnoreAll?: (ids: string[]) => void;
    onRestoreAll?: () => void;
    onSelectDate?: (isoDate: string) => void;
    onClose?: () => void;
  }

  let {
    conflicts = [],
    ignoredConflicts = [],
    onIgnoreConflict,
    onRestoreConflict,
    onIgnoreAll,
    onRestoreAll,
    onSelectDate,
    onClose
  }: Props = $props();

  // Tab selection with smart fallback
  let userSelectedTab = $state<"active" | "ignored" | null>(null);
  let activeTab = $derived<"active" | "ignored">(
    userSelectedTab ?? (conflicts.length === 0 && ignoredConflicts.length > 0 ? "ignored" : "active")
  );
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
        <h2>Analisi Collisioni & Buffer CFU</h2>
      </div>
      <button
        type="button"
        class="sheet-close-btn pressable"
        aria-label="Chiudi finestra conflitti"
        onclick={() => onClose?.()}
      >
        ✕
      </button>
    </header>

    <!-- Segmented Tab Bar + Batch Action Header -->
    <div class="sheet-subnav">
      <div class="segmented-control" role="tablist">
        <button
          type="button"
          class="seg-btn pressable"
          class:active={activeTab === "active"}
          onclick={() => (userSelectedTab = "active")}
          role="tab"
          aria-selected={activeTab === "active"}
        >
          <span>Attivi</span>
          <span class="badge" class:has-items={conflicts.length > 0}>{conflicts.length}</span>
        </button>

        <button
          type="button"
          class="seg-btn pressable"
          class:active={activeTab === "ignored"}
          onclick={() => (userSelectedTab = "ignored")}
          role="tab"
          aria-selected={activeTab === "ignored"}
        >
          <span>Ignorati</span>
          <span class="badge" class:has-items={ignoredConflicts.length > 0}>{ignoredConflicts.length}</span>
        </button>
      </div>

      <div class="subnav-actions">
        {#if activeTab === "active" && conflicts.length > 0}
          <button
            type="button"
            class="batch-action-btn pressable"
            title="Ignora tutti i conflitti attivi rilevati"
            onclick={() => onIgnoreAll?.(conflicts.map((c) => c.id))}
          >
            <span class="btn-icon">🙈</span>
            <span>Ignora tutti</span>
          </button>
        {:else if activeTab === "ignored" && ignoredConflicts.length > 0}
          <button
            type="button"
            class="batch-action-btn pressable restore-all"
            title="Ripristina tutti i conflitti ignorati tra gli allarmi attivi"
            onclick={() => onRestoreAll?.()}
          >
            <span class="btn-icon">↩</span>
            <span>Ripristina tutti</span>
          </button>
        {/if}
      </div>
    </div>

    <div class="sheet-body">
      {#if activeTab === "active"}
        {#if conflicts.length === 0}
          <div class="no-conflicts-state">
            <span class="state-icon">✓</span>
            <strong>Nessuna collisione attiva rilevata</strong>
            {#if ignoredConflicts.length > 0}
              <p>
                Hai <strong>{ignoredConflicts.length}</strong> {ignoredConflicts.length === 1 ? "conflitto ignorato" : "conflitti ignorati"} manualmente. Puoi visualizzarli ed eventualmente ripristinarli dalla scheda <em>Ignorati</em>.
              </p>
            {:else}
              <p>Tutti gli appelli pianificati rispettano il buffer minimo di giorni calcolato sui CFU e sulle unità rimanenti.</p>
            {/if}
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
                    class="jump-btn pressable"
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
                      class="jump-btn pressable"
                      onclick={() => {
                        if (cf.secondaryDate) onSelectDate?.(cf.secondaryDate);
                        onClose?.();
                      }}
                    >
                      Salta a {formatItalianDateLong(cf.secondaryDate)}
                    </button>
                  {/if}

                  <button
                    type="button"
                    class="action-btn ignore-btn pressable"
                    title="Ignora questo conflitto e nascondi gli allarmi sul calendario"
                    onclick={() => onIgnoreConflict?.(cf.id)}
                  >
                    <span class="btn-icon">🙈</span>
                    <span>Ignora conflitto</span>
                  </button>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      {:else}
        {#if ignoredConflicts.length === 0}
          <div class="no-conflicts-state muted-state">
            <span class="state-icon">ℹ</span>
            <strong>Nessun conflitto ignorato</strong>
            <p>I conflitti che decidi di silenziare compariranno in questa sezione e potranno essere ripristinati in qualsiasi momento.</p>
          </div>
        {:else}
          <div class="conflicts-list">
            {#each ignoredConflicts as cf (cf.id)}
              <div class="conflict-card ignored-mode">
                <div class="cf-top">
                  <span class="cf-severity-badge ignored">
                    IGNORATO · SILENZIATO DAL CALENDARIO
                  </span>
                  <span class="cf-siglas">{cf.courseSiglas.join(" ↔ ")}</span>
                </div>

                <h3 class="cf-title">{cf.title}</h3>
                <p class="cf-desc">{cf.description}</p>

                <div class="cf-metrics-row">
                  <span>Distanza rilevata: <strong>{cf.daysGap} gg</strong></span>
                  <span>Buffer calcolato: <strong>{cf.recommendedMinDays} gg</strong></span>
                </div>

                <div class="cf-dates-actions">
                  <button
                    type="button"
                    class="jump-btn pressable"
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
                      class="jump-btn pressable"
                      onclick={() => {
                        if (cf.secondaryDate) onSelectDate?.(cf.secondaryDate);
                        onClose?.();
                      }}
                    >
                      Salta a {formatItalianDateLong(cf.secondaryDate)}
                    </button>
                  {/if}

                  <button
                    type="button"
                    class="action-btn restore-btn pressable"
                    title="Ripristina questo conflitto tra gli allarmi attivi"
                    onclick={() => onRestoreConflict?.(cf.id)}
                  >
                    <span class="btn-icon">↩</span>
                    <span>Ripristina conflitto</span>
                  </button>
                </div>
              </div>
            {/each}
          </div>
        {/if}
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
    backdrop-filter: blur(16px) saturate(170%);
    -webkit-backdrop-filter: blur(16px) saturate(170%);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1.5rem;
    animation: backdropFade 160ms cubic-bezier(0.16, 1, 0.3, 1);
  }

  .modal-sheet {
    width: 100%;
    max-width: 40rem;
    max-height: 85vh;
    border-radius: var(--radius-xl);
    background: var(--popover-bg);
    backdrop-filter: blur(28px) saturate(195%);
    -webkit-backdrop-filter: blur(28px) saturate(195%);
    border: 0.5px solid var(--hairline-border-strong);
    box-shadow: var(--shadow-modal);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: modalSheetEnter 260ms cubic-bezier(0.16, 1, 0.3, 1);
  }

  .sheet-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    padding: 1.1rem 1.35rem 0.75rem;
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
    font-size: 1.15rem;
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
    display: inline-flex;
    align-items: center;
    justify-content: center;
    transition: transform 120ms cubic-bezier(0.16, 1, 0.3, 1), background 120ms ease, color 120ms ease;
  }

  .sheet-close-btn:hover {
    background: var(--surface-card);
    color: var(--text-primary);
  }

  .sheet-close-btn:active {
    transform: scale(0.92);
  }

  /* Segmented Subnav Bar */
  .sheet-subnav {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.65rem 1.35rem;
    background: rgba(0, 0, 0, 0.08);
    border-bottom: 0.5px solid var(--hairline-border);
    gap: 0.75rem;
  }

  .segmented-control {
    display: inline-flex;
    padding: 2.5px;
    border-radius: var(--radius-sm);
    background: var(--surface-card);
    border: 0.5px solid var(--hairline-border);
    gap: 2px;
  }

  .seg-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
    padding: 0.28rem 0.75rem;
    border-radius: calc(var(--radius-sm) - 2px);
    border: none;
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.76rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 140ms ease, color 140ms ease, transform 120ms cubic-bezier(0.16, 1, 0.3, 1);
  }

  .seg-btn:hover {
    color: var(--text-primary);
  }

  .seg-btn.active {
    background: var(--surface-pill);
    color: var(--text-primary);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.15);
  }

  .badge {
    padding: 0.05rem 0.38rem;
    border-radius: var(--radius-pill);
    background: rgba(142, 142, 147, 0.2);
    font-size: 0.66rem;
    font-weight: 700;
    color: var(--text-secondary);
  }

  .badge.has-items {
    background: var(--accent-color, #007aff);
    color: #ffffff;
  }

  .seg-btn.active .badge.has-items {
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  }

  .batch-action-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.3rem 0.7rem;
    border-radius: var(--radius-xs);
    border: 0.5px solid var(--hairline-border);
    background: var(--surface-card);
    color: var(--text-secondary);
    font-size: 0.72rem;
    font-weight: 600;
    cursor: pointer;
    transition: transform 120ms cubic-bezier(0.16, 1, 0.3, 1), background 120ms ease, color 120ms ease;
  }

  .batch-action-btn:hover {
    background: var(--surface-pill);
    color: var(--text-primary);
  }

  .batch-action-btn.restore-all:hover {
    color: var(--accent-color, #007aff);
    border-color: rgba(0, 122, 255, 0.4);
  }

  .batch-action-btn:active {
    transform: scale(0.96);
  }

  .btn-icon {
    font-size: 0.8rem;
    line-height: 1;
  }

  .sheet-body {
    flex: 1;
    overflow-y: auto;
    padding: 1rem 1.35rem 1.25rem;
  }

  .no-conflicts-state {
    padding: 2rem 1.5rem;
    border-radius: var(--radius-md);
    background: rgba(52, 199, 89, 0.12);
    border: 0.5px solid rgba(52, 199, 89, 0.35);
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.35rem;
  }

  .no-conflicts-state.muted-state {
    background: rgba(142, 142, 147, 0.1);
    border-color: var(--hairline-border);
  }

  .state-icon {
    font-size: 1.4rem;
    line-height: 1;
    margin-bottom: 0.2rem;
  }

  .no-conflicts-state strong {
    font-size: 0.94rem;
  }

  .no-conflicts-state p {
    margin: 0;
    font-size: 0.78rem;
    color: var(--text-secondary);
    max-width: 28rem;
  }

  .conflicts-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .conflict-card {
    padding: 0.9rem 1.05rem;
    border-radius: var(--radius-md);
    background: rgba(255, 159, 10, 0.12);
    border: 0.5px solid rgba(255, 159, 10, 0.4);
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    transition: transform 160ms cubic-bezier(0.16, 1, 0.3, 1), box-shadow 160ms ease;
  }

  .conflict-card.critical {
    background: rgba(255, 69, 58, 0.14);
    border-color: rgba(255, 69, 58, 0.45);
  }

  .conflict-card.ignored-mode {
    background: rgba(142, 142, 147, 0.08);
    border-color: var(--hairline-border);
    opacity: 0.9;
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

  .cf-severity-badge.ignored {
    color: var(--text-tertiary);
  }

  .cf-siglas {
    padding: 0.1rem 0.45rem;
    border-radius: var(--radius-pill);
    background: var(--surface-pill);
    font-size: 0.65rem;
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
    gap: 1.25rem;
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
    align-items: center;
    flex-wrap: wrap;
    margin-top: 0.35rem;
    padding-top: 0.35rem;
    border-top: 0.5px solid var(--hairline-border);
  }

  .jump-btn,
  .action-btn {
    padding: 0.3rem 0.68rem;
    border-radius: var(--radius-xs);
    border: 0.5px solid var(--hairline-border-strong);
    background: var(--surface-card);
    color: var(--text-primary);
    font-size: 0.7rem;
    font-weight: 600;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    transition: transform 120ms cubic-bezier(0.16, 1, 0.3, 1), background 120ms ease, color 120ms ease, border-color 120ms ease;
  }

  .jump-btn:hover {
    background: var(--surface-pill);
  }

  .jump-btn:active,
  .action-btn:active {
    transform: scale(0.97);
  }

  .ignore-btn {
    margin-left: auto;
    color: var(--text-secondary);
    border-color: var(--hairline-border);
  }

  .ignore-btn:hover {
    background: rgba(255, 159, 10, 0.16);
    color: #ff9f0a;
    border-color: rgba(255, 159, 10, 0.4);
  }

  .restore-btn {
    margin-left: auto;
    color: var(--accent-color, #007aff);
    border-color: rgba(0, 122, 255, 0.3);
    background: rgba(0, 122, 255, 0.08);
  }

  .restore-btn:hover {
    background: rgba(0, 122, 255, 0.16);
    border-color: rgba(0, 122, 255, 0.5);
  }
</style>
