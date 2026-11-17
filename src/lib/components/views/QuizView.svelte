<script lang="ts">
  import { Bot, CheckCircle2, Flame, HelpCircle, Play, RotateCcw } from "lucide-svelte";
  import { navStore } from "../../stores/navigation.svelte";
  import { sessionStore } from "../../stores/session.svelte";
  import { formatItalianDateShort, getTodayIso } from "../../utils/calendar";

  let snap = $derived(sessionStore.activeSnapshot);
  let todayIso = $derived(getTodayIso());

  let statusFilter = $state<"all" | "today" | "todo" | "done" | "errors">("all");
  let courseFilter = $state<string>("all");

  let filteredQuizzes = $derived.by(() => {
    if (!snap) return [];
    return snap.quizzes.filter((q) => {
      if (courseFilter !== "all" && q.courseId !== courseFilter) return false;
      if (statusFilter === "today" && q.date !== todayIso) return false;
      if (statusFilter === "todo" && q.latestAttempt !== null) return false;
      if (statusFilter === "done" && q.latestAttempt === null) return false;
      if (
        statusFilter === "errors" &&
        (!q.latestAttempt || q.latestAttempt.wrongQuestionIds.length === 0)
      ) {
        return false;
      }
      return true;
    });
  });

  let wrongQuestionsCount = $derived.by(() => {
    if (!snap) return 0;
    return snap.quizzes.reduce((acc, q) => acc + (q.latestAttempt?.wrongQuestionIds.length ?? 0), 0);
  });
</script>

{#if snap}
  <div class="quiz-layout bento-enter">
    <div class="quiz-main">
      <header class="quiz-head">
        <div>
          <h1>Quiz Giornalieri</h1>
          <p>Un quiz mirato per ogni materia e argomento del piano di studio</p>
        </div>

        <div class="filter-pills">
          <button type="button" class="pill-filter" class:active={statusFilter === "all"} onclick={() => (statusFilter = "all")}>
            Tutti ({snap.quizzes.length})
          </button>
          <button type="button" class="pill-filter" class:active={statusFilter === "today"} onclick={() => (statusFilter = "today")}>
            Di Oggi
          </button>
          <button type="button" class="pill-filter" class:active={statusFilter === "todo"} onclick={() => (statusFilter = "todo")}>
            Da svolgere
          </button>
          <button type="button" class="pill-filter" class:active={statusFilter === "done"} onclick={() => (statusFilter = "done")}>
            Completati
          </button>
          <button type="button" class="pill-filter" class:active={statusFilter === "errors"} onclick={() => (statusFilter = "errors")}>
            Con errori ({wrongQuestionsCount})
          </button>
        </div>
      </header>

      <div class="course-chips">
        <button type="button" class="pill-filter" class:active={courseFilter === "all"} onclick={() => (courseFilter = "all")}>
          Tutte le materie
        </button>
        {#each snap.courses as c (c.id)}
          <button
            type="button"
            class="pill-filter"
            class:active={courseFilter === c.id}
            onclick={() => (courseFilter = c.id)}
          >
            <span class="dot" style:background={c.colorHex}></span>
            {c.sigla}
          </button>
        {/each}
      </div>

      {#if filteredQuizzes.length === 0}
        <div class="bento-card empty-box">
          <HelpCircle size={28} />
          <h3>Nessun quiz trovato con questi filtri</h3>
          <p>I quiz vengono generati automaticamente nei file v3 del Builder per ogni materia del giorno, oppure puoi chiederne uno all'AI.</p>
          <button type="button" class="pill-btn pill-btn-primary" onclick={() => navStore.navigate("ai")}>
            <Bot size={15} />
            Chiedi all'AI di creare un quiz
          </button>
        </div>
      {:else}
        <div class="quiz-grid">
          {#each filteredQuizzes as quiz (quiz.id)}
            {@const course = snap.courses.find((c) => c.id === quiz.courseId)}
            {@const att = quiz.latestAttempt}
            <div class="bento-card bento-card-interactive q-card">
              <div class="qc-banner" style:--c-hex={course?.colorHex ?? "#7C5CFC"}>
                <div class="qc-top-row">
                  <span class="course-pill" style:background={course?.colorHex ?? "#7C5CFC"}>
                    {course?.sigla ?? "CORSO"}
                  </span>
                  <span class="date-tag">{formatItalianDateShort(quiz.date)}</span>
                </div>
                <div class="qc-icon-orb">
                  <HelpCircle size={24} />
                </div>
              </div>

              <div class="qc-body">
                <h3>{quiz.title}</h3>
                <p>{quiz.topicSummary || "Verifica sugli argomenti della giornata"}</p>

                <div class="qc-meta">
                  <span>{quiz.questions.length} domande</span>
                  {#if att}
                    <span class="score-ok">
                      <CheckCircle2 size={13} />
                      {att.scorePct}% ({att.correctCount}/{att.totalCount})
                    </span>
                  {:else}
                    <span class="todo-badge">Nuovo</span>
                  {/if}
                </div>

                <button
                  type="button"
                  class="pill-btn pill-btn-primary full-btn"
                  onclick={() => (navStore.activeQuizId = quiz.id)}
                >
                  {#if att}
                    <RotateCcw size={14} /> Ripeti Quiz
                  {:else}
                    <Play size={14} /> Inizia Quiz
                  {/if}
                </button>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Colonna destra: Statistiche Quiz per Materia (stile video) -->
    <aside class="bento-card quiz-side">
      <div class="side-head">
        <h3>Rendimento Quiz</h3>
        <span class="avg-big">
          {snap.telemetry.quizOverallAvgScore !== null ? `${snap.telemetry.quizOverallAvgScore}%` : "—"}
        </span>
      </div>
      <p class="side-sub">
        {snap.telemetry.quizzesCompleted} svolti su {snap.telemetry.quizzesTotal} disponibili
      </p>

      <div class="course-quiz-stats">
        {#each snap.telemetry.courseProgress as m (m.courseId)}
          <div class="cqs-row">
            <span class="cqs-dot" style:background={m.colorHex}></span>
            <div class="cqs-info">
              <strong>{m.sigla}</strong>
              <span>{m.quizzesCompleted}/{m.quizzesTotal} quiz</span>
            </div>
            <strong class="cqs-score">
              {m.quizAverageScore !== null ? `${m.quizAverageScore}%` : "—"}
            </strong>
          </div>
        {/each}
      </div>

      <button type="button" class="pill-btn pill-btn-soft full-btn" onclick={() => navStore.navigate("ai")}>
        <Flame size={15} />
        Genera Quiz extra con AI
      </button>
    </aside>
  </div>
{/if}

<style>
  .quiz-layout {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 310px;
    gap: 18px;
    align-items: start;
  }

  .quiz-main {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .quiz-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
    flex-wrap: wrap;
  }

  .quiz-head h1 {
    margin: 0;
    font-size: 24px;
    font-weight: 800;
  }

  .quiz-head p {
    margin: 4px 0 0;
    font-size: 13px;
    color: var(--text-secondary);
  }

  .filter-pills,
  .course-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .quiz-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 16px;
  }

  .q-card {
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .qc-banner {
    padding: 16px;
    min-height: 112px;
    background: linear-gradient(135deg, color-mix(in srgb, var(--c-hex) 18%, var(--bg-surface-muted)), var(--bg-surface-muted));
    display: flex;
    flex-direction: column;
    justify-content: space-between;
  }

  .qc-top-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .course-pill {
    padding: 3px 10px;
    border-radius: 999px;
    color: #fff;
    font-size: 11px;
    font-weight: 800;
  }

  .date-tag {
    padding: 3px 8px;
    border-radius: 999px;
    background: var(--bg-surface);
    font-size: 11px;
    font-weight: 700;
    color: var(--text-secondary);
  }

  .qc-icon-orb {
    align-self: center;
    width: 44px;
    height: 44px;
    border-radius: 50%;
    background: var(--bg-surface);
    display: grid;
    place-items: center;
    color: var(--accent);
    box-shadow: var(--shadow-card);
  }

  .qc-body {
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    flex: 1;
  }

  .qc-body h3 {
    margin: 0;
    font-size: 15px;
    font-weight: 800;
  }

  .qc-body p {
    margin: 0;
    font-size: 12.5px;
    color: var(--text-secondary);
    flex: 1;
  }

  .qc-meta {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .score-ok {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--success);
    font-weight: 700;
  }

  .todo-badge {
    padding: 2px 8px;
    border-radius: 999px;
    background: var(--accent-soft);
    color: var(--accent);
    font-size: 11px;
    font-weight: 700;
  }

  .full-btn {
    width: 100%;
    justify-content: center;
    margin-top: 4px;
  }

  .quiz-side {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .side-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
  }

  .side-head h3 {
    margin: 0;
    font-size: 15.5px;
  }

  .avg-big {
    font-size: 26px;
    font-weight: 800;
    color: var(--accent);
  }

  .side-sub {
    margin: -6px 0 4px;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .course-quiz-stats {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .cqs-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 9px 11px;
    border-radius: 12px;
    background: var(--bg-surface-muted);
  }

  .cqs-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
  }

  .cqs-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    font-size: 12.5px;
  }

  .cqs-info span {
    font-size: 11px;
    color: var(--text-secondary);
  }

  .cqs-score {
    font-size: 13px;
  }

  .empty-box {
    padding: 40px 24px;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 10px;
    color: var(--text-secondary);
  }

  .empty-box h3 {
    margin: 0;
    color: var(--text-primary);
  }

  @media (max-width: 1100px) {
    .quiz-layout {
      grid-template-columns: 1fr;
    }
  }
</style>
