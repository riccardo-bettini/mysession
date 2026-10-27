<script lang="ts">
  import { sessionStore } from "../../stores/session.svelte";
  import type { Course, StudyUnit, StudyUnitDraft } from "../../types/domain";
  import { UNIFIED_STANDARD_PALETTE } from "../../utils/color";
  import ColorPickerField from "../common/ColorPickerField.svelte";

  interface Props {
    initialCourse?: Course | null;
    existingUnits?: StudyUnit[];
    onClose?: () => void;
    onSaved?: () => void;
  }

  let {
    initialCourse = null,
    existingUnits = [],
    onClose,
    onSaved
  }: Props = $props();

  let sigla = $state<string>("");
  let name = $state<string>("");
  let code = $state<string>("");
  let cfu = $state<number>(9);
  let professor = $state<string>("");
  let difficultyWeight = $state<number>(1.0);
  let colorHex = $state<string>(UNIFIED_STANDARD_PALETTE[0]);
  let notes = $state<string>("");
  let units = $state<StudyUnitDraft[]>([]);
  let isSaving = $state<boolean>(false);

  $effect(() => {
    if (initialCourse) {
      sigla = initialCourse.sigla;
      name = initialCourse.name;
      code = initialCourse.code;
      cfu = initialCourse.cfu;
      professor = initialCourse.professor;
      difficultyWeight = initialCourse.difficultyWeight || 1.0;
      colorHex = initialCourse.colorHex;
      notes = initialCourse.notes;
      units = existingUnits
        .filter((u) => u.courseId === initialCourse.id)
        .map((u) => ({
          id: u.id,
          unitNumber: u.unitNumber,
          title: u.title,
          description: u.description,
          estimatedHours: u.estimatedHours,
          isCompleted: u.isCompleted,
          assignedDate: u.assignedDate
        }));
    } else {
      const usedCount = sessionStore.courses.length;
      sigla = "";
      name = "";
      code = "";
      cfu = 9;
      professor = "";
      difficultyWeight = 1.0;
      colorHex = UNIFIED_STANDARD_PALETTE[usedCount % UNIFIED_STANDARD_PALETTE.length];
      notes = "";
      units = [
        { unitNumber: 1, title: "Modulo 1 — Fondamenti", estimatedHours: 12, isCompleted: false },
        { unitNumber: 2, title: "Modulo 2 — Approfondimenti", estimatedHours: 15, isCompleted: false },
        { unitNumber: 3, title: "Modulo 3 — Esercitazioni & Temi d'Esame", estimatedHours: 18, isCompleted: false }
      ];
    }
  });

  function addUnitRow() {
    const nextNum = units.length + 1;
    units = [
      ...units,
      {
        unitNumber: nextNum,
        title: `Unità Didattica ${nextNum}`,
        estimatedHours: 10,
        isCompleted: false
      }
    ];
  }

  function generateUnitsFromCfu() {
    const count = Math.max(2, Math.min(12, Math.round(cfu / 2)));
    const hoursPerUnit = Math.max(6, Math.round((cfu * 8) / count));
    const generated: StudyUnitDraft[] = [];
    for (let i = 1; i <= count; i++) {
      generated.push({
        unitNumber: i,
        title: i === count ? `Unità ${i} — Simulazioni & Ripasso Finale` : `Unità Didattica ${i}`,
        estimatedHours: hoursPerUnit,
        isCompleted: false
      });
    }
    units = generated;
  }

  function removeUnitRow(idx: number) {
    units = units
      .filter((_, i) => i !== idx)
      .map((u, i) => ({ ...u, unitNumber: i + 1 }));
  }

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    const sessionId = sessionStore.activeSession?.id;
    if (!sessionId || !sigla.trim() || !name.trim()) return;

    isSaving = true;
    try {
      await sessionStore.upsertCourse({
        id: initialCourse?.id ?? null,
        sessionId,
        sigla: sigla.trim().toUpperCase(),
        name: name.trim(),
        code: code.trim(),
        cfu: Math.max(1, Math.min(60, Number(cfu) || 6)),
        professor: professor.trim(),
        colorHex,
        difficultyWeight: Number(difficultyWeight) || 1.0,
        targetHoursOverride: null,
        notes: notes.trim() || null,
        units
      });
      onSaved?.();
      onClose?.();
    } finally {
      isSaving = false;
    }
  }

  async function handleDelete() {
    if (!initialCourse) return;
    isSaving = true;
    try {
      await sessionStore.deleteCourse(initialCourse.id);
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
  aria-label={initialCourse ? "Modifica Corso" : "Nuovo Corso"}
  tabindex="-1"
  onclick={(e) => {
    if (e.target === e.currentTarget) onClose?.();
  }}
>
  <form class="modal-sheet wide" onsubmit={handleSubmit}>
    <header class="sheet-header">
      <div>
        <span class="sheet-eyebrow">DOMINIO ACCADEMICO & UNITÀ DIDATTICHE</span>
        <h2>{initialCourse ? `Modifica ${initialCourse.sigla} — ${initialCourse.name}` : "Nuova Materia / Corso"}</h2>
      </div>
      <button type="button" class="sheet-close-btn" onclick={() => onClose?.()}>✕</button>
    </header>

    <div class="sheet-body">
      <div class="field-row three-cols">
        <label class="form-field">
          <span>Sigla Breve *</span>
          <input type="text" required maxlength="8" bind:value={sigla} placeholder="es. MAM" />
        </label>

        <label class="form-field span-2">
          <span>Nome Completo Insegnamento *</span>
          <input type="text" required bind:value={name} placeholder="es. Meccanica Applicata alle Macchine" />
        </label>
      </div>

      <div class="field-row four-cols">
        <label class="form-field">
          <span>CFU *</span>
          <input type="number" min="1" max="45" required bind:value={cfu} />
        </label>

        <label class="form-field">
          <span>Peso Difficoltà</span>
          <select bind:value={difficultyWeight}>
            <option value={0.85}>0.85× (Agevole)</option>
            <option value={1.0}>1.00× (Standard)</option>
            <option value={1.2}>1.20× (Impegnativo)</option>
            <option value={1.45}>1.45× (Critico / Scoglio)</option>
          </select>
        </label>

        <label class="form-field">
          <span>Codice Esame</span>
          <input type="text" bind:value={code} placeholder="es. 054828" />
        </label>

        <label class="form-field">
          <span>Docente Titolare</span>
          <input type="text" bind:value={professor} placeholder="es. Prof. Rossi" />
        </label>
      </div>

      <!-- Palette Colori Unificata + Slider 360° + WCAG Check -->
      <ColorPickerField
        bind:value={colorHex}
        label="Colore Corso (Palette Unificata + Contrasto WCAG AA/AAA)"
        previewText={sigla.trim().toUpperCase() || "CORSO"}
      />

      <!-- Unità Didattiche / Moduli del Corso -->
      <div class="units-builder-box">
        <div class="ub-header">
          <span>UNITÀ DIDATTICHE / MODULI ({units.length})</span>
          <div class="ub-actions">
            <button type="button" class="ub-btn" onclick={generateUnitsFromCfu}>
              Auto-genera da {cfu} CFU
            </button>
            <button type="button" class="ub-btn primary" onclick={addUnitRow}>
              + Aggiungi Unità
            </button>
          </div>
        </div>

        <div class="units-list">
          {#each units as unit, idx (idx)}
            <div class="unit-draft-row">
              <span class="unit-num">#{idx + 1}</span>
              <input
                type="text"
                class="unit-title-input"
                placeholder="Titolo unità o argomento…"
                bind:value={unit.title}
              />
              <label class="unit-hours-wrap">
                <input
                  type="number"
                  min="1"
                  max="120"
                  step="1"
                  bind:value={unit.estimatedHours}
                />
                <span>h</span>
              </label>
              <button
                type="button"
                class="unit-del-btn"
                title="Rimuovi unità"
                onclick={() => removeUnitRow(idx)}
              >
                ✕
              </button>
            </div>
          {/each}
        </div>
      </div>
    </div>

    <footer class="sheet-footer">
      {#if initialCourse}
        <button type="button" class="btn-danger" disabled={isSaving} onclick={handleDelete}>
          Elimina Corso
        </button>
      {:else}
        <span></span>
      {/if}

      <div class="footer-right">
        <button type="button" class="btn-secondary" onclick={() => onClose?.()}>Annulla</button>
        <button type="submit" class="btn-primary" disabled={isSaving}>
          {isSaving ? "Salvataggio…" : initialCourse ? "Salva Corso" : "Crea Corso"}
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

  .modal-sheet.wide {
    width: 100%;
    max-width: 42rem;
    max-height: 90vh;
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
    padding: 1rem 1.35rem;
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
  }

  .field-row.three-cols {
    display: grid;
    grid-template-columns: 110px 1fr 1fr;
    gap: 0.7rem;
  }

  .field-row.four-cols {
    display: grid;
    grid-template-columns: 80px 1.2fr 1fr 1.2fr;
    gap: 0.7rem;
  }

  .span-2 {
    grid-column: span 2;
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
  .form-field select {
    height: 2.05rem;
    padding: 0 0.65rem;
    border-radius: var(--radius-sm);
    border: 0.5px solid var(--hairline-border-strong);
    background: var(--surface-card);
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: 0.8rem;
  }

  /* Units Builder */
  .units-builder-box {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.75rem;
    border-radius: var(--radius-md);
    background: var(--surface-card);
    border: 0.5px solid var(--hairline-border);
  }

  .ub-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 0.66rem;
    font-weight: 700;
    letter-spacing: 0.05em;
    color: var(--text-tertiary);
  }

  .ub-actions {
    display: flex;
    gap: 0.35rem;
  }

  .ub-btn {
    padding: 0.22rem 0.55rem;
    border-radius: var(--radius-xs);
    border: 0.5px solid var(--hairline-border);
    background: var(--surface-pill);
    color: var(--text-primary);
    font-size: 0.68rem;
    font-weight: 600;
    cursor: pointer;
  }

  .ub-btn.primary {
    background: var(--accent-soft);
    color: var(--accent-primary);
    border-color: transparent;
  }

  .units-list {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    max-height: 11.5rem;
    overflow-y: auto;
  }

  .unit-draft-row {
    display: flex;
    align-items: center;
    gap: 0.45rem;
  }

  .unit-num {
    width: 1.8rem;
    font-size: 0.7rem;
    font-weight: 700;
    color: var(--text-tertiary);
  }

  .unit-title-input {
    flex: 1;
    height: 1.85rem;
    padding: 0 0.55rem;
    border-radius: var(--radius-xs);
    border: 0.5px solid var(--hairline-border);
    background: var(--surface-pill);
    color: var(--text-primary);
    font-size: 0.76rem;
  }

  .unit-hours-wrap {
    display: inline-flex;
    align-items: center;
    gap: 0.2rem;
    font-size: 0.7rem;
    color: var(--text-secondary);
  }

  .unit-hours-wrap input {
    width: 3.4rem;
    height: 1.85rem;
    padding: 0 0.35rem;
    border-radius: var(--radius-xs);
    border: 0.5px solid var(--hairline-border);
    background: var(--surface-pill);
    color: var(--text-primary);
    text-align: center;
    font-size: 0.75rem;
  }

  .unit-del-btn {
    border: none;
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
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
    padding: 0 0.95rem;
    border-radius: var(--radius-sm);
    font-size: 0.78rem;
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
