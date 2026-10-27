<script lang="ts">
  import { sessionStore } from "../../stores/session.svelte";
  import type { Course, Exam } from "../../types/domain";
  import { addDaysIso, getTodayIso } from "../../utils/calendar";

  interface Props {
    initialExam?: Exam | null;
    defaultDate?: string | null;
    defaultCourseId?: string | null;
    courses: Course[];
    onClose?: () => void;
    onSaved?: () => void;
  }

  let {
    initialExam = null,
    defaultDate = null,
    defaultCourseId = null,
    courses = [],
    onClose,
    onSaved
  }: Props = $props();

  let mode = $state<"single" | "itinere_pair">("single");
  let courseId = $state<string>("");
  let date1 = $state<string>(getTodayIso());
  let date2 = $state<string>(addDaysIso(getTodayIso(), 45));
  let timeSlot1 = $state<string>("09:00");
  let timeSlot2 = $state<string>("09:00");
  let round = $state<string>("1");
  let examType = $state<string>("Scritto + Orale");
  let classroom = $state<string>("");
  let status = $state<"planned" | "passed" | "failed" | "withdrawn">("planned");
  let grade = $state<string>("");
  let details = $state<string>("");
  let isSaving = $state<boolean>(false);

  $effect(() => {
    if (initialExam) {
      mode = "single";
      courseId = initialExam.courseId;
      date1 = initialExam.date;
      timeSlot1 = initialExam.timeSlot || "09:00";
      round = initialExam.round;
      examType = initialExam.examType || "Scritto + Orale";
      classroom = initialExam.classroom || "";
      status = initialExam.status || "planned";
      grade = initialExam.grade ?? "";
      details = initialExam.details || "";
    } else {
      mode = "single";
      courseId = defaultCourseId ?? courses[0]?.id ?? "";
      const d = defaultDate ?? sessionStore.activeSession?.startDate ?? getTodayIso();
      date1 = d;
      date2 = addDaysIso(d, 45);
      timeSlot1 = "09:00";
      timeSlot2 = "09:00";
      round = "1";
      examType = "Scritto + Orale";
      classroom = "";
      status = "planned";
      grade = "";
      details = "";
    }
  });

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    const sessionId = sessionStore.activeSession?.id;
    if (!sessionId || !courseId) return;

    isSaving = true;
    try {
      if (!initialExam && mode === "itinere_pair") {
        await sessionStore.createItinerePair({
          sessionId,
          courseId,
          date1,
          date2,
          timeSlot1: timeSlot1.trim() || null,
          timeSlot2: timeSlot2.trim() || null,
          classroom: classroom.trim() || null,
          details: details.trim() || null
        });
      } else {
        await sessionStore.upsertExam({
          id: initialExam?.id ?? null,
          sessionId,
          courseId,
          date: date1,
          timeSlot: timeSlot1.trim() || null,
          durationMinutes: 120,
          round,
          examType,
          classroom: classroom.trim() || null,
          weightFraction: round.startsWith("itinere") ? 0.5 : 1.0,
          pairId: initialExam?.pairId ?? null,
          title: null,
          details: details.trim() || null,
          status,
          grade: grade.trim() || null
        });
      }
      onSaved?.();
      onClose?.();
    } finally {
      isSaving = false;
    }
  }

  async function handleDelete(deletePair = false) {
    if (!initialExam) return;
    isSaving = true;
    try {
      await sessionStore.deleteExam(initialExam.id, deletePair);
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
  aria-label={initialExam ? "Modifica Appello" : "Pianifica Appello o Prove in Itinere"}
  tabindex="-1"
  onclick={(e) => {
    if (e.target === e.currentTarget) onClose?.();
  }}
>
  <form class="modal-sheet" onsubmit={handleSubmit}>
    <header class="sheet-header">
      <div>
        <span class="sheet-eyebrow">PIANIFICAZIONE ESAMI & PARZIALI</span>
        <h2>{initialExam ? "Modifica Appello d'Esame" : "Nuovo Appello / Prove in Itinere"}</h2>
      </div>
      <button type="button" class="sheet-close-btn" onclick={() => onClose?.()}>✕</button>
    </header>

    <div class="sheet-body">
      {#if !initialExam}
        <div class="mode-switch-bar">
          <button
            type="button"
            class="mode-btn"
            class:active={mode === "single"}
            onclick={() => (mode = "single")}
          >
            Appello Singolo (1° / 2° / 3° Giro)
          </button>
          <button
            type="button"
            class="mode-btn"
            class:active={mode === "itinere_pair"}
            onclick={() => (mode = "itinere_pair")}
          >
            Coppia Prove in Itinere (1° + 2° Parziale)
          </button>
        </div>
      {/if}

      <label class="form-field">
        <span>Insegnamento / Materia *</span>
        <select required bind:value={courseId}>
          {#each courses as c (c.id)}
            <option value={c.id}>{c.sigla} — {c.name} ({c.cfu} CFU)</option>
          {/each}
        </select>
      </label>

      {#if mode === "itinere_pair" && !initialExam}
        <div class="itinere-pair-box">
          <div class="pair-col">
            <span class="pair-badge">1ª PROVA IN ITINERE (50%)</span>
            <label class="form-field">
              <span>Data 1° Parziale *</span>
              <input type="date" required bind:value={date1} />
            </label>
            <label class="form-field">
              <span>Orario 1° Parziale</span>
              <input type="time" bind:value={timeSlot1} />
            </label>
          </div>

          <div class="pair-col">
            <span class="pair-badge">2ª PROVA IN ITINERE (50%)</span>
            <label class="form-field">
              <span>Data 2° Parziale *</span>
              <input type="date" required bind:value={date2} />
            </label>
            <label class="form-field">
              <span>Orario 2° Parziale</span>
              <input type="time" bind:value={timeSlot2} />
            </label>
          </div>
        </div>
      {:else}
        <div class="field-row two-cols">
          <label class="form-field">
            <span>Giro d'Appello *</span>
            <select bind:value={round}>
              <option value="1">1° Appello</option>
              <option value="2">2° Appello</option>
              <option value="3">3° Appello / Straordinario</option>
              <option value="itinere_1">1ª Prova in Itinere</option>
              <option value="itinere_2">2ª Prova in Itinere</option>
            </select>
          </label>

          <label class="form-field">
            <span>Modalità d'Esame</span>
            <select bind:value={examType}>
              <option value="Scritto + Orale">Scritto + Orale</option>
              <option value="Scritto">Solo Scritto</option>
              <option value="Orale">Solo Orale</option>
              <option value="Progetto">Progetto / Laboratorio</option>
            </select>
          </label>
        </div>

        <div class="field-row two-cols">
          <label class="form-field">
            <span>Data Appello *</span>
            <input type="date" required bind:value={date1} />
          </label>

          <label class="form-field">
            <span>Fascia Oraria</span>
            <input type="time" bind:value={timeSlot1} />
          </label>
        </div>
      {/if}

      <div class="field-row two-cols">
        <label class="form-field">
          <span>Aula / Edificio</span>
          <input type="text" bind:value={classroom} placeholder="es. Aula B.2.1" />
        </label>

        {#if initialExam}
          <label class="form-field">
            <span>Esito / Stato</span>
            <select bind:value={status}>
              <option value="planned">Pianificato</option>
              <option value="passed">Superato</option>
              <option value="failed">Da ripetere</option>
              <option value="withdrawn">Ritirato</option>
            </select>
          </label>
        {:else}
          <label class="form-field">
            <span>Voto / Target (opzionale)</span>
            <input type="text" bind:value={grade} placeholder="es. 30L" />
          </label>
        {/if}
      </div>

      <label class="form-field">
        <span>Dettagli & Argomenti della Prova</span>
        <textarea rows="2" bind:value={details} placeholder="Capitoli, regole d'esame, materiale ammesso…"></textarea>
      </label>
    </div>

    <footer class="sheet-footer">
      {#if initialExam}
        <div class="delete-group">
          <button type="button" class="btn-danger" disabled={isSaving} onclick={() => handleDelete(false)}>
            Elimina
          </button>
          {#if initialExam.pairId}
            <button type="button" class="btn-danger" disabled={isSaving} onclick={() => handleDelete(true)}>
              Elimina Coppia Itinere
            </button>
          {/if}
        </div>
      {:else}
        <span></span>
      {/if}

      <div class="footer-right">
        <button type="button" class="btn-secondary" onclick={() => onClose?.()}>Annulla</button>
        <button type="submit" class="btn-primary" disabled={isSaving || courses.length === 0}>
          {isSaving
            ? "Salvataggio…"
            : initialExam
              ? "Salva Appello"
              : mode === "itinere_pair"
                ? "Crea Coppia Parziali"
                : "Pianifica Appello"}
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
    max-width: 36rem;
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
    padding: 1rem 1.35rem;
    display: flex;
    flex-direction: column;
    gap: 0.8rem;
  }

  .mode-switch-bar {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.25rem;
    padding: 0.18rem;
    border-radius: var(--radius-sm);
    background: var(--surface-pill);
    border: 0.5px solid var(--hairline-border);
  }

  .mode-btn {
    height: 1.85rem;
    border: none;
    border-radius: var(--radius-xs);
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.72rem;
    font-weight: 600;
    cursor: pointer;
  }

  .mode-btn.active {
    background: var(--segment-active-bg);
    color: var(--text-primary);
  }

  .itinere-pair-box {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
    padding: 0.75rem;
    border-radius: var(--radius-md);
    background: var(--surface-card);
    border: 0.5px solid var(--hairline-border);
  }

  .pair-col {
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
  }

  .pair-badge {
    font-size: 0.62rem;
    font-weight: 700;
    letter-spacing: 0.04em;
    color: var(--accent-primary);
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

  .delete-group,
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
