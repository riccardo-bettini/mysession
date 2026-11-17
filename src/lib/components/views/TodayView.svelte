<script lang="ts">
  import {
    BookOpen,
    Check,
    ChevronLeft,
    ChevronRight,
    Clock,
    FileText,
    HelpCircle,
    Plus,
    Sparkles,
    Trash2
  } from "lucide-svelte";
  import { navStore } from "../../stores/navigation.svelte";
  import { sessionStore } from "../../stores/session.svelte";
  import { themeStore } from "../../stores/theme.svelte";
  import type { TaskCategory } from "../../types/domain";
  import { addDaysIso, formatItalianDateLong, getTodayIso } from "../../utils/calendar";
  import { getCategoryColor, getCategoryLabel, TASK_CATEGORIES } from "../../utils/categories";

  let snap = $derived(sessionStore.activeSnapshot);
  let todayIso = $derived(getTodayIso());
  let currentDate = $derived(navStore.todayTargetDate || todayIso);

  let categoryFilter = $state<"all" | TaskCategory>("all");
  let showAddForm = $state(false);
  let newTitle = $state("");
  let newCourseId = $state("");
  let newCategory = $state<TaskCategory>("teoria");
  let newChapter = $state("");
  let newTopic = $state("");
  let newPages = $state("");
  let newHours = $state(1.5);

  let opPlan = $derived(snap?.operationalPlans.find((p) => p.date === currentDate) ?? null);
  let dayTasks = $derived(snap?.tasks.filter((t) => t.date === currentDate) ?? []);
  let filteredTasks = $derived(
    categoryFilter === "all" ? dayTasks : dayTasks.filter((t) => t.category === categoryFilter)
  );
  let dayQuizzes = $derived(snap?.quizzes.filter((q) => q.date === currentDate) ?? []);

  let dayProgress = $derived.by(() => {
    if (dayTasks.length === 0) return { done: 0, total: 0, pct: 0, hours: 0 };
    const done = dayTasks.filter((t) => t.done).length;
    const hours = dayTasks.reduce((a, t) => a + t.plannedHours, 0);
    return {
      done,
      total: dayTasks.length,
      pct: Math.round((done / dayTasks.length) * 100),
      hours: Math.round(hours * 10) / 10
    };
  });

  function setDate(iso: string) {
    navStore.todayTargetDate = iso;
  }

  async function addStructuredTask() {
    if (!snap || !newTitle.trim()) return;
    await sessionStore.upsertTask({
      sessionId: snap.session.id,
      courseId: newCourseId || snap.courses[0]?.id || null,
      date: currentDate,
      category: newCategory,
      title: newTitle.trim(),
      chapter: newChapter.trim(),
      topic: newTopic.trim(),
      pages: newPages.trim(),
      plannedHours: Number(newHours) || 1.0,
      priority: 2
    });
    newTitle = "";
    newChapter = "";
    newTopic = "";
    newPages = "";
    showAddForm = false;
    navStore.showToast("Task aggiunta alla giornata", "success");
  }
</script>

{#if snap}
  <div class="today-wrap bento-enter">
    <!-- Header giornata con switcher Ieri / Oggi / Domani -->
    <header class="bento-card today-header">
      <div class="th-left">
        <div class="date-nav">
          <button type="button" class="icon-pill" onclick={() => setDate(addDaysIso(currentDate, -1))} aria-label="Giorno precedente">
            <ChevronLeft size={16} />
          </button>
          <button type="button" class="pill-btn pill-btn-soft btn-xs" class:active-today={currentDate === todayIso} onclick={() => setDate(todayIso)}>
            Oggi
          </button>
          <button type="button" class="icon-pill" onclick={() => setDate(addDaysIso(currentDate, 1))} aria-label="Giorno successivo">
            <ChevronRight size={16} />
          </button>
          <input
            type="date"
            class="date-picker-mini"
            value={currentDate}
            onchange={(e) => setDate((e.currentTarget as HTMLInputElement).value)}
          />
        </div>
        <h1>{formatItalianDateLong(currentDate)}</h1>
      </div>

      <div class="th-right">
        <div class="progress-stat">
          <strong>{dayProgress.done}/{dayProgress.total} task</strong>
          <span>{dayProgress.hours}h pianificate · {dayProgress.pct}%</span>
        </div>
        <div class="ring-bar">
          <div class="ring-fill" style:width="{dayProgress.pct}%"></div>
        </div>
      </div>
    </header>

    <div class="today-grid">
      <!-- Colonna principale: Piano Operativo + To-Do Strutturata -->
      <div class="today-main">
        <!-- 1. Piano Operativo -->
        <section class="bento-card op-card">
          <div class="sec-head">
            <div class="sec-title">
              <Sparkles size={17} class="accent-ic" />
              <h2>Piano Operativo</h2>
            </div>
            {#if opPlan}
              <span class="pill-tag-soft">{opPlan.blocks.length} blocchi sequenziali</span>
            {/if}
          </div>

          {#if opPlan}
            <div class="op-goal-box">
              <strong>Obiettivo:</strong> {opPlan.dailyGoal}
              {#if opPlan.strategyNote}
                <p class="op-strategy">{opPlan.strategyNote}</p>
              {/if}
            </div>

            <div class="blocks-timeline">
              {#each opPlan.blocks as blk (blk.id)}
                {@const course = snap.courses.find((c) => c.id === blk.courseId)}
                <div class="op-block" class:done={blk.done}>
                  <button
                    type="button"
                    class="blk-check"
                    class:checked={blk.done}
                    onclick={() => sessionStore.toggleOperationalBlock(blk.id, !blk.done)}
                    aria-label="Segna blocco completato"
                  >
                    {#if blk.done}<Check size={13} />{:else}{blk.blockOrder}{/if}
                  </button>
                  <div class="blk-body">
                    <div class="blk-top">
                      {#if course}
                        <span class="course-chip" style:background={course.colorHex}>{course.sigla}</span>
                      {/if}
                      <span class="cat-chip" style:background={getCategoryColor(blk.category, themeStore.settings)}>
                        {getCategoryLabel(blk.category)}
                      </span>
                      <span class="dur-chip"><Clock size={12} /> {blk.durationHours}h</span>
                    </div>
                    <strong>{blk.title}</strong>
                    <div class="meta-chips">
                      {#if blk.chapter}<span><BookOpen size={11} /> {blk.chapter}</span>{/if}
                      {#if blk.topic}<span>Argomento: {blk.topic}</span>{/if}
                      {#if blk.pages}<span><FileText size={11} /> {blk.pages}</span>{/if}
                    </div>
                    {#if blk.instructions}
                      <p class="blk-instr">{blk.instructions}</p>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          {:else}
            <p class="empty-soft">Nessun piano operativo generato per questa data. Puoi seguire la To-Do List qui sotto.</p>
          {/if}
        </section>

        <!-- 2. To-Do List Strutturata per Categorie -->
        <section class="bento-card todo-card">
          <div class="sec-head">
            <div class="sec-title">
              <h2>To-Do List Strutturata</h2>
            </div>
            <button type="button" class="pill-btn pill-btn-primary btn-xs" onclick={() => (showAddForm = !showAddForm)}>
              <Plus size={14} />
              Nuova Task
            </button>
          </div>

          <!-- Filtri 5 Categorie -->
          <div class="filter-row">
            <button type="button" class="pill-filter" class:active={categoryFilter === "all"} onclick={() => (categoryFilter = "all")}>
              Tutte ({dayTasks.length})
            </button>
            {#each TASK_CATEGORIES as cat (cat.id)}
              {@const cnt = dayTasks.filter((t) => t.category === cat.id).length}
              {#if cnt > 0 || cat.id !== "ripasso"}
                <button
                  type="button"
                  class="pill-filter"
                  class:active={categoryFilter === cat.id}
                  onclick={() => (categoryFilter = cat.id)}
                >
                  <span class="f-dot" style:background={getCategoryColor(cat.id, themeStore.settings)}></span>
                  {cat.label} ({cnt})
                </button>
              {/if}
            {/each}
          </div>

          {#if showAddForm}
            <div class="add-task-box">
              <div class="form-grid">
                <input type="text" class="soft-input span-2" placeholder="Titolo attività *" bind:value={newTitle} />
                <select class="soft-input" bind:value={newCourseId}>
                  <option value="">Nessun corso</option>
                  {#each snap.courses as c (c.id)}
                    <option value={c.id}>{c.sigla} — {c.name}</option>
                  {/each}
                </select>
                <select class="soft-input" bind:value={newCategory}>
                  {#each TASK_CATEGORIES as cat (cat.id)}
                    <option value={cat.id}>{cat.label}</option>
                  {/each}
                </select>
                <input type="text" class="soft-input" placeholder="Capitolo (es. Cap. 3)" bind:value={newChapter} />
                <input type="text" class="soft-input" placeholder="Argomento specifico" bind:value={newTopic} />
                <input type="text" class="soft-input" placeholder="Pagine / Esercizi (es. pp. 40-58)" bind:value={newPages} />
                <input type="number" class="soft-input" min="0.5" max="12" step="0.5" bind:value={newHours} />
              </div>
              <div class="form-actions">
                <button type="button" class="pill-btn pill-btn-soft btn-xs" onclick={() => (showAddForm = false)}>Annulla</button>
                <button type="button" class="pill-btn pill-btn-primary btn-xs" onclick={addStructuredTask}>Salva Task</button>
              </div>
            </div>
          {/if}

          {#if filteredTasks.length === 0}
            <p class="empty-soft">Nessuna task in questa categoria per {currentDate}.</p>
          {:else}
            <div class="tasks-list">
              {#each filteredTasks as t (t.id)}
                {@const course = snap.courses.find((c) => c.id === t.courseId)}
                <div class="task-item-card" class:done={t.done}>
                  <div class="task-main-row">
                    <button
                      type="button"
                      class="chk-box"
                      class:checked={t.done}
                      onclick={() => sessionStore.toggleTask(t.id, !t.done)}
                      aria-label="Completa task"
                    >
                      {#if t.done}<Check size={14} />{/if}
                    </button>

                    <div class="task-content">
                      <div class="task-badges">
                        {#if course}
                          <span class="course-chip" style:background={course.colorHex}>{course.sigla}</span>
                        {/if}
                        <span class="cat-chip" style:background={getCategoryColor(t.category, themeStore.settings)}>
                          {getCategoryLabel(t.category)}
                        </span>
                        <span class="dur-chip">{t.plannedHours}h</span>
                      </div>

                      <strong class="task-title">{t.title}</strong>

                      <div class="structured-fields">
                        {#if t.chapter}
                          <div class="sf-pill"><span>Capitolo</span><b>{t.chapter}</b></div>
                        {/if}
                        {#if t.topic}
                          <div class="sf-pill"><span>Argomento</span><b>{t.topic}</b></div>
                        {/if}
                        {#if t.pages}
                          <div class="sf-pill"><span>Pagine / Es.</span><b>{t.pages}</b></div>
                        {/if}
                        {#if t.source}
                          <div class="sf-pill"><span>Fonte</span><b>{t.source}</b></div>
                        {/if}
                      </div>

                      {#if t.subtasks && t.subtasks.length > 0}
                        <div class="subtasks-box">
                          {#each t.subtasks as sub (sub.id)}
                            <button
                              type="button"
                              class="subtask-row"
                              class:done={sub.done}
                              onclick={() => sessionStore.toggleTaskSubitem(sub.id, !sub.done)}
                            >
                              <span class="sub-chk" class:checked={sub.done}>
                                {#if sub.done}<Check size={11} />{/if}
                              </span>
                              <span class="sub-txt">{sub.text}</span>
                              {#if sub.pages}
                                <span class="sub-pages">{sub.pages}</span>
                              {/if}
                            </button>
                          {/each}
                        </div>
                      {/if}
                    </div>

                    <button
                      type="button"
                      class="del-icon"
                      title="Elimina task"
                      onclick={() => sessionStore.deleteTask(t.id)}
                    >
                      <Trash2 size={14} />
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </section>
      </div>

      <!-- Colonna Destra: Quiz di Oggi -->
      <aside class="today-side">
        <div class="bento-card quiz-day-card">
          <div class="sec-head">
            <div class="sec-title">
              <HelpCircle size={17} class="accent-ic" />
              <h2>Quiz del Giorno</h2>
            </div>
          </div>

          {#if dayQuizzes.length === 0}
            <p class="empty-soft">Nessun quiz giornaliero collegato a questa data.</p>
          {:else}
            <div class="day-quiz-list">
              {#each dayQuizzes as quiz (quiz.id)}
                {@const course = snap.courses.find((c) => c.id === quiz.courseId)}
                <div class="dq-card">
                  <div class="dq-top">
                    {#if course}
                      <span class="course-chip" style:background={course.colorHex}>{course.sigla}</span>
                    {/if}
                    {#if quiz.latestAttempt}
                      <span class="score-pill">{quiz.latestAttempt.scorePct}%</span>
                    {:else}
                      <span class="todo-pill">Da svolgere</span>
                    {/if}
                  </div>
                  <strong>{quiz.title}</strong>
                  {#if quiz.topicSummary}
                    <p>{quiz.topicSummary}</p>
                  {/if}
                  <div class="dq-foot">
                    <span>{quiz.questions.length} domande</span>
                    <button
                      type="button"
                      class="pill-btn pill-btn-primary btn-xs"
                      onclick={() => (navStore.activeQuizId = quiz.id)}
                    >
                      {quiz.latestAttempt ? "Ripeti Quiz" : "Avvia Quiz"}
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </aside>
    </div>
  </div>
{/if}

<style>
  .today-wrap {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .today-header {
    padding: 18px 22px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
  }

  .th-left {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .date-nav {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .icon-pill {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: var(--bg-surface-muted);
    display: grid;
    place-items: center;
  }

  .active-today {
    background: var(--accent-soft);
    color: var(--accent);
  }

  .date-picker-mini {
    border: 1px solid var(--border-subtle);
    border-radius: 999px;
    padding: 4px 10px;
    background: var(--bg-surface-muted);
    color: var(--text-primary);
    font-size: 12px;
  }

  .today-header h1 {
    margin: 0;
    font-size: 22px;
    font-weight: 800;
  }

  .th-right {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 6px;
    min-width: 190px;
  }

  .progress-stat {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .progress-stat strong {
    font-size: 16px;
    color: var(--text-primary);
  }

  .ring-bar {
    width: 100%;
    height: 8px;
    border-radius: 999px;
    background: var(--bg-surface-muted);
    overflow: hidden;
  }

  .ring-fill {
    height: 100%;
    border-radius: 999px;
    background: var(--accent);
    transition: width 0.25s ease;
  }

  .today-grid {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 330px;
    gap: 16px;
    align-items: start;
  }

  .today-main {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .op-card,
  .todo-card,
  .quiz-day-card {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .sec-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }

  .sec-title {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .sec-title h2 {
    margin: 0;
    font-size: 16.5px;
    font-weight: 800;
  }

  :global(.accent-ic) {
    color: var(--accent);
  }

  .pill-tag-soft {
    padding: 4px 10px;
    border-radius: 999px;
    background: var(--bg-surface-muted);
    font-size: 11.5px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .op-goal-box {
    padding: 12px 14px;
    border-radius: 16px;
    background: var(--accent-soft);
    font-size: 13.5px;
  }

  .op-strategy {
    margin: 6px 0 0;
    font-size: 12.5px;
    color: var(--text-secondary);
  }

  .blocks-timeline {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .op-block {
    display: flex;
    gap: 12px;
    padding: 12px 14px;
    border-radius: 16px;
    background: var(--bg-surface-muted);
  }

  .op-block.done {
    opacity: 0.6;
  }

  .blk-check {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    background: var(--bg-surface);
    border: 1.5px solid var(--border-strong);
    display: grid;
    place-items: center;
    font-size: 12px;
    font-weight: 700;
    flex-shrink: 0;
  }

  .blk-check.checked {
    background: var(--success);
    border-color: var(--success);
    color: #fff;
  }

  .blk-body {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .blk-top,
  .task-badges {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }

  .course-chip,
  .cat-chip {
    padding: 2px 8px;
    border-radius: 999px;
    color: #fff;
    font-size: 10.5px;
    font-weight: 700;
  }

  .dur-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    border-radius: 999px;
    background: var(--bg-surface);
    font-size: 11px;
    font-weight: 700;
    color: var(--text-secondary);
  }

  .meta-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .meta-chips span {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .blk-instr {
    margin: 2px 0 0;
    font-size: 12.5px;
    color: var(--text-secondary);
  }

  .filter-row {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .f-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .add-task-box {
    padding: 14px;
    border-radius: 16px;
    background: var(--bg-surface-muted);
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .form-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
  }

  .span-2 {
    grid-column: span 2;
  }

  .soft-input {
    padding: 8px 11px;
    border-radius: 12px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface);
    color: var(--text-primary);
    font-size: 12.5px;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .tasks-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .task-item-card {
    padding: 14px;
    border-radius: 18px;
    background: var(--bg-surface-muted);
    border: 1px solid var(--border-subtle);
  }

  .task-item-card.done {
    opacity: 0.6;
  }

  .task-item-card.done .task-title {
    text-decoration: line-through;
  }

  .task-main-row {
    display: flex;
    align-items: flex-start;
    gap: 12px;
  }

  .chk-box {
    width: 22px;
    height: 22px;
    margin-top: 2px;
    border-radius: 7px;
    background: var(--bg-surface);
    border: 1.5px solid var(--border-strong);
    display: grid;
    place-items: center;
    flex-shrink: 0;
  }

  .chk-box.checked {
    background: var(--success);
    border-color: var(--success);
    color: #fff;
  }

  .task-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }

  .task-title {
    font-size: 14.5px;
    font-weight: 700;
  }

  .structured-fields {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .sf-pill {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 3px 9px;
    border-radius: 10px;
    background: var(--bg-surface);
    font-size: 11.5px;
  }

  .sf-pill span {
    color: var(--text-muted);
    font-weight: 600;
  }

  .sf-pill b {
    color: var(--text-primary);
    font-weight: 600;
  }

  .subtasks-box {
    margin-top: 4px;
    padding-top: 8px;
    border-top: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .subtask-row {
    display: flex;
    align-items: center;
    gap: 8px;
    text-align: left;
    font-size: 12.5px;
    padding: 3px 0;
  }

  .subtask-row.done .sub-txt {
    text-decoration: line-through;
    opacity: 0.6;
  }

  .sub-chk {
    width: 16px;
    height: 16px;
    border-radius: 5px;
    border: 1.5px solid var(--border-strong);
    background: var(--bg-surface);
    display: grid;
    place-items: center;
    flex-shrink: 0;
  }

  .sub-chk.checked {
    background: var(--accent);
    border-color: var(--accent);
    color: #fff;
  }

  .sub-txt {
    flex: 1;
  }

  .sub-pages {
    font-size: 11px;
    color: var(--text-secondary);
    background: var(--bg-surface);
    padding: 1px 6px;
    border-radius: 6px;
  }

  .del-icon {
    color: var(--text-muted);
    padding: 4px;
  }

  .del-icon:hover {
    color: var(--danger);
  }

  .day-quiz-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .dq-card {
    padding: 14px;
    border-radius: 16px;
    background: var(--bg-surface-muted);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .dq-top,
  .dq-foot {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .dq-card p {
    margin: 0;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .dq-foot span {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .score-pill {
    padding: 2px 8px;
    border-radius: 999px;
    background: rgba(16, 185, 129, 0.16);
    color: #059669;
    font-size: 11px;
    font-weight: 700;
  }

  .todo-pill {
    padding: 2px 8px;
    border-radius: 999px;
    background: var(--accent-soft);
    color: var(--accent);
    font-size: 11px;
    font-weight: 700;
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

  @media (max-width: 1140px) {
    .today-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
