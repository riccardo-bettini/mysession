<script lang="ts">
  import { Check, Sparkles, X } from "lucide-svelte";
  import { navStore } from "../../stores/navigation.svelte";
  import { sessionStore } from "../../stores/session.svelte";
  import type { PlanDiffSummary } from "../../types/domain";

  let diff = $state<PlanDiffSummary | null>(null);
  let loadError = $state<string | null>(null);
  let resolutions = $state<Record<string, string>>({});

  $effect(() => {
    const pending = navStore.pendingPlanModal;
    if (!pending) {
      diff = null;
      loadError = null;
      return;
    }
    loadError = null;
    resolutions = {};
    void sessionStore
      .previewPlan(pending.filePath)
      .then((res) => {
        diff = res;
        const initRes: Record<string, string> = {};
        for (const c of res.conflicts) {
          initRes[c.taskId] = "keep_mine";
        }
        resolutions = initRes;
      })
      .catch((err) => {
        loadError = String(err);
      });
  });

  async function confirmApply() {
    const pending = navStore.pendingPlanModal;
    if (!pending) return;
    await sessionStore.applyPlan(pending.filePath, resolutions, null);
  }
</script>

{#if navStore.pendingPlanModal}
  <div class="modal-backdrop" role="dialog" aria-modal="true" aria-label="Importa piano di studio">
    <div class="bento-card plan-modal bento-enter">
      <header class="pm-head">
        <div class="pm-title">
          <Sparkles size={18} class="accent-ic" />
          <div>
            <h2>{navStore.pendingPlanModal.planName}</h2>
            <p>Revisione {navStore.pendingPlanModal.revision} · Schema v3</p>
          </div>
        </div>
        <button type="button" class="close-btn" onclick={() => (navStore.pendingPlanModal = null)}>
          <X size={18} />
        </button>
      </header>

      <div class="pm-body">
        {#if loadError}
          <div class="err-box">{loadError}</div>
        {:else if !diff}
          <p class="loading-txt">Analisi del piano v3 in corso...</p>
        {:else}
          <div class="summary-grid">
            <div class="sum-box"><strong>+{diff.addedTasks.length}</strong><span>Task aggiunte</span></div>
            <div class="sum-box"><strong>{diff.movedTasks.length}</strong><span>Spostate</span></div>
            <div class="sum-box"><strong>-{diff.removedTasks.length}</strong><span>Rimosse</span></div>
            <div class="sum-box"><strong>{diff.quizzesCount}</strong><span>Quiz giornalieri</span></div>
          </div>

          {#if diff.conflicts.length > 0}
            <div class="conflicts-box">
              <h4>Conflitti con modifiche manuali ({diff.conflicts.length})</h4>
              {#each diff.conflicts as c (c.taskId)}
                <div class="conf-row">
                  <div>
                    <strong>{c.myTitle}</strong>
                    <small>Tua data: {c.myDate} · Piano: {c.theirDate}</small>
                  </div>
                  <div class="seg-btns">
                    <button
                      type="button"
                      class="pill-filter"
                      class:active={resolutions[c.taskId] === "keep_mine"}
                      onclick={() => (resolutions = { ...resolutions, [c.taskId]: "keep_mine" })}
                    >
                      Mantieni mio
                    </button>
                    <button
                      type="button"
                      class="pill-filter"
                      class:active={resolutions[c.taskId] === "use_theirs"}
                      onclick={() => (resolutions = { ...resolutions, [c.taskId]: "use_theirs" })}
                    >
                      Usa piano
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        {/if}
      </div>

      <footer class="pm-foot">
        <button type="button" class="pill-btn pill-btn-soft" onclick={() => (navStore.pendingPlanModal = null)}>
          Annulla
        </button>
        <button
          type="button"
          class="pill-btn pill-btn-primary"
          disabled={!diff || sessionStore.isBusy}
          onclick={confirmApply}
        >
          <Check size={15} />
          {diff?.isInitialImport ? "Importa Sessione" : "Applica Aggiornamento"}
        </button>
      </footer>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 145;
    background: rgba(12, 12, 18, 0.55);
    backdrop-filter: blur(8px);
    display: grid;
    place-items: center;
    padding: 20px;
  }

  .plan-modal {
    width: min(600px, 100%);
    padding: 22px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .pm-head {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
  }

  .pm-title {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .pm-title h2 {
    margin: 0;
    font-size: 18px;
  }

  .pm-title p {
    margin: 2px 0 0;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .close-btn {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    background: var(--bg-surface-muted);
    display: grid;
    place-items: center;
  }

  .summary-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 10px;
  }

  .sum-box {
    padding: 12px;
    border-radius: 14px;
    background: var(--bg-surface-muted);
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .sum-box strong {
    font-size: 20px;
    font-weight: 800;
  }

  .sum-box span {
    font-size: 11px;
    color: var(--text-secondary);
  }

  .conflicts-box {
    margin-top: 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .conflicts-box h4 {
    margin: 0;
    font-size: 13px;
  }

  .conf-row {
    padding: 10px;
    border-radius: 12px;
    background: var(--bg-surface-muted);
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }

  .conf-row div {
    display: flex;
    flex-direction: column;
  }

  .seg-btns {
    display: flex;
    flex-direction: row !important;
    gap: 6px;
  }

  .err-box {
    padding: 12px;
    border-radius: 12px;
    background: rgba(239, 68, 68, 0.14);
    color: #DC2626;
    font-size: 12.5px;
    white-space: pre-wrap;
  }

  .pm-foot {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
  }
</style>
