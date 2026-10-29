<script lang="ts">
  import { sessionStore } from "../../stores/session.svelte";
  import type { AcademicSession, SessionType } from "../../types/domain";
  import { getTodayIso } from "../../utils/calendar";

  interface Props {
    initialSession?: AcademicSession | null;
    onClose?: () => void;
    onSaved?: (session: AcademicSession) => void;
  }

  let {
    initialSession = null,
    onClose,
    onSaved
  }: Props = $props();

  let name = $state<string>("");
  let subtitle = $state<string>("");
  let sessionType = $state<SessionType>("invernale");
  let academicYear = $state<string>("2026/2027");
  let startDate = $state<string>(getTodayIso());
  let endDate = $state<string>(getTodayIso());
  let notes = $state<string>("");
  let isSaving = $state<boolean>(false);

  let isProtected = $derived(
    Boolean(initialSession?.isSystem || initialSession?.id === "session_generale")
  );

  $effect(() => {
    if (initialSession) {
      name = initialSession.name;
      subtitle = initialSession.subtitle;
      sessionType = initialSession.sessionType;
      academicYear = initialSession.academicYear;
      startDate = initialSession.startDate;
      endDate = initialSession.endDate;
      notes = initialSession.notes;
    } else {
      const y = new Date().getFullYear();
      name = "Sessione Invernale";
      subtitle = "Pianificazione Appelli";
      sessionType = "invernale";
      academicYear = `${y}/${y + 1}`;
      startDate = `${y + 1}-01-07`;
      endDate = `${y + 1}-02-28`;
      notes = "";
    }
  });

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    if (isProtected || !name.trim()) return;
    isSaving = true;
    try {
      const payload = await sessionStore.upsertSession({
        id: initialSession?.id ?? null,
        name: name.trim(),
        subtitle: subtitle.trim(),
        sessionType,
        academicYear: academicYear.trim(),
        startDate,
        endDate,
        notes: notes.trim() || null
      });
      if (payload.activeSnapshot?.session) {
        onSaved?.(payload.activeSnapshot.session);
      }
      onClose?.();
    } finally {
      isSaving = false;
    }
  }

  async function handleDelete() {
    if (!initialSession || isProtected) return;
    isSaving = true;
    try {
      await sessionStore.deleteSession(initialSession.id);
      onClose?.();
    } finally {
      isSaving = false;
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="modal-scrim"
  role="dialog"
  aria-modal="true"
  aria-label={initialSession ? "Dettagli Sessione Accademica" : "Nuova Sessione Accademica"}
  tabindex="-1"
  onclick={(e) => {
    if (e.target === e.currentTarget) onClose?.();
  }}
>
  <form class="modal-sheet" onsubmit={handleSubmit}>
    <header class="sheet-header">
      <div>
        <span class="sheet-eyebrow">GESTIONE MULTI-SESSIONE</span>
        <h2>{initialSession ? (isProtected ? "Sessione Generale (Protetta)" : "Modifica Sessione Accademica") : "Nuova Sessione Accademica"}</h2>
      </div>
      <button type="button" class="sheet-close-btn pressable" onclick={() => onClose?.()}>✕</button>
    </header>

    <div class="sheet-body">
      {#if isProtected}
        <div class="protected-session-banner">
          <span class="prot-icon">🔒</span>
          <div class="prot-content">
            <strong>Sessione di Sistema Permanente</strong>
            <p>La sessione "Generale" è lo spazio libero continuativo predefinito: è sempre vuota, non modificabile e protetta da eliminazione.</p>
          </div>
        </div>
      {/if}

      <div class="field-row two-cols">
        <label class="form-field">
          <span>Nome Sessione *</span>
          <input type="text" required disabled={isProtected} bind:value={name} placeholder="es. Sessione Invernale" />
        </label>

        <label class="form-field">
          <span>Tipologia</span>
          <select disabled={isProtected} bind:value={sessionType}>
            <option value="invernale">Invernale (Gen–Feb)</option>
            <option value="estiva">Estiva (Giu–Lug)</option>
            <option value="autunnale">Autunnale (Set)</option>
            <option value="custom">Personalizzata / Straordinaria</option>
          </select>
        </label>
      </div>

      <div class="field-row two-cols">
        <label class="form-field">
          <span>Anno Accademico</span>
          <input type="text" disabled={isProtected} bind:value={academicYear} placeholder="es. 2026/2027" />
        </label>

        <label class="form-field">
          <span>Sottotitolo / Corso di Studi</span>
          <input type="text" disabled={isProtected} bind:value={subtitle} placeholder="es. Ingegneria Meccanica" />
        </label>
      </div>

      <div class="field-row two-cols">
        <label class="form-field">
          <span>Data Inizio Sessione *</span>
          <input type="date" required disabled={isProtected} bind:value={startDate} />
        </label>

        <label class="form-field">
          <span>Data Fine Sessione *</span>
          <input type="date" required disabled={isProtected} bind:value={endDate} />
        </label>
      </div>

      <label class="form-field">
        <span>Note di Sessione</span>
        <textarea rows="3" disabled={isProtected} bind:value={notes} placeholder="Obiettivi CFU, vincoli o promemoria…"></textarea>
      </label>
    </div>

    <footer class="sheet-footer">
      {#if !isProtected && initialSession && sessionStore.sessions.length > 1}
        <button type="button" class="btn-danger pressable" disabled={isSaving} onclick={handleDelete}>
          Elimina Sessione
        </button>
      {:else}
        <span></span>
      {/if}

      <div class="footer-right">
        {#if isProtected}
          <button type="button" class="btn-primary pressable" onclick={() => onClose?.()}>Chiudi</button>
        {:else}
          <button type="button" class="btn-secondary pressable" onclick={() => onClose?.()}>Annulla</button>
          <button type="submit" class="btn-primary pressable" disabled={isSaving}>
            {isSaving ? "Salvataggio…" : initialSession ? "Salva Modifiche" : "Crea Sessione"}
          </button>
        {/if}
      </div>
    </footer>
  </form>
</div>

<style>
  .modal-scrim {
    position: fixed;
    inset: 0;
    z-index: 400;
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
    max-width: 34rem;
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
      transform: scale(0.95);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .sheet-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    padding: 1.15rem 1.35rem 0.85rem;
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

  .sheet-body {
    padding: 1.1rem 1.35rem;
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
  }

  .protected-session-banner {
    display: flex;
    gap: 0.75rem;
    align-items: flex-start;
    padding: 0.75rem 0.9rem;
    border-radius: var(--radius-sm);
    background: rgba(10, 132, 255, 0.08);
    border: 0.5px solid rgba(10, 132, 255, 0.28);
  }

  .prot-icon {
    font-size: 1.15rem;
    line-height: 1;
  }

  .prot-content strong {
    display: block;
    font-size: 0.8rem;
    font-weight: 700;
    color: var(--accent-primary);
    margin-bottom: 0.2rem;
  }

  .prot-content p {
    margin: 0;
    font-size: 0.72rem;
    line-height: 1.38;
    color: var(--text-secondary);
  }

  .field-row.two-cols {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
  }

  .form-field {
    display: flex;
    flex-direction: column;
    gap: 0.32rem;
  }

  .form-field span {
    font-size: 0.72rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .form-field input,
  .form-field select,
  .form-field textarea {
    padding: 0.48rem 0.65rem;
    border-radius: var(--radius-sm);
    border: 0.5px solid var(--hairline-border-strong);
    background: var(--surface-card);
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: 0.8rem;
    transition: border-color 140ms ease, box-shadow 140ms ease;
  }

  .form-field input:disabled,
  .form-field select:disabled,
  .form-field textarea:disabled {
    opacity: 0.65;
    cursor: not-allowed;
    background: var(--surface-pill);
  }

  .form-field input:focus-visible:not(:disabled),
  .form-field select:focus-visible:not(:disabled),
  .form-field textarea:focus-visible:not(:disabled) {
    outline: none;
    border-color: var(--accent-primary);
    box-shadow: 0 0 0 2.5px var(--accent-soft);
  }

  .sheet-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.85rem 1.35rem;
    border-top: 0.5px solid var(--hairline-border);
    background: var(--surface-pill);
  }

  .footer-right {
    display: flex;
    gap: 0.5rem;
  }

  .btn-primary,
  .btn-secondary,
  .btn-danger {
    height: 2.05rem;
    padding: 0 0.9rem;
    border-radius: var(--radius-sm);
    font-size: 0.76rem;
    font-weight: 600;
    cursor: pointer;
    transition: transform 120ms cubic-bezier(0.16, 1, 0.3, 1), filter 120ms ease;
  }

  .btn-primary:active,
  .btn-secondary:active,
  .btn-danger:active {
    transform: scale(0.97);
  }

  .btn-primary {
    border: none;
    background: var(--accent-primary);
    color: #fff;
  }

  .btn-secondary {
    border: 0.5px solid var(--hairline-border-strong);
    background: var(--surface-card);
    color: var(--text-primary);
  }

  .btn-danger {
    border: 0.5px solid rgba(255, 69, 58, 0.45);
    background: rgba(255, 69, 58, 0.14);
    color: #ff453a;
  }
</style>