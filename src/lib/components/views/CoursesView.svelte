<script lang="ts">
  import { ArrowLeft, Calendar, Check, GraduationCap, HelpCircle, Plus, Trash2 } from "lucide-svelte";
  import { navStore } from "../../stores/navigation.svelte";
  import { sessionStore } from "../../stores/session.svelte";
  import { themeStore } from "../../stores/theme.svelte";
  import type { TaskCategory } from "../../types/domain";
  import { formatItalianDateShort } from "../../utils/calendar";
  import { CORE_FIVE_CATEGORIES, getCategoryColor, getCategoryLabel } from "../../utils/categories";
  import ColorPickerField from "../common/ColorPickerField.svelte";

  let snap = $derived(sessionStore.activeSnapshot);

  let selectedCourse = $derived(
    snap?.courses.find((c) => c.id === navStore.selectedCourseId) ?? null
  );
  let courseMetric = $derived(
    snap?.telemetry.courseProgress.find((m) => m.courseId === navStore.selectedCourseId) ?? null
  );

  let detailTab = $state<"tasks" | "quizzes" | "units" | "exams">("tasks");
  let catFilter = $state<"all" | TaskCategory>("all");

  // New Course modal/inline drawer state
  let showAddCourse = $state(false);
  let cSigla = $state("");
  let cName = $state("");
  let cCfu = $state(6);
  let cProf = $state("");
  let cColor = $state("#7C5CFC");

  // Quick Exam add inside course detail
  let showAddExam = $state(false);
  let exDate = $state("");
  let exRound = $state("1");
  let exTime = $state("09:00");
  let exRoom = $state("");

  let courseTasks = $derived.by(() => {
    if (!snap || !selectedCourse) return [];
    const list = snap.tasks.filter((t) => t.courseId === selectedCourse.id);
    return catFilter === "all" ? list : list.filter((t) => t.category === catFilter);
  });

  let courseQuizzes = $derived(
    snap && selectedCourse ? snap.quizzes.filter((q) => q.courseId === selectedCourse.id) : []
  );
  let courseUnits = $derived(
    snap && selectedCourse ? snap.studyUnits.filter((u) => u.courseId === selectedCourse.id) : []
  );
  let courseExams = $derived(
    snap && selectedCourse ? snap.exams.filter((e) => e.courseId === selectedCourse.id) : []
  );

  async function saveNewCourse() {
    if (!snap || !cSigla.trim() || !cName.trim()) return;
    await sessionStore.upsertCourse({
      sessionId: snap.session.id,
      sigla: cSigla.trim().toUpperCase(),
      name: cName.trim(),
      cfu: Number(cCfu) || 6,
      professor: cProf.trim(),
      colorHex: cColor
    });
    cSigla = "";
    cName = "";
    cProf = "";
    showAddCourse = false;
    navStore.showToast("Corso aggiunto alla sessione", "success");
  }

  async function saveNewExam() {
    if (!snap || !selectedCourse || !exDate) return;
    await sessionStore.upsertExam({
      sessionId: snap.session.id,
      courseId: selectedCourse.id,
      date: exDate,
      round: exRound,
      timeSlot: exTime,
      classroom: exRoom,
      title: `${selectedCourse.sigla} - ${exRound}° Appello`
    });
    exDate = "";
    exRoom = "";
    showAddExam = false;
    navStore.showToast("Appello aggiunto", "success");
  }
</script>

{#if snap}
  {#if selectedCourse && courseMetric}
    <!-- Schermata dedicata al singolo corso -->
    <div class="course-detail bento-enter">
      <div class="detail-top-bar">
        <button type="button" class="pill-btn pill-btn-soft" onclick={() => (navStore.selectedCourseId = null)}>
          <ArrowLeft size={15} />
          Tutti i Corsi
        </button>
      </div>

      <section class="bento-card detail-hero" style:--c-hex={selectedCourse.colorHex}>
        <div class="dh-left">
          <div class="dh-badges">
            <span class="course-sigla-lg" style:background={selectedCourse.colorHex}>{selectedCourse.sigla}</span>
            <span class="tag-pill">{selectedCourse.cfu} CFU</span>
            {#if courseMetric.daysUntilExam !== null}
              <span class="countdown-pill">
                <Calendar size={13} />
                Esame tra {courseMetric.daysUntilExam} gg ({formatItalianDateShort(courseMetric.nextExamDate!)})
              </span>
            {/if}
          </div>
          <h1>{selectedCourse.name}</h1>
          {#if selectedCourse.professor}
            <p class="prof-line">Docente: {selectedCourse.professor}</p>
          {/if}
        </div>

        <div class="dh-metrics">
          <div class="dm-box">
            <strong>{courseMetric.completionPct}%</strong>
            <span>Completamento</span>
          </div>
          <div class="dm-box">
            <strong>{courseMetric.completedHoursTotal}/{courseMetric.plannedHoursTotal}h</strong>
            <span>Ore svolte</span>
          </div>
          <div class="dm-box">
            <strong>{courseMetric.quizzesCompleted}/{courseMetric.quizzesTotal}</strong>
            <span>Quiz{courseMetric.quizAverageScore !== null ? ` (${courseMetric.quizAverageScore}%)` : ""}</span>
          </div>
        </div>
      </section>

      <!-- Barra 5 Categorie del corso -->
      <div class="bento-card cat-breakdown-card">
        <h3>Avanzamento per Categoria</h3>
        <div class="cat-grid-5">
          {#each CORE_FIVE_CATEGORIES as cat (cat.id)}
            {@const stat = courseMetric.categoryBreakdown.find((x) => x.category === cat.id)}
            <div class="cat-col">
              <div class="cat-col-top">
                <span>{cat.label}</span>
                <strong>{stat?.completionPct ?? 0}%</strong>
              </div>
              <div class="mini-track">
                <div
                  class="mini-fill"
                  style:width="{stat?.completionPct ?? 0}%"
                  style:background={getCategoryColor(cat.id, themeStore.settings)}
                ></div>
              </div>
              <small>{stat?.completedTasks ?? 0}/{stat?.totalTasks ?? 0} task · {stat?.plannedHours ?? 0}h</small>
            </div>
          {/each}
        </div>
      </div>

      <!-- Tab Sezioni del Corso -->
      <div class="tab-bar">
        <button type="button" class="pill-filter" class:active={detailTab === "tasks"} onclick={() => (detailTab = "tasks")}>
          Task ({snap.tasks.filter((t) => t.courseId === selectedCourse.id).length})
        </button>
        <button type="button" class="pill-filter" class:active={detailTab === "quizzes"} onclick={() => (detailTab = "quizzes")}>
          Quiz ({courseQuizzes.length})
        </button>
        <button type="button" class="pill-filter" class:active={detailTab === "units"} onclick={() => (detailTab = "units")}>
          Unità ({courseUnits.length})
        </button>
        <button type="button" class="pill-filter" class:active={detailTab === "exams"} onclick={() => (detailTab = "exams")}>
          Appelli ({courseExams.length})
        </button>
      </div>

      {#if detailTab === "tasks"}
        <div class="bento-card tab-panel">
          <div class="filter-pills">
            <button type="button" class="pill-filter" class:active={catFilter === "all"} onclick={() => (catFilter = "all")}>Tutte</button>
            {#each CORE_FIVE_CATEGORIES as cat (cat.id)}
              <button type="button" class="pill-filter" class:active={catFilter === cat.id} onclick={() => (catFilter = cat.id)}>
                {cat.label}
              </button>
            {/each}
          </div>

          {#if courseTasks.length === 0}
            <p class="empty-soft">Nessuna task in questa vista.</p>
          {:else}
            <div class="list-stack">
              {#each courseTasks as t (t.id)}
                <div class="row-card" class:done={t.done}>
                  <button
                    type="button"
                    class="chk-box"
                    class:checked={t.done}
                    onclick={() => sessionStore.toggleTask(t.id, !t.done)}
                  >
                    {#if t.done}<Check size={13} />{/if}
                  </button>
                  <div class="row-info">
                    <div class="row-meta">
                      <span class="cat-badge" style:background={getCategoryColor(t.category, themeStore.settings)}>
                        {getCategoryLabel(t.category)}
                      </span>
                      <span>{formatItalianDateShort(t.date)} · {t.plannedHours}h</span>
                    </div>
                    <strong>{t.title}</strong>
                    <small>{[t.chapter, t.topic, t.pages].filter(Boolean).join(" · ")}</small>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {:else if detailTab === "quizzes"}
        <div class="bento-card tab-panel">
          {#if courseQuizzes.length === 0}
            <p class="empty-soft">Nessun quiz generato per questo corso.</p>
          {:else}
            <div class="quiz-cards-2">
              {#each courseQuizzes as q (q.id)}
                <div class="row-card">
                  <div class="row-info">
                    <div class="row-meta">
                      <span>{formatItalianDateShort(q.date)} · {q.questions.length} domande</span>
                      {#if q.latestAttempt}
                        <span class="score-pill">{q.latestAttempt.scorePct}%</span>
                      {/if}
                    </div>
                    <strong>{q.title}</strong>
                    <small>{q.topicSummary}</small>
                  </div>
                  <button type="button" class="pill-btn pill-btn-primary btn-xs" onclick={() => (navStore.activeQuizId = q.id)}>
                    {q.latestAttempt ? "Ripeti" : "Avvia"}
                  </button>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {:else if detailTab === "units"}
        <div class="bento-card tab-panel">
          {#if courseUnits.length === 0}
            <p class="empty-soft">Nessuna unità didattica caricata.</p>
          {:else}
            <div class="list-stack">
              {#each courseUnits as u (u.id)}
                <div class="row-card" class:done={u.isCompleted}>
                  <button
                    type="button"
                    class="chk-box"
                    class:checked={u.isCompleted}
                    onclick={() => sessionStore.toggleStudyUnit(u.id, !u.isCompleted)}
                  >
                    {#if u.isCompleted}<Check size={13} />{/if}
                  </button>
                  <div class="row-info">
                    <strong>{u.chapter ? `${u.chapter}: ` : ""}{u.title}</strong>
                    <small>{u.description || u.source} · {u.estimatedHours}h stimate</small>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {:else if detailTab === "exams"}
        <div class="bento-card tab-panel">
          <div class="panel-head-row">
            <h3>Appelli del Corso</h3>
            <button type="button" class="pill-btn pill-btn-primary btn-xs" onclick={() => (showAddExam = !showAddExam)}>
              <Plus size={14} /> Aggiungi Appello
            </button>
          </div>
          {#if showAddExam}
            <div class="inline-form">
              <input type="date" class="soft-input" bind:value={exDate} />
              <input type="text" class="soft-input" placeholder="Appello (es. 1)" bind:value={exRound} />
              <input type="time" class="soft-input" bind:value={exTime} />
              <input type="text" class="soft-input" placeholder="Aula" bind:value={exRoom} />
              <button type="button" class="pill-btn pill-btn-primary btn-xs" onclick={saveNewExam}>Salva</button>
            </div>
          {/if}
          <div class="list-stack">
            {#each courseExams as ex (ex.id)}
              <div class="row-card">
                <div class="row-info">
                  <strong>{ex.title}</strong>
                  <small>{ex.date} {ex.timeSlot} {ex.classroom ? `· ${ex.classroom}` : ""} · Stato: {ex.status}</small>
                </div>
                <button type="button" class="del-icon" onclick={() => sessionStore.deleteExam(ex.id)}>
                  <Trash2 size={14} />
                </button>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  {:else}
    <!-- Griglia elenco Corsi -->
    <div class="courses-wrap bento-enter">
      <div class="page-head">
        <div>
          <h1>Corsi della Sessione</h1>
          <p>{snap.courses.length} materie · {snap.telemetry.totalCfu} CFU attivi</p>
        </div>
        <button type="button" class="pill-btn pill-btn-primary" onclick={() => (showAddCourse = !showAddCourse)}>
          <Plus size={16} />
          Nuovo Corso
        </button>
      </div>

      {#if showAddCourse}
        <div class="bento-card add-course-panel">
          <div class="form-2">
            <input type="text" class="soft-input" placeholder="Sigla (es. MAM) *" bind:value={cSigla} />
            <input type="text" class="soft-input" placeholder="Nome materia *" bind:value={cName} />
            <input type="number" class="soft-input" placeholder="CFU" min="1" max="30" bind:value={cCfu} />
            <input type="text" class="soft-input" placeholder="Docente" bind:value={cProf} />
          </div>
          <ColorPickerField bind:value={cColor} previewText={cSigla || "CORSO"} />
          <div class="form-actions">
            <button type="button" class="pill-btn pill-btn-soft" onclick={() => (showAddCourse = false)}>Annulla</button>
            <button type="button" class="pill-btn pill-btn-primary" onclick={saveNewCourse}>Crea Corso</button>
          </div>
        </div>
      {/if}

      <div class="courses-grid">
        {#each snap.telemetry.courseProgress as metric (metric.courseId)}
          <div class="bento-card bento-card-interactive course-card">
            <button
              type="button"
              class="course-click-area"
              onclick={() => navStore.openCourseDetail(metric.courseId)}
            >
              <div class="cc-top">
                <span class="course-sigla-lg" style:background={metric.colorHex}>{metric.sigla}</span>
                {#if metric.daysUntilExam !== null}
                  <span class="days-badge">-{metric.daysUntilExam} gg esame</span>
                {/if}
              </div>

              <h3>{metric.name}</h3>
              <p class="cc-meta">{metric.cfu} CFU {metric.professor ? `· ${metric.professor}` : ""}</p>

              <div class="cc-prog">
                <div class="prog-labels">
                  <span>Avanzamento</span>
                  <strong>{metric.completionPct}%</strong>
                </div>
                <div class="mini-track">
                  <div class="mini-fill" style:width="{metric.completionPct}%" style:background={metric.colorHex}></div>
                </div>
              </div>

              <div class="cc-stats">
                <span><GraduationCap size={13} /> {metric.completedTasks}/{metric.totalTasks} task</span>
                <span><HelpCircle size={13} /> {metric.quizzesCompleted}/{metric.quizzesTotal} quiz</span>
              </div>
            </button>
          </div>
        {/each}
      </div>
    </div>
  {/if}
{/if}

<style>
  .courses-wrap,
  .course-detail {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .page-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
    flex-wrap: wrap;
  }

  .page-head h1 {
    margin: 0;
    font-size: 24px;
    font-weight: 800;
  }

  .page-head p {
    margin: 4px 0 0;
    font-size: 13px;
    color: var(--text-secondary);
  }

  .add-course-panel {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .form-2 {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 10px;
  }

  .soft-input {
    padding: 9px 12px;
    border-radius: 12px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface-muted);
    color: var(--text-primary);
    font-size: 13px;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .courses-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 16px;
  }

  .course-card {
    overflow: hidden;
  }

  .course-click-area {
    width: 100%;
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    text-align: left;
  }

  .cc-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .course-sigla-lg {
    padding: 4px 12px;
    border-radius: 999px;
    color: #fff;
    font-size: 12px;
    font-weight: 800;
  }

  .days-badge,
  .countdown-pill {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 4px 10px;
    border-radius: 999px;
    background: var(--accent-soft);
    color: var(--accent);
    font-size: 11.5px;
    font-weight: 700;
  }

  .course-card h3 {
    margin: 0;
    font-size: 16.5px;
    font-weight: 800;
  }

  .cc-meta {
    margin: -6px 0 0;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .cc-prog {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .prog-labels {
    display: flex;
    justify-content: space-between;
    font-size: 12px;
  }

  .mini-track {
    height: 7px;
    border-radius: 999px;
    background: var(--bg-surface-muted);
    overflow: hidden;
  }

  .mini-fill {
    height: 100%;
    border-radius: 999px;
    transition: width 0.25s ease;
  }

  .cc-stats {
    display: flex;
    justify-content: space-between;
    font-size: 12px;
    color: var(--text-secondary);
    padding-top: 6px;
    border-top: 1px solid var(--border-subtle);
  }

  .cc-stats span {
    display: inline-flex;
    align-items: center;
    gap: 5px;
  }

  /* Course Detail */
  .detail-hero {
    padding: 24px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 20px;
    flex-wrap: wrap;
  }

  .dh-badges {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .tag-pill {
    padding: 4px 10px;
    border-radius: 999px;
    background: var(--bg-surface-muted);
    font-size: 12px;
    font-weight: 700;
  }

  .detail-hero h1 {
    margin: 10px 0 4px;
    font-size: 24px;
    font-weight: 800;
  }

  .prof-line {
    margin: 0;
    font-size: 13px;
    color: var(--text-secondary);
  }

  .dh-metrics {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
  }

  .dm-box {
    padding: 12px 16px;
    border-radius: 16px;
    background: var(--bg-surface-muted);
    display: flex;
    flex-direction: column;
    min-width: 110px;
  }

  .dm-box strong {
    font-size: 20px;
    font-weight: 800;
  }

  .dm-box span {
    font-size: 11.5px;
    color: var(--text-secondary);
  }

  .cat-breakdown-card {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .cat-breakdown-card h3 {
    margin: 0;
    font-size: 15px;
  }

  .cat-grid-5 {
    display: grid;
    grid-template-columns: repeat(5, minmax(0, 1fr));
    gap: 12px;
  }

  .cat-col {
    padding: 12px;
    border-radius: 14px;
    background: var(--bg-surface-muted);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .cat-col-top {
    display: flex;
    justify-content: space-between;
    font-size: 12px;
  }

  .cat-col small {
    font-size: 11px;
    color: var(--text-secondary);
  }

  .tab-bar,
  .filter-pills {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .tab-panel {
    padding: 18px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .list-stack {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .quiz-cards-2 {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 10px;
  }

  .row-card {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    border-radius: 14px;
    background: var(--bg-surface-muted);
  }

  .row-card.done {
    opacity: 0.6;
  }

  .chk-box {
    width: 20px;
    height: 20px;
    border-radius: 6px;
    border: 1.5px solid var(--border-strong);
    background: var(--bg-surface);
    display: grid;
    place-items: center;
    flex-shrink: 0;
  }

  .chk-box.checked {
    background: var(--success);
    border-color: var(--success);
    color: #fff;
  }

  .row-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .row-meta {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 11.5px;
    color: var(--text-secondary);
  }

  .cat-badge {
    padding: 2px 7px;
    border-radius: 999px;
    color: #fff;
    font-size: 10px;
    font-weight: 700;
  }

  .score-pill {
    padding: 2px 7px;
    border-radius: 999px;
    background: rgba(16, 185, 129, 0.16);
    color: #059669;
    font-weight: 700;
  }

  .panel-head-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .panel-head-row h3 {
    margin: 0;
    font-size: 15px;
  }

  .inline-form {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }

  .del-icon {
    color: var(--text-muted);
  }

  .del-icon:hover {
    color: var(--danger);
  }

  .empty-soft {
    margin: 0;
    font-size: 13px;
    color: var(--text-secondary);
  }

  .btn-xs {
    padding: 6px 11px;
    font-size: 12px;
  }

  @media (max-width: 960px) {
    .cat-grid-5 {
      grid-template-columns: repeat(2, 1fr);
    }
  }
</style>
