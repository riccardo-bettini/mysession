<script lang="ts">
  import { themeStore } from "../../stores/theme.svelte";
  import type { ConflictItem, Course, Exam } from "../../types/domain";
  import {
    diffDaysIso,
    formatExamRoundLabel,
    formatItalianDateLong
  } from "../../utils/calendar";
  import { generateCourseColorTokens } from "../../utils/color";

  interface Props {
    exams: Exam[];
    conflicts: ConflictItem[];
    courseById: Map<string, Course>;
    selectedCourseId?: string | null;
    todayIso: string;
    onSelectDay?: (isoDate: string) => void;
    onSelectExam?: (exam: Exam, e: MouseEvent) => void;
    onOpenNewExam?: () => void;
  }

  let {
    exams = [],
    conflicts = [],
    courseById,
    selectedCourseId = null,
    todayIso,
    onSelectDay,
    onSelectExam,
    onOpenNewExam
  }: Props = $props();

  let groupMode = $state<"chronological" | "by_round">("chronological");
  let isDark = $derived(themeStore.effectiveTheme === "dark");

  let filteredExams = $derived.by(() => {
    let list = exams.filter((e) => {
      const c = courseById.get(e.courseId);
      if (c && c.isVisible === false) return false;
      return true;
    });
    if (selectedCourseId) {
      list = list.filter((e) => e.courseId === selectedCourseId);
    }
    return list.sort((a, b) => a.date.localeCompare(b.date));
  });

  let conflictsByExamId = $derived.by(() => {
    const map = new Map<string, ConflictItem[]>();
    for (const c of conflicts) {
      for (const exId of c.examIds) {
        const list = map.get(exId) ?? [];
        list.push(c);
        map.set(exId, list);
      }
    }
    return map;
  });

  let groupedByRound = $derived.by(() => {
    const buckets: { key: string; title: string; items: Exam[] }[] = [
      { key: "itinere", title: "Prove in Itinere (Parziali 1 & 2)", items: [] },
      { key: "1", title: "1° Giro d'Appello", items: [] },
      { key: "2", title: "2° Giro d'Appello", items: [] },
      { key: "3", title: "3° Giro / Appelli Straordinari", items: [] }
    ];

    for (const ex of filteredExams) {
      if (ex.round.startsWith("itinere")) {
        buckets[0].items.push(ex);
      } else if (ex.round.startsWith("1")) {
        buckets[1].items.push(ex);
      } else if (ex.round.startsWith("2")) {
        buckets[2].items.push(ex);
      } else {
        buckets[3].items.push(ex);
      }
    }
    return buckets.filter((b) => b.items.length > 0);
  });
</script>

<div class="agenda-view-shell" aria-label="Vista Agenda Appelli">
  <header class="agenda-header">
    <div>
      <h2 class="agenda-title">Agenda Appelli & Prove in Itinere</h2>
      <p class="agenda-subtitle">
        {filteredExams.length} {filteredExams.length === 1 ? "appello pianificato" : "appelli pianificati"}
        {#if conflicts.length > 0}
          · <span class="conflict-text">{conflicts.length} conflitti rilevati dal Conflict Engine</span>
        {/if}
      </p>
    </div>

    <div class="agenda-toolbar-right">
      <div class="group-segmented">
        <button
          type="button"
          class="group-btn pressable"
          class:active={groupMode === "chronological"}
          onclick={() => (groupMode = "chronological")}
        >
          Cronologico
        </button>
        <button
          type="button"
          class="group-btn pressable"
          class:active={groupMode === "by_round"}
          onclick={() => (groupMode = "by_round")}
        >
          Per Giro d'Appello
        </button>
      </div>

      <button type="button" class="new-exam-btn pressable" onclick={() => onOpenNewExam?.()}>
        + Nuovo Appello
      </button>
    </div>
  </header>

  <div class="agenda-scroll-body">
    {#if filteredExams.length === 0}
      <div class="agenda-empty-card">
        <h3>Nessun appello in agenda</h3>
        <p>Aggiungi gli appelli d'esame o le coppie di prove in itinere per visualizzare la timeline e l'analisi dei conflitti.</p>
        <button type="button" class="new-exam-btn pressable" onclick={() => onOpenNewExam?.()}>
          + Pianifica il primo appello
        </button>
      </div>
    {:else if groupMode === "chronological"}
      <div class="agenda-cards-list">
        {#each filteredExams as exam (exam.id)}
          {@const course = courseById.get(exam.courseId)}
          {@const palette = generateCourseColorTokens(course?.colorHex ?? "#007AFF", isDark)}
          {@const deltaDays = diffDaysIso(todayIso, exam.date)}
          {@const examConflicts = conflictsByExamId.get(exam.id) ?? []}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <div
            role="button"
            tabindex="0"
            class="agenda-exam-row"
            style:--row-accent={palette.solid}
            onclick={(e) => {
              onSelectDay?.(exam.date);
              onSelectExam?.(exam, e);
            }}
          >
            <div class="row-left-accent" style:background={palette.solid}></div>

            <div class="row-main-info">
              <div class="row-top-badges">
                <span class="course-pill" style:background={palette.solid} style:color={palette.badgeText}>
                  {course?.sigla ?? "ESAME"}
                </span>
                <span class="round-pill">{formatExamRoundLabel(exam.round)}</span>
                <span class="type-pill">{exam.examType}</span>
                {#if course}
                  <span class="cfu-pill">{course.cfu} CFU</span>
                {/if}
              </div>

              <h3 class="row-course-title">{course?.name ?? exam.title}</h3>
              <div class="row-date-line">
                <span>📅 {formatItalianDateLong(exam.date)}</span>
                {#if exam.timeSlot}
                  <span>· 🕒 {exam.timeSlot}</span>
                {/if}
                {#if exam.classroom}
                  <span>· 📍 {exam.classroom}</span>
                {/if}
              </div>

              {#if examConflicts.length > 0}
                <div class="row-conflicts-list">
                  {#each examConflicts as cf (cf.id)}
                    <div class="conflict-inline-alert" class:critical={cf.severity === "critical"}>
                      <span>⚠ {cf.title}: {cf.description}</span>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>

            <div class="row-countdown-col">
              {#if deltaDays > 0}
                <span class="cd-num">−{deltaDays}</span>
                <span class="cd-lbl">{deltaDays === 1 ? "giorno" : "giorni"}</span>
              {:else if deltaDays === 0}
                <span class="cd-today">OGGI</span>
              {:else}
                <span class="cd-past">Svolto</span>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {:else}
      {#each groupedByRound as bucket (bucket.key)}
        <section class="round-bucket-section">
          <h3 class="bucket-heading">{bucket.title} ({bucket.items.length})</h3>
          <div class="agenda-cards-list">
            {#each bucket.items as exam (exam.id)}
              {@const course = courseById.get(exam.courseId)}
              {@const palette = generateCourseColorTokens(course?.colorHex ?? "#007AFF", isDark)}
              {@const deltaDays = diffDaysIso(todayIso, exam.date)}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <div
                role="button"
                tabindex="0"
                class="agenda-exam-row"
                onclick={(e) => {
                  onSelectDay?.(exam.date);
                  onSelectExam?.(exam, e);
                }}
              >
                <div class="row-left-accent" style:background={palette.solid}></div>
                <div class="row-main-info">
                  <div class="row-top-badges">
                    <span class="course-pill" style:background={palette.solid} style:color={palette.badgeText}>
                      {course?.sigla ?? "ESAME"}
                    </span>
                    <span class="round-pill">{formatExamRoundLabel(exam.round)}</span>
                    {#if course}
                      <span class="cfu-pill">{course.cfu} CFU</span>
                    {/if}
                  </div>
                  <h4 class="row-course-title">{course?.name ?? exam.title}</h4>
                  <div class="row-date-line">
                    <span>📅 {formatItalianDateLong(exam.date)}</span>
                    {#if exam.timeSlot}
                      <span>· 🕒 {exam.timeSlot}</span>
                    {/if}
                  </div>
                </div>
                <div class="row-countdown-col">
                  {#if deltaDays >= 0}
                    <span class="cd-num">−{deltaDays}gg</span>
                  {:else}
                    <span class="cd-past">Svolto</span>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        </section>
      {/each}
    {/if}
  </div>
</div>

<style>
  .agenda-view-shell {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 0.95rem 1.25rem 1.25rem;
    overflow: hidden;
  }

  .agenda-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 0.95rem;
    flex-shrink: 0;
  }

  .agenda-title {
    margin: 0;
    font-family: var(--font-display);
    font-size: 1.4rem;
    font-weight: 800;
    letter-spacing: var(--tracking-tight);
    color: var(--text-primary);
  }

  .agenda-subtitle {
    margin: 0.2rem 0 0;
    font-size: 0.78rem;
    letter-spacing: var(--tracking-normal);
    color: var(--text-secondary);
  }

  .conflict-text {
    color: var(--conflict-critical);
    font-weight: 600;
  }

  .agenda-toolbar-right {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .group-segmented {
    display: inline-flex;
    padding: 0.15rem;
    background: var(--surface-pill);
    border: 0.5px solid var(--hairline-border);
    box-shadow: var(--shadow-pill);
    border-radius: var(--radius-sm);
    gap: 0.12rem;
  }

  .group-btn {
    height: 1.75rem;
    padding: 0 0.7rem;
    border: none;
    border-radius: var(--radius-xs);
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.74rem;
    font-weight: 600;
    letter-spacing: var(--tracking-normal);
    cursor: pointer;
  }

  .group-btn.active {
    background: var(--segment-active-bg);
    color: var(--text-primary);
    box-shadow: var(--shadow-segment-active);
  }

  .new-exam-btn {
    height: 2.05rem;
    padding: 0 0.9rem;
    border-radius: var(--radius-sm);
    border: none;
    background: var(--accent-primary);
    color: #fff;
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: var(--tracking-normal);
    cursor: pointer;
    box-shadow: 0 1px 3px rgba(10, 132, 255, 0.3);
  }

  .agenda-scroll-body {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding-right: 0.25rem;
  }

  .agenda-empty-card {
    margin: auto;
    max-width: 26rem;
    padding: 2.2rem;
    border-radius: var(--radius-xl);
    background: var(--surface-elevated);
    border: 0.5px solid var(--hairline-border-strong);
    box-shadow: var(--shadow-pill);
    text-align: center;
  }

  .agenda-empty-card h3 {
    margin: 0 0 0.45rem;
    font-size: 1.1rem;
    font-weight: 700;
  }

  .agenda-empty-card p {
    margin: 0 0 1.2rem;
    font-size: 0.82rem;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .agenda-cards-list {
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
  }

  .agenda-exam-row {
    display: flex;
    align-items: stretch;
    border-radius: var(--radius-lg);
    background: var(--surface-card);
    border: 0.5px solid var(--hairline-border);
    box-shadow: var(--shadow-pill);
    overflow: hidden;
    cursor: pointer;
    will-change: transform;
    transition: transform 120ms var(--spring-press), border-color 130ms ease, background-color 130ms ease;
  }

  .agenda-exam-row:hover {
    transform: translateY(-1px);
    border-color: var(--hairline-border-strong);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  }

  .agenda-exam-row:active {
    transform: scale(0.985);
  }

  .row-left-accent {
    width: 5px;
    flex-shrink: 0;
  }

  .row-main-info {
    flex: 1;
    padding: 0.8rem 1.1rem;
    display: flex;
    flex-direction: column;
    gap: 0.32rem;
  }

  .row-top-badges {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-wrap: wrap;
  }

  .course-pill {
    font-size: 0.66rem;
    font-weight: 800;
    padding: 0.12rem 0.45rem;
    border-radius: 0.3rem;
    letter-spacing: var(--tracking-wide);
  }

  .round-pill,
  .type-pill,
  .cfu-pill {
    font-size: 0.65rem;
    font-weight: 600;
    padding: 0.1rem 0.42rem;
    border-radius: 0.3rem;
    background: var(--surface-pill);
    border: 0.5px solid var(--hairline-border);
    color: var(--text-secondary);
  }

  .row-course-title {
    margin: 0;
    font-size: 0.98rem;
    font-weight: 700;
    letter-spacing: var(--tracking-tight);
  }

  .row-date-line {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.76rem;
    color: var(--text-secondary);
  }

  .row-conflicts-list {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    margin-top: 0.25rem;
  }

  .conflict-inline-alert {
    padding: 0.35rem 0.6rem;
    border-radius: var(--radius-xs);
    background: rgba(255, 159, 10, 0.15);
    border: 0.5px solid rgba(255, 159, 10, 0.35);
    color: var(--conflict-warning);
    font-size: 0.68rem;
    font-weight: 600;
  }

  .conflict-inline-alert.critical {
    background: rgba(255, 69, 58, 0.16);
    border-color: rgba(255, 69, 58, 0.4);
    color: var(--conflict-critical);
  }

  .row-countdown-col {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-width: 5.8rem;
    padding: 0 1rem;
    border-left: 0.5px solid var(--hairline-border);
    background: var(--surface-pill);
  }

  .cd-num {
    font-size: 1.25rem;
    font-weight: 800;
    letter-spacing: var(--tracking-tight);
    font-variant-numeric: tabular-nums;
    color: var(--accent-primary);
  }

  .cd-lbl {
    font-size: 0.62rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: var(--tracking-caps);
    color: var(--text-secondary);
  }

  .cd-today {
    font-size: 0.74rem;
    font-weight: 800;
    color: #34c759;
    letter-spacing: var(--tracking-wide);
  }

  .cd-past {
    font-size: 0.72rem;
    color: var(--text-tertiary);
  }

  .round-bucket-section {
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
  }

  .bucket-heading {
    margin: 0;
    font-size: 0.76rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: var(--tracking-caps);
    color: var(--text-tertiary);
  }
</style>
