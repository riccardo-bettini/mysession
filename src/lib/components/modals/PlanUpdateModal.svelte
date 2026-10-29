<script lang="ts">
  import {
    AlertTriangle,
    ArrowRight,
    Calendar,
    Check,
    CheckCircle2,
    Clock,
    Layers,
    Sparkles,
    X
  } from "lucide-svelte";
  import { planStore } from "../../stores/plan.svelte";

  interface Props {
    onToast?: (msg: string) => void;
  }

  let { onToast }: Props = $props();

  let isApplying = $state(false);
  let errorMessage = $state<string | null>(null);

  let diff = $derived(planStore.activePreview);

  function handleSelectConflict(taskId: string, choice: "keep_mine" | "use_theirs") {
    planStore.setConflictResolution(taskId, choice);
  }

  async function handleApply() {
    isApplying = true;
    errorMessage = null;
    try {
      await planStore.applyActivePlan();
      onToast?.("Piano di studio applicato con successo al calendario!");
    } catch (e) {
      errorMessage = e instanceof Error ? e.message : String(e);
    } finally {
      isApplying = false;
    }
  }

  function handleClose() {
    planStore.closePreview();
  }
</script>

{#if planStore.isUpdateModalOpen && diff}
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    aria-labelledby="plan-modal-title"
    tabindex="-1"
    onkeydown={(e) => e.key === "Escape" && handleClose()}
  >
    <div class="modal-card">
      <!-- Header -->
      <header class="modal-header">
        <div class="header-content">
          <div class="header-icon">
            <Sparkles size={20} class="text-sky-500" />
          </div>
          <div>
            <div class="header-badges">
              {#if diff.isInitialImport}
                <span class="badge badge-emerald">Nuovo Piano Rilevato</span>
              {:else}
                <span class="badge badge-sky">Aggiornamento Piano</span>
                <span class="badge badge-neutral">Rev. {diff.oldRevision} → {diff.newRevision}</span>
              {/if}
            </div>
            <h2 id="plan-modal-title" class="modal-title">
              {diff.planId}
            </h2>
          </div>
        </div>
        <button
          type="button"
          class="close-button"
          aria-label="Chiudi"
          onclick={handleClose}
        >
          <X size={18} />
        </button>
      </header>

      <!-- Scrollable Body -->
      <div class="modal-body custom-scroll">
        {#if errorMessage}
          <div class="banner banner-error">
            <AlertTriangle size={16} />
            <span>{errorMessage}</span>
          </div>
        {/if}

        <!-- Metric Pills -->
        <div class="summary-grid">
          <div class="metric-card">
            <span class="metric-num text-emerald-500">+{diff.addedTasks.length}</span>
            <span class="metric-label">Attività Nuove</span>
          </div>
          <div class="metric-card">
            <span class="metric-num text-sky-500">~{diff.movedTasks.length}</span>
            <span class="metric-label">Spostamenti</span>
          </div>
          <div class="metric-card">
            <span class="metric-num text-rose-500">-{diff.removedTasks.length}</span>
            <span class="metric-label">Rimosse</span>
          </div>
          <div class="metric-card">
            <span class="metric-num {diff.conflicts.length > 0 ? 'text-amber-500' : 'text-zinc-400'}">
              !{diff.conflicts.length}
            </span>
            <span class="metric-label">Conflitti Manuali</span>
          </div>
        </div>

        <!-- Section: Conflicts (High Priority) -->
        {#if diff.conflicts.length > 0}
          <section class="section-box conflict-box">
            <div class="section-header text-amber-500">
              <AlertTriangle size={18} />
              <h3 class="section-title text-amber-500">
                Risoluzione Conflitti ({diff.conflicts.length})
              </h3>
            </div>
            <p class="section-desc">
              Hai modificato manualmente queste attività in MySession. Scegli per ciascuna se conservare le tue modifiche o adottare la nuova pianificazione del Builder:
            </p>

            <div class="conflict-list">
              {#each diff.conflicts as conflict (conflict.taskId)}
                {@const resolution = planStore.activeConflictResolutions[conflict.taskId] || "use_theirs"}
                <div class="conflict-card">
                  <div class="conflict-info">
                    <span class="conflict-task-text">{conflict.baseText}</span>
                    {#if conflict.isCompleted}
                      <span class="badge badge-emerald">Già Completata</span>
                    {/if}
                  </div>

                  <div class="conflict-choices">
                    <button
                      type="button"
                      class="choice-btn {resolution === 'keep_mine' ? 'active-mine' : ''}"
                      onclick={() => handleSelectConflict(conflict.taskId, "keep_mine")}
                    >
                      <div class="choice-title">
                        <CheckCircle2 size={15} />
                        <span>Mantieni la Mia Versione</span>
                      </div>
                      <div class="choice-meta">
                        <span>Data: <strong>{conflict.myDate}</strong></span>
                        {#if conflict.myText !== conflict.baseText}
                          <span>Testo: "{conflict.myText}"</span>
                        {/if}
                      </div>
                    </button>

                    <button
                      type="button"
                      class="choice-btn {resolution === 'use_theirs' ? 'active-theirs' : ''}"
                      onclick={() => handleSelectConflict(conflict.taskId, "use_theirs")}
                    >
                      <div class="choice-title">
                        <ArrowRight size={15} />
                        <span>Usa quella del Nuovo Piano</span>
                      </div>
                      <div class="choice-meta">
                        <span>Data: <strong>{conflict.theirDate}</strong></span>
                        {#if conflict.theirText !== conflict.baseText}
                          <span>Testo: "{conflict.theirText}"</span>
                        {/if}
                      </div>
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          </section>
        {/if}

        <!-- Section: Spostamenti -->
        {#if diff.movedTasks.length > 0}
          <section class="section-box">
            <div class="section-header">
              <Calendar size={18} class="text-sky-500" />
              <h3 class="section-title">Date Ricalibrate ({diff.movedTasks.length})</h3>
            </div>
            <div class="task-list">
              {#each diff.movedTasks as task (task.taskId)}
                <div class="task-row">
                  <span class="task-desc">{task.text}</span>
                  <div class="task-dates">
                    <span class="date-tag line-through opacity-60">{task.oldDate}</span>
                    <ArrowRight size={14} class="text-sky-500" />
                    <span class="date-tag font-medium text-sky-400">{task.newDate}</span>
                  </div>
                </div>
              {/each}
            </div>
          </section>
        {/if}

        <!-- Section: Attività Aggiunte -->
        {#if diff.addedTasks.length > 0}
          <section class="section-box">
            <div class="section-header">
              <Layers size={18} class="text-emerald-500" />
              <h3 class="section-title">Nuove Attività da Schedulare ({diff.addedTasks.length})</h3>
            </div>
            <div class="task-list">
              {#each diff.addedTasks.slice(0, 10) as task (task.taskId)}
                <div class="task-row">
                  <span class="task-desc">{task.text}</span>
                  <div class="task-dates">
                    <span class="date-tag">{task.date}</span>
                    <span class="badge badge-neutral">{task.plannedHours}h</span>
                  </div>
                </div>
              {/each}
              {#if diff.addedTasks.length > 10}
                <div class="more-row">
                  + altre {diff.addedTasks.length - 10} attività pianificate
                </div>
              {/if}
            </div>
          </section>
        {/if}

        <!-- Section: Esami Aggiornati -->
        {#if diff.examsChanged.length > 0}
          <section class="section-box">
            <div class="section-header">
              <Clock size={18} class="text-purple-500" />
              <h3 class="section-title">Appelli d'Esame ({diff.examsChanged.length})</h3>
            </div>
            <div class="task-list">
              {#each diff.examsChanged as exam (exam.examId)}
                <div class="task-row">
                  <span class="task-desc">{exam.title || exam.examId}</span>
                  <div class="task-dates">
                    {#if exam.oldDate}
                      <span class="date-tag line-through opacity-60">{exam.oldDate}</span>
                      <ArrowRight size={14} />
                    {/if}
                    <span class="date-tag text-purple-400 font-semibold">{exam.newDate}</span>
                    {#if exam.newStatus === "withdrawn"}
                      <span class="badge badge-rose">Ritirato</span>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          </section>
        {/if}
      </div>

      <!-- Footer Actions -->
      <footer class="modal-footer">
        <button
          type="button"
          class="btn-secondary"
          onclick={handleClose}
          disabled={isApplying}
        >
          Più tardi
        </button>
        <button
          type="button"
          class="btn-primary"
          onclick={handleApply}
          disabled={isApplying}
        >
          {#if isApplying}
            <span class="spinner"></span>
            <span>Applicazione in corso...</span>
          {:else}
            <Check size={16} />
            <span>Applica al Calendario</span>
          {/if}
        </button>
      </footer>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 9999;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.65);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    padding: 1rem;
    animation: fadeIn 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: scale(0.98);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .modal-card {
    display: flex;
    flex-direction: column;
    width: 100%;
    max-width: 680px;
    max-height: 90vh;
    background: var(--bg-surface, #18181b);
    border: 0.5px solid rgba(255, 255, 255, 0.12);
    border-radius: 1.25rem;
    box-shadow: 0 24px 48px -12px rgba(0, 0, 0, 0.5);
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1.25rem 1.5rem;
    border-bottom: 0.5px solid rgba(255, 255, 255, 0.08);
  }

  .header-content {
    display: flex;
    align-items: center;
    gap: 0.875rem;
  }

  .header-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    border-radius: 0.75rem;
    background: rgba(14, 165, 233, 0.12);
    border: 0.5px solid rgba(14, 165, 233, 0.25);
  }

  .header-badges {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 0.25rem;
  }

  .modal-title {
    font-size: 1.15rem;
    font-weight: 600;
    color: var(--text-primary, #fafafa);
    margin: 0;
  }

  .close-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: 0.5rem;
    background: transparent;
    border: none;
    color: var(--text-secondary, #a1a1aa);
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .close-button:hover {
    background: rgba(255, 255, 255, 0.08);
    color: #fff;
  }

  .modal-body {
    flex: 1;
    overflow-y: auto;
    padding: 1.25rem 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .summary-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 0.75rem;
  }

  .metric-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 0.75rem 0.5rem;
    border-radius: 0.875rem;
    background: rgba(255, 255, 255, 0.03);
    border: 0.5px solid rgba(255, 255, 255, 0.06);
  }

  .metric-num {
    font-size: 1.35rem;
    font-weight: 700;
    line-height: 1;
    margin-bottom: 0.25rem;
  }

  .metric-label {
    font-size: 0.75rem;
    color: var(--text-secondary, #a1a1aa);
  }

  .section-box {
    border-radius: 1rem;
    padding: 1rem;
    background: rgba(255, 255, 255, 0.02);
    border: 0.5px solid rgba(255, 255, 255, 0.06);
  }

  .conflict-box {
    background: rgba(245, 158, 11, 0.05);
    border-color: rgba(245, 158, 11, 0.2);
  }

  .section-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }

  .section-title {
    font-size: 0.95rem;
    font-weight: 600;
    margin: 0;
  }

  .section-desc {
    font-size: 0.8rem;
    color: var(--text-secondary, #a1a1aa);
    margin: 0 0 0.875rem 0;
    line-height: 1.4;
  }

  .conflict-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .conflict-card {
    padding: 0.875rem;
    border-radius: 0.75rem;
    background: rgba(0, 0, 0, 0.3);
    border: 0.5px solid rgba(255, 255, 255, 0.08);
  }

  .conflict-info {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.75rem;
  }

  .conflict-task-text {
    font-size: 0.9rem;
    font-weight: 500;
    color: #fafafa;
  }

  .conflict-choices {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.625rem;
  }

  .choice-btn {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    padding: 0.625rem 0.75rem;
    border-radius: 0.625rem;
    background: rgba(255, 255, 255, 0.04);
    border: 0.5px solid rgba(255, 255, 255, 0.1);
    color: var(--text-secondary, #a1a1aa);
    cursor: pointer;
    text-align: left;
    transition: all 0.15s ease;
  }

  .choice-btn:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .choice-btn.active-mine {
    background: rgba(16, 185, 129, 0.15);
    border-color: rgba(16, 185, 129, 0.4);
    color: #34d399;
  }

  .choice-btn.active-theirs {
    background: rgba(14, 165, 233, 0.15);
    border-color: rgba(14, 165, 233, 0.4);
    color: #38bdf8;
  }

  .choice-title {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    font-size: 0.8rem;
    font-weight: 600;
    margin-bottom: 0.25rem;
  }

  .choice-meta {
    font-size: 0.75rem;
    opacity: 0.85;
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
  }

  .task-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .task-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 0.625rem;
    border-radius: 0.5rem;
    background: rgba(255, 255, 255, 0.02);
    font-size: 0.85rem;
  }

  .task-desc {
    color: #e4e4e7;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 65%;
  }

  .task-dates {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .date-tag {
    font-size: 0.75rem;
    font-family: monospace;
    color: #d4d4d8;
  }

  .more-row {
    text-align: center;
    font-size: 0.75rem;
    color: #71717a;
    padding: 0.25rem 0;
  }

  .modal-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 0.75rem;
    padding: 1rem 1.5rem;
    border-top: 0.5px solid rgba(255, 255, 255, 0.08);
    background: rgba(0, 0, 0, 0.2);
  }

  .btn-secondary {
    padding: 0.6rem 1.1rem;
    border-radius: 0.625rem;
    background: rgba(255, 255, 255, 0.06);
    border: 0.5px solid rgba(255, 255, 255, 0.1);
    color: #e4e4e7;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .btn-secondary:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.1);
  }

  .btn-primary {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.6rem 1.25rem;
    border-radius: 0.625rem;
    background: #0284c7;
    border: none;
    color: #fff;
    font-size: 0.875rem;
    font-weight: 600;
    cursor: pointer;
    transition: filter 0.15s ease, transform 0.1s ease;
  }

  .btn-primary:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .btn-primary:active:not(:disabled) {
    transform: scale(0.98);
  }

  .badge {
    padding: 0.2rem 0.5rem;
    border-radius: 0.375rem;
    font-size: 0.7rem;
    font-weight: 600;
    letter-spacing: 0.02em;
    text-transform: uppercase;
  }

  .badge-emerald {
    background: rgba(16, 185, 129, 0.15);
    color: #34d399;
  }

  .badge-sky {
    background: rgba(14, 165, 233, 0.15);
    color: #38bdf8;
  }

  .badge-rose {
    background: rgba(244, 63, 94, 0.15);
    color: #fb7185;
  }

  .badge-neutral {
    background: rgba(255, 255, 255, 0.08);
    color: #d4d4d8;
  }

  .banner {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1rem;
    border-radius: 0.625rem;
    font-size: 0.85rem;
  }

  .banner-error {
    background: rgba(239, 68, 68, 0.15);
    border: 0.5px solid rgba(239, 68, 68, 0.3);
    color: #fca5a5;
  }

  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: #fff;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
