<script lang="ts">
  import { sessionStore } from "../../stores/session.svelte";
  import type { Course, EventCategory, GenericEvent } from "../../types/domain";
  import { getTodayIso } from "../../utils/calendar";

  interface Props {
    initialEvent?: GenericEvent | null;
    defaultDate?: string | null;
    courses: Course[];
    onClose?: () => void;
    onSaved?: () => void;
  }

  let {
    initialEvent = null,
    defaultDate = null,
    courses = [],
    onClose,
    onSaved
  }: Props = $props();

  let title = $state<string>("");
  let category = $state<EventCategory>("studio");
  let courseId = $state<string>("");
  let date = $state<string>(getTodayIso());
  let timeSlot = $state<string>("14:30");
  let details = $state<string>("");
  let isSaving = $state<boolean>(false);

  $effect(() => {
    if (initialEvent) {
      title = initialEvent.title;
      category = initialEvent.category;
      courseId = initialEvent.courseId ?? "";
      date = initialEvent.date;
      timeSlot = initialEvent.timeSlot || "";
      details = initialEvent.details || "";
    } else {
      title = "";
      category = "studio";
      courseId = "";
      date = defaultDate ?? getTodayIso();
      timeSlot = "14:30";
      details = "";
    }
  });

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    const sessionId = sessionStore.activeSession?.id;
    if (!sessionId || !title.trim()) return;

    isSaving = true;
    try {
      await sessionStore.upsertGenericEvent({
        id: initialEvent?.id ?? null,
        sessionId,
        courseId: courseId || null,
        date,
        timeSlot: timeSlot.trim() || null,
        title: title.trim(),
        category,
        details: details.trim() || null
      });
      onSaved?.();
      onClose?.();
    } finally {
      isSaving = false;
    }
  }

  async function handleDelete() {
    if (!initialEvent) return;
    isSaving = true;
    try {
      await sessionStore.deleteGenericEvent(initialEvent.id);
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
  aria-label={initialEvent ? "Modifica Impegno" : "Nuovo Impegno o Scadenza"}
  tabindex="-1"
  onclick={(e) => {
    if (e.target === e.currentTarget) onClose?.();
  }}
>
  <form class="modal-sheet" onsubmit={handleSubmit}>
    <header class="sheet-header">
      <div>
        <span class="sheet-eyebrow">AGENDA GIORNALIERA</span>
        <h2>{initialEvent ? "Modifica Impegno" : "Nuovo Impegno / Scadenza"}</h2>
      </div>
      <button type="button" class="sheet-close-btn" onclick={() => onClose?.()}>✕</button>
    </header>

    <div class="sheet-body">
      <label class="form-field">
        <span>Titolo Impegno *</span>
        <input type="text" required bind:value={title} placeholder="es. Consegna Relazione Laboratorio" />
      </label>

      <div class="field-row two-cols">
        <label class="form-field">
          <span>Categoria *</span>
          <select bind:value={category}>
            <option value="studio">Sessione di Studio</option>
            <option value="scadenza">Scadenza / Consegna</option>
            <option value="lezione">Lezione / Ricevimento</option>
            <option value="personale">Impegno Personale</option>
          </select>
        </label>

        <label class="form-field">
          <span>Materia Collegata</span>
          <select bind:value={courseId}>
            <option value="">Nessuna (Generale)</option>
            {#each courses as c (c.id)}
              <option value={c.id}>{c.sigla} — {c.name}</option>
            {/each}
          </select>
        </label>
      </div>

      <div class="field-row two-cols">
        <label class="form-field">
          <span>Data *</span>
          <input type="date" required bind:value={date} />
        </label>

        <label class="form-field">
          <span>Fascia Oraria</span>
          <input type="time" bind:value={timeSlot} />
        </label>
      </div>

      <label class="form-field">
        <span>Dettagli o Note</span>
        <textarea rows="2" bind:value={details} placeholder="Link, aula o note aggiuntive…"></textarea>
      </label>
    </div>

    <footer class="sheet-footer">
      {#if initialEvent}
        <button type="button" class="btn-danger" disabled={isSaving} onclick={handleDelete}>
          Elimina Impegno
        </button>
      {:else}
        <span></span>
      {/if}

      <div class="footer-right">
        <button type="button" class="btn-secondary" onclick={() => onClose?.()}>Annulla</button>
        <button type="submit" class="btn-primary" disabled={isSaving}>
          {isSaving ? "Salvataggio…" : initialEvent ? "Salva Modifiche" : "Aggiungi Impegno"}
        </button>
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
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1.5rem;
  }

  .modal-sheet {
    width: 100%;
    max-width: 32rem;
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
    font-size: 1.1rem;
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
    padding: 1rem 1.35rem;
    display: flex;
    flex-direction: column;
    gap: 0.8rem;
  }

  .field-row.two-cols {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
  }

  .form-field {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
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
    gap: 0.45rem;
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
