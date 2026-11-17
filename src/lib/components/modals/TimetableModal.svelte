<script lang="ts">
  import { CalendarCheck, ImagePlus, Plus, Sparkles, Trash2, X } from "lucide-svelte";
  import { ipc } from "../../api/ipc";
  import { navStore } from "../../stores/navigation.svelte";
  import { sessionStore } from "../../stores/session.svelte";
  import type { ExtractedLessonItem } from "../../types/domain";

  let snap = $derived(sessionStore.activeSnapshot);

  let mode = $state<"list" | "scan">("list");
  let promptDraft = $state("");
  let isAnalyzing = $state(false);
  let extractedPreview = $state<ExtractedLessonItem[]>([]);
  let fileInputEl = $state<HTMLInputElement | null>(null);

  // Manual add fields
  let mDay = $state(1);
  let mStart = $state("08:15");
  let mEnd = $state("10:15");
  let mCourse = $state("");
  let mRoom = $state("");
  let mProf = $state("");

  const DAY_NAMES = ["", "Lunedì", "Martedì", "Mercoledì", "Giovedì", "Venerdì", "Sabato", "Domenica"];

  async function addManualLesson() {
    if (!snap || !mCourse.trim()) return;
    await sessionStore.upsertWeeklyLesson({
      sessionId: snap.session.id,
      courseName: mCourse.trim(),
      dayOfWeek: Number(mDay),
      startTime: mStart,
      endTime: mEnd,
      classroom: mRoom.trim(),
      professor: mProf.trim()
    });
    mCourse = "";
    mRoom = "";
    mProf = "";
  }

  async function analyzeImageFile(file: File) {
    isAnalyzing = true;
    try {
      const mime = file.type || "image/png";
      const dataUrl: string = await new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => resolve(String(reader.result || ""));
        reader.onerror = () => reject(new Error("Lettura immagine fallita"));
        reader.readAsDataURL(file);
      });
      extractedPreview = await ipc.parseTimetableImage(dataUrl, mime);
      mode = "scan";
    } catch (err) {
      navStore.showToast(`Errore analisi orario: ${String(err)}`, "error");
    } finally {
      isAnalyzing = false;
    }
  }

  async function analyzeTextPrompt() {
    if (!promptDraft.trim()) return;
    isAnalyzing = true;
    try {
      extractedPreview = await ipc.parseTimetablePrompt(promptDraft.trim());
    } catch (err) {
      navStore.showToast(`Errore analisi testo: ${String(err)}`, "error");
    } finally {
      isAnalyzing = false;
    }
  }

  function handlePaste(e: ClipboardEvent) {
    const item = Array.from(e.clipboardData?.items ?? []).find((i) => i.type.startsWith("image/"));
    const file = item?.getAsFile();
    if (file) {
      void analyzeImageFile(file);
    }
  }

  async function confirmExtractedLessons() {
    if (!snap || extractedPreview.length === 0) return;
    for (const l of extractedPreview) {
      await sessionStore.upsertWeeklyLesson({
        sessionId: snap.session.id,
        courseName: l.courseName,
        dayOfWeek: l.dayOfWeek,
        startTime: l.startTime,
        endTime: l.endTime,
        classroom: l.classroom,
        professor: l.professor
      });
    }
    extractedPreview = [];
    mode = "list";
    navStore.showToast("Lezioni importate nell'orario settimanale!", "success");
  }

  async function projectToCalendar() {
    if (!snap) return;
    const count = await sessionStore.projectLessonsToCalendar(snap.session.startDate, snap.session.endDate);
    navStore.showToast(`Proiettate ${count} lezioni nel calendario della sessione`, "success");
  }
</script>

{#if navStore.isTimetableModalOpen && snap}
  <div class="modal-backdrop" role="dialog" aria-modal="true" aria-label="Orario Lezioni" onpaste={handlePaste}>
    <div class="bento-card tt-modal bento-enter">
      <header class="tt-head">
        <div>
          <h2>Orario Lezioni Settimanale</h2>
          <p>Inserisci a mano oppure scansiona una foto/testo con AI Locale o Agente CLI</p>
        </div>
        <button type="button" class="close-btn" onclick={() => (navStore.isTimetableModalOpen = false)}>
          <X size={18} />
        </button>
      </header>

      <div class="tt-tabs">
        <button type="button" class="pill-filter" class:active={mode === "list"} onclick={() => (mode = "list")}>
          Orario ({snap.weeklyLessons.length})
        </button>
        <button type="button" class="pill-filter" class:active={mode === "scan"} onclick={() => (mode = "scan")}>
          <Sparkles size={13} /> Scansione Foto / Testo AI
        </button>
      </div>

      <div class="tt-body">
        {#if mode === "scan"}
          <div class="scan-panel">
            <input
              type="file"
              accept="image/*"
              class="hidden-file"
              bind:this={fileInputEl}
              onchange={(e) => {
                const f = (e.currentTarget as HTMLInputElement).files?.[0];
                if (f) void analyzeImageFile(f);
              }}
            />
            <div class="scan-actions">
              <button type="button" class="pill-btn pill-btn-primary" disabled={isAnalyzing} onclick={() => fileInputEl?.click()}>
                <ImagePlus size={15} />
                {isAnalyzing ? "Analisi in corso..." : "Carica foto orario (o Ctrl+V)"}
              </button>
            </div>
            <div class="prompt-row">
              <input
                type="text"
                class="soft-input"
                placeholder="Oppure descrivi/incolla l'orario testuale (es. MAM mercoledì 8:15-10:15 aula BL28)..."
                bind:value={promptDraft}
              />
              <button type="button" class="pill-btn pill-btn-soft" disabled={isAnalyzing} onclick={analyzeTextPrompt}>
                Analizza testo
              </button>
            </div>

            {#if extractedPreview.length > 0}
              <div class="preview-table">
                <h4>Anteprima modificabile ({extractedPreview.length} lezioni rilevate)</h4>
                {#each extractedPreview as item, idx (idx)}
                  <div class="prev-row">
                    <select class="soft-input" bind:value={item.dayOfWeek}>
                      {#each [1, 2, 3, 4, 5, 6, 7] as d (d)}
                        <option value={d}>{DAY_NAMES[d]}</option>
                      {/each}
                    </select>
                    <input type="text" class="soft-input sm" bind:value={item.startTime} />
                    <input type="text" class="soft-input sm" bind:value={item.endTime} />
                    <input type="text" class="soft-input" bind:value={item.courseName} />
                    <input type="text" class="soft-input" placeholder="Aula" bind:value={item.classroom} />
                    <button
                      type="button"
                      class="del-btn"
                      onclick={() => (extractedPreview = extractedPreview.filter((_, i) => i !== idx))}
                    >
                      <Trash2 size={14} />
                    </button>
                  </div>
                {/each}
                <button type="button" class="pill-btn pill-btn-primary self-end" onclick={confirmExtractedLessons}>
                  Conferma e Aggiungi all'Orario
                </button>
              </div>
            {/if}
          </div>
        {:else}
          <div class="manual-row">
            <select class="soft-input" bind:value={mDay}>
              {#each [1, 2, 3, 4, 5, 6, 7] as d (d)}
                <option value={d}>{DAY_NAMES[d]}</option>
              {/each}
            </select>
            <input type="time" class="soft-input" bind:value={mStart} />
            <input type="time" class="soft-input" bind:value={mEnd} />
            <input type="text" class="soft-input" placeholder="Materia *" bind:value={mCourse} />
            <input type="text" class="soft-input" placeholder="Aula" bind:value={mRoom} />
            <button type="button" class="pill-btn pill-btn-primary" onclick={addManualLesson}>
              <Plus size={14} /> Aggiungi
            </button>
          </div>

          <div class="lessons-list">
            {#each snap.weeklyLessons as l (l.id)}
              <div class="lesson-row">
                <span class="day-pill">{DAY_NAMES[l.dayOfWeek]}</span>
                <strong>{l.startTime} - {l.endTime}</strong>
                <span class="l-name">{l.courseName}</span>
                <span class="l-meta">{l.classroom} {l.professor ? `· ${l.professor}` : ""}</span>
                <button type="button" class="del-btn" onclick={() => sessionStore.deleteWeeklyLesson(l.id)}>
                  <Trash2 size={14} />
                </button>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <footer class="tt-foot">
        <button type="button" class="pill-btn pill-btn-soft" onclick={projectToCalendar}>
          <CalendarCheck size={15} />
          Proietta sul Calendario
        </button>
        <button type="button" class="pill-btn pill-btn-primary" onclick={() => (navStore.isTimetableModalOpen = false)}>
          Fatto
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

  .tt-modal {
    width: min(820px, 100%);
    max-height: 88vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .tt-head {
    padding: 20px 24px 12px;
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
  }

  .tt-head h2 {
    margin: 0;
    font-size: 19px;
  }

  .tt-head p {
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

  .tt-tabs {
    padding: 0 24px 12px;
    display: flex;
    gap: 8px;
    border-bottom: 1px solid var(--border-subtle);
  }

  .tt-body {
    padding: 18px 24px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .hidden-file {
    display: none;
  }

  .scan-panel {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .prompt-row,
  .manual-row {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .soft-input {
    padding: 8px 10px;
    border-radius: 10px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface-muted);
    color: var(--text-primary);
    font-size: 12.5px;
    flex: 1;
    min-width: 90px;
  }

  .soft-input.sm {
    max-width: 78px;
  }

  .preview-table {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    border-radius: 14px;
    background: var(--bg-surface-muted);
  }

  .preview-table h4 {
    margin: 0;
    font-size: 13px;
  }

  .prev-row {
    display: flex;
    gap: 6px;
    align-items: center;
  }

  .lessons-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .lesson-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    border-radius: 12px;
    background: var(--bg-surface-muted);
    font-size: 13px;
  }

  .day-pill {
    padding: 2px 8px;
    border-radius: 999px;
    background: var(--accent-soft);
    color: var(--accent);
    font-size: 11px;
    font-weight: 700;
  }

  .l-name {
    font-weight: 700;
    flex: 1;
  }

  .l-meta {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .del-btn {
    color: var(--text-muted);
    padding: 4px;
  }

  .del-btn:hover {
    color: var(--danger);
  }

  .tt-foot {
    padding: 14px 24px 20px;
    border-top: 1px solid var(--border-subtle);
    display: flex;
    justify-content: space-between;
  }

  .self-end {
    align-self: flex-end;
  }
</style>
