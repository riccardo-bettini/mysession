<script lang="ts">
  import { sessionStore } from "../../stores/session.svelte";
  import type { AcademicSession } from "../../types/domain";

  interface Props {
    onClose?: () => void;
    onNewSession?: () => void;
    onEditSession?: (session: AcademicSession) => void;
  }

  let { onClose, onNewSession, onEditSession }: Props = $props();

  let activeSessionId = $derived(sessionStore.activeSession?.id ?? null);
  let sessions = $derived(sessionStore.sessions);

  let systemSession = $derived(
    sessions.find((s) => s.isSystem || s.id === "session_generale") ?? null
  );

  let userSessions = $derived(
    sessions.filter((s) => !s.isSystem && s.id !== "session_generale")
  );

  let isOperating = $state(false);
  let sessionToDelete = $state<AcademicSession | null>(null);

  async function handleSelect(sessionId: string) {
    if (activeSessionId === sessionId) return;
    isOperating = true;
    try {
      await sessionStore.setActiveSession(sessionId);
    } finally {
      isOperating = false;
    }
  }

  async function confirmDeleteSession() {
    if (!sessionToDelete) return;
    isOperating = true;
    try {
      await sessionStore.deleteSession(sessionToDelete.id);
      sessionToDelete = null;
    } finally {
      isOperating = false;
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="modal-scrim"
  role="dialog"
  aria-modal="true"
  aria-label="Gestione Sessioni Accademiche"
  tabindex="-1"
  onclick={(e) => {
    if (e.target === e.currentTarget) onClose?.();
  }}
>
  <div class="modal-sheet">
    <header class="sheet-header">
      <div>
        <span class="sheet-eyebrow">CONFIGURAZIONE ACCADEMICA</span>
        <h2>Gestione Sessioni</h2>
      </div>
      <div class="header-actions">
        <button
          type="button"
          class="btn-new-session pressable"
          onclick={() => {
            onNewSession?.();
          }}
        >
          + Nuova Sessione
        </button>
        <button
          type="button"
          class="sheet-close-btn pressable"
          title="Chiudi pannello (Esc)"
          onclick={() => onClose?.()}
        >
          ✕
        </button>
      </div>
    </header>

    <div class="sheet-body">
      <!-- 1. SESSIONE DI SISTEMA "GENERALE" -->
      {#if systemSession}
        <section class="session-category-block">
          <div class="category-header">
            <h3>Spazio Continuativo</h3>
            <span class="badge-system">SISTEMA</span>
          </div>

          <div
            class="session-card system-card"
            class:is-active={activeSessionId === systemSession.id}
          >
            <div class="card-left">
              <div class="card-icon-wrap">
                <span class="system-icon">🌐</span>
              </div>
              <div class="card-info">
                <div class="card-title-row">
                  <span class="card-name">{systemSession.name}</span>
                  {#if activeSessionId === systemSession.id}
                    <span class="pill-active">Attiva</span>
                  {/if}
                </div>
                <p class="card-desc">
                  Sessione continuativa sempre vuota e protetta. Ideale per consultare il calendario senza vincoli o esami prefissati.
                </p>
                <div class="card-meta-row">
                  <span class="meta-tag">🔒 Non modificabile</span>
                  <span class="meta-tag">♾ Scorrimento libero</span>
                </div>
              </div>
            </div>

            <div class="card-actions">
              {#if activeSessionId === systemSession.id}
                <span class="active-indicator">In uso</span>
              {:else}
                <button
                  type="button"
                  class="btn-activate pressable"
                  disabled={isOperating}
                  onclick={() => handleSelect(systemSession!.id)}
                >
                  Seleziona
                </button>
              {/if}
            </div>
          </div>
        </section>
      {/if}

      <!-- 2. SESSIONI UTENTE -->
      <section class="session-category-block">
        <div class="category-header">
          <h3>Sessioni Accademiche ({userSessions.length})</h3>
        </div>

        {#if userSessions.length === 0}
          <div class="empty-sessions-hint">
            <p>Nessuna sessione accademica personalizzata configurata.</p>
            <button
              type="button"
              class="btn-new-inline pressable"
              onclick={() => onNewSession?.()}
            >
              Crea la tua prima sessione
            </button>
          </div>
        {:else}
          <div class="sessions-stack">
            {#each userSessions as s (s.id)}
              <div class="session-card" class:is-active={activeSessionId === s.id}>
                <div class="card-left">
                  <div class="card-icon-wrap">
                    <span class="session-type-icon">
                      {#if s.sessionType === "invernale"}
                        ❄️
                      {:else if s.sessionType === "estiva"}
                        ☀️
                      {:else if s.sessionType === "autunnale"}
                        🍂
                      {:else}
                        📅
                      {/if}
                    </span>
                  </div>

                  <div class="card-info">
                    <div class="card-title-row">
                      <span class="card-name">{s.name}</span>
                      {#if s.academicYear}
                        <span class="pill-year">{s.academicYear}</span>
                      {/if}
                      {#if activeSessionId === s.id}
                        <span class="pill-active">Attiva</span>
                      {/if}
                    </div>

                    {#if s.subtitle}
                      <p class="card-subtitle">{s.subtitle}</p>
                    {/if}

                    <div class="card-meta-row">
                      <span class="meta-tag">🗓 {s.startDate} ➔ {s.endDate}</span>
                      <span class="meta-tag capitalize">🏷 {s.sessionType}</span>
                      {#if s.planId}
                        <span class="meta-tag plan-linked">📦 Collegata a Piano</span>
                      {/if}
                    </div>
                  </div>
                </div>

                <div class="card-actions">
                  {#if activeSessionId === s.id}
                    <span class="active-indicator">In uso</span>
                  {:else}
                    <button
                      type="button"
                      class="btn-activate pressable"
                      disabled={isOperating}
                      onclick={() => handleSelect(s.id)}
                    >
                      Seleziona
                    </button>
                  {/if}

                  <button
                    type="button"
                    class="btn-action-icon pressable"
                    title="Modifica parametri sessione"
                    disabled={isOperating}
                    onclick={() => onEditSession?.(s)}
                  >
                    ✎
                  </button>

                  <button
                    type="button"
                    class="btn-action-icon danger pressable"
                    title="Elimina sessione"
                    disabled={isOperating}
                    onclick={() => (sessionToDelete = s)}
                  >
                    🗑
                  </button>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </section>
    </div>

    <!-- MODALE DI CONFERMA ELIMINAZIONE SESSIONE -->
    {#if sessionToDelete}
      <div class="confirm-overlay">
        <div class="confirm-box">
          <div class="confirm-head">
            <span class="confirm-alert-icon">⚠</span>
            <h4>Eliminare "{sessionToDelete.name}"?</h4>
          </div>
          <p>
            Tutti i corsi, gli appelli d'esame, le lezioni e le attività pianificate associate a questa sessione verranno rimossi permanentemente dal database locale.
          </p>
          <div class="confirm-actions">
            <button
              type="button"
              class="btn-secondary pressable"
              disabled={isOperating}
              onclick={() => (sessionToDelete = null)}
            >
              Annulla
            </button>
            <button
              type="button"
              class="btn-danger pressable"
              disabled={isOperating}
              onclick={confirmDeleteSession}
            >
              {isOperating ? "Eliminazione…" : "Conferma Eliminazione"}
            </button>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .modal-scrim {
    position: fixed;
    inset: 0;
    z-index: 410;
    background: rgba(0, 0, 0, 0.48);
    backdrop-filter: blur(18px) saturate(180%);
    -webkit-backdrop-filter: blur(18px) saturate(180%);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1.5rem;
    animation: backdropFade 160ms cubic-bezier(0.16, 1, 0.3, 1);
  }

  .modal-sheet {
    position: relative;
    width: 100%;
    max-width: 44rem;
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
    animation: modalPop 180ms var(--ease-out);
  }

  @keyframes modalPop {
    from {
      opacity: 0;
      transform: scale(0.96);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .sheet-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1.15rem 1.45rem 1rem;
    border-bottom: 0.5px solid var(--hairline-border);
  }

  .sheet-eyebrow {
    font-size: 0.62rem;
    font-weight: 700;
    letter-spacing: 0.06em;
    color: var(--text-tertiary);
  }

  .sheet-header h2 {
    margin: 0.12rem 0 0;
    font-size: 1.25rem;
    font-weight: 800;
    letter-spacing: -0.025em;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.65rem;
  }

  .btn-new-session {
    height: 1.95rem;
    padding: 0 0.85rem;
    border-radius: var(--radius-sm);
    border: none;
    background: var(--accent-primary);
    color: #fff;
    font-size: 0.74rem;
    font-weight: 700;
    cursor: pointer;
    box-shadow: 0 1px 3px rgba(10, 132, 255, 0.3);
  }

  .sheet-close-btn {
    width: 1.85rem;
    height: 1.85rem;
    border-radius: 999px;
    border: 0.5px solid var(--hairline-border);
    background: var(--surface-pill);
    color: var(--text-secondary);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .sheet-body {
    padding: 1.2rem 1.45rem 1.6rem;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 1.35rem;
  }

  .session-category-block {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
  }

  .category-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .category-header h3 {
    margin: 0;
    font-size: 0.82rem;
    font-weight: 700;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .badge-system {
    font-size: 0.58rem;
    font-weight: 800;
    padding: 0.1rem 0.4rem;
    border-radius: var(--radius-pill);
    background: rgba(10, 132, 255, 0.15);
    color: var(--accent-primary);
  }

  .session-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.9rem 1.1rem;
    border-radius: var(--radius-md);
    background: var(--surface-card);
    border: 0.5px solid var(--hairline-border);
    transition: transform 140ms ease, border-color 140ms ease, box-shadow 140ms ease;
  }

  .session-card.is-active {
    border-color: var(--accent-primary);
    box-shadow: 0 0 0 1.5px var(--accent-primary), 0 4px 16px rgba(10, 132, 255, 0.15);
  }

  .session-card.system-card {
    background: rgba(10, 132, 255, 0.04);
  }

  .card-left {
    display: flex;
    align-items: flex-start;
    gap: 0.85rem;
    min-width: 0;
  }

  .card-icon-wrap {
    width: 2.35rem;
    height: 2.35rem;
    border-radius: var(--radius-sm);
    background: var(--surface-pill);
    border: 0.5px solid var(--hairline-border);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.15rem;
    flex-shrink: 0;
  }

  .card-info {
    display: flex;
    flex-direction: column;
    gap: 0.22rem;
    min-width: 0;
  }

  .card-title-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .card-name {
    font-size: 0.92rem;
    font-weight: 800;
    color: var(--text-primary);
  }

  .pill-year {
    font-size: 0.65rem;
    font-weight: 700;
    padding: 0.08rem 0.4rem;
    border-radius: var(--radius-pill);
    background: var(--surface-pill);
    color: var(--text-secondary);
    border: 0.5px solid var(--hairline-border);
  }

  .pill-active {
    font-size: 0.62rem;
    font-weight: 800;
    padding: 0.08rem 0.45rem;
    border-radius: var(--radius-pill);
    background: #34c759;
    color: #fff;
    letter-spacing: 0.03em;
  }

  .card-desc,
  .card-subtitle {
    margin: 0;
    font-size: 0.72rem;
    color: var(--text-secondary);
    line-height: 1.35;
  }

  .card-meta-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    margin-top: 0.2rem;
    flex-wrap: wrap;
  }

  .meta-tag {
    font-size: 0.65rem;
    font-weight: 600;
    color: var(--text-tertiary);
    background: var(--surface-pill);
    padding: 0.1rem 0.38rem;
    border-radius: 0.28rem;
  }

  .meta-tag.plan-linked {
    color: #af52de;
    background: rgba(175, 82, 222, 0.12);
  }

  .capitalize {
    text-transform: capitalize;
  }

  .card-actions {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    flex-shrink: 0;
  }

  .active-indicator {
    font-size: 0.72rem;
    font-weight: 700;
    color: #34c759;
    padding: 0.25rem 0.55rem;
  }

  .btn-activate {
    height: 1.85rem;
    padding: 0 0.75rem;
    border-radius: var(--radius-xs);
    border: 0.5px solid var(--hairline-border-strong);
    background: var(--surface-card);
    color: var(--text-primary);
    font-size: 0.72rem;
    font-weight: 600;
    cursor: pointer;
  }

  .btn-activate:hover {
    background: var(--surface-pill);
    border-color: var(--accent-primary);
  }

  .btn-action-icon {
    width: 1.85rem;
    height: 1.85rem;
    border-radius: var(--radius-xs);
    border: 0.5px solid var(--hairline-border);
    background: var(--surface-pill);
    color: var(--text-secondary);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-size: 0.75rem;
    cursor: pointer;
  }

  .btn-action-icon.danger:hover {
    background: rgba(255, 69, 58, 0.18);
    color: #ff453a;
  }

  .sessions-stack {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
  }

  .empty-sessions-hint {
    padding: 1.5rem;
    text-align: center;
    border-radius: var(--radius-md);
    border: 1px dashed var(--hairline-border-strong);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
  }

  .empty-sessions-hint p {
    margin: 0;
    font-size: 0.78rem;
    color: var(--text-secondary);
  }

  .btn-new-inline {
    height: 1.9rem;
    padding: 0 0.85rem;
    border-radius: var(--radius-sm);
    border: none;
    background: var(--accent-primary);
    color: #fff;
    font-size: 0.74rem;
    font-weight: 700;
    cursor: pointer;
  }

  /* Modale di conferma interna */
  .confirm-overlay {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(10px);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1.5rem;
    z-index: 50;
    animation: backdropFade 120ms ease;
  }

  .confirm-box {
    width: 100%;
    max-width: 25rem;
    border-radius: var(--radius-lg);
    background: var(--popover-bg);
    border: 0.5px solid var(--hairline-border-strong);
    padding: 1.25rem 1.4rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    box-shadow: 0 20px 50px rgba(0, 0, 0, 0.5);
  }

  .confirm-head {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .confirm-alert-icon {
    font-size: 1.2rem;
    color: #ff9500;
  }

  .confirm-head h4 {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 800;
  }

  .confirm-box p {
    margin: 0;
    font-size: 0.75rem;
    color: var(--text-secondary);
    line-height: 1.42;
  }

  .confirm-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 0.4rem;
  }

  .btn-secondary,
  .btn-danger {
    height: 2rem;
    padding: 0 0.85rem;
    border-radius: var(--radius-xs);
    font-size: 0.74rem;
    font-weight: 600;
    cursor: pointer;
  }

  .btn-secondary {
    border: 0.5px solid var(--hairline-border-strong);
    background: var(--surface-card);
    color: var(--text-primary);
  }

  .btn-danger {
    border: none;
    background: #ff453a;
    color: #fff;
  }
</style>
