<script lang="ts">
  import { ArrowRight, Bot, Calendar, CheckCircle2, Flame, HelpCircle, Sparkles } from "lucide-svelte";
  import { navStore } from "../../stores/navigation.svelte";
  import { sessionStore } from "../../stores/session.svelte";
  import { themeStore } from "../../stores/theme.svelte";
  import {
    buildWeekGrid,
    formatItalianDateShort,
    getTodayIso,
    MESI_ITALIANI,
    parseIsoDate
  } from "../../utils/calendar";
  import { CORE_FIVE_CATEGORIES, getCategoryColor, getCategoryLabel } from "../../utils/categories";

  let snap = $derived(sessionStore.activeSnapshot);
  let todayIso = $derived(getTodayIso());
  let statsScope = $state<"week" | "session">("session");
  let rightTab = $state<"exams" | "plans">("exams");

  let weekCells = $derived(buildWeekGrid(todayIso, themeStore.settings.startDayOfWeek, todayIso));
  let monthHeaderLabel = $derived.by(() => {
    const { year, month } = parseIsoDate(todayIso);
    return `${MESI_ITALIANI[month - 1]} ${year}`;
  });

  let todayOpPlan = $derived(snap?.operationalPlans.find((p) => p.date === todayIso) ?? null);
  let todayTasks = $derived(snap?.tasks.filter((t) => t.date === todayIso) ?? []);
  let todayQuizzes = $derived(snap?.quizzes.filter((q) => q.date === todayIso) ?? []);
  let upcomingExams = $derived.by(() => {
    if (!snap) return [];
    return snap.exams
      .filter((e) => e.status === "planned" && e.date >= todayIso)
      .slice(0, 4);
  });

  let weekTasksStats = $derived.by(() => {
    if (!snap) return { done: 0, total: 0, hours: 0 };
    const weekSet = new Set(weekCells.map((c) => c.isoDate));
    const list = snap.tasks.filter((t) => weekSet.has(t.date));
    const done = list.filter((t) => t.done).length;
    const hours = list.filter((t) => t.done).reduce((acc, t) => acc + t.plannedHours, 0);
    return { done, total: list.length, hours: Math.round(hours * 10) / 10 };
  });

  let max7DayHours = $derived.by(() => {
    const vals = snap?.telemetry.hoursLast7Days.map((x) => x[1]) ?? [1];
    return Math.max(1, ...vals);
  });

  const WEEKDAY_MINI = ["D", "L", "M", "M", "G", "V", "S"];
</script>

{#if snap}
  <div class="home-layout bento-enter">
    <!-- Colonna principale sinistra/centrale -->
    <div class="main-col">
      <!-- Hero Banner (stile video) -->
      <section class="hero-banner">
        <div class="hero-copy">
          <div class="hero-eyebrow">
            <span>{snap.session.academicYear || "Sessione attiva"}</span>
            {#if snap.telemetry.daysToNextTarget !== null}
              <span class="hero-chip">
                Prossimo esame tra {snap.telemetry.daysToNextTarget} gg
                {#if snap.telemetry.nextTargetCourse}
                  ({snap.telemetry.nextTargetCourse.sigla})
                {/if}
              </span>
            {/if}
          </div>
          <h1>{snap.session.name}</h1>
          <p>
            {#if snap.telemetry.overallProgressPct > 0}
              Avanzamento complessivo al <strong>{snap.telemetry.overallProgressPct}%</strong> ·
              {snap.telemetry.tasksCompleted}/{snap.telemetry.tasksTotal} task completate.
            {:else}
              Pianifica le tue giornate con il piano operativo, le categorie e i quiz giornalieri.
            {/if}
          </p>
        </div>
        <div class="hero-actions">
          <button type="button" class="pill-btn pill-btn-primary" onclick={() => navStore.openDayInTodayView(todayIso)}>
            Vai a Oggi
            <ArrowRight size={15} />
          </button>
          <button type="button" class="pill-btn pill-btn-soft" onclick={() => navStore.navigate("quiz")}>
            <HelpCircle size={15} />
            Quiz ({todayQuizzes.length} oggi)
          </button>
        </div>
      </section>

      <!-- Riga centrale a 3 card (Corsi, Ore, Categorie) -->
      <div class="mid-grid">
        <!-- Card Corsi Attivi -->
        <div class="bento-card mid-card">
          <div class="card-top">
            <span class="card-title">Corsi</span>
            <button type="button" class="link-mini" onclick={() => navStore.navigate("courses")}>Tutti</button>
          </div>
          <div class="courses-bubbles">
            {#each snap.courses.filter((c) => c.isActive).slice(0, 4) as course (course.id)}
              <button
                type="button"
                class="course-bubble"
                onclick={() => navStore.openCourseDetail(course.id)}
                title={course.name}
              >
                <span class="bubble-avatar" style:background={course.colorHex}>{course.sigla}</span>
                <span class="bubble-sub">{course.cfu} CFU</span>
              </button>
            {/each}
          </div>
        </div>

        <!-- Card Ore -->
        <div class="bento-card mid-card">
          <div class="card-top">
            <span class="card-title">Ore ultimi 7 gg</span>
            <span class="tag-pill">Settimana</span>
          </div>
          <div class="big-metric">
            {snap.telemetry.hoursLast7Days.reduce((a, [, h]) => a + h, 0).toFixed(1)}h
          </div>
          <div class="bars-7">
            {#each snap.telemetry.hoursLast7Days as [iso, hrs] (iso)}
              {@const pct = Math.max(12, Math.round((hrs / max7DayHours) * 100))}
              <div class="bar-col" title="{iso}: {hrs}h">
                <div class="bar-track">
                  <div class="bar-fill" style:height="{pct}%"></div>
                </div>
              </div>
            {/each}
          </div>
        </div>

        <!-- Card Categorie (stile Impact del video) -->
        <button type="button" class="bento-card bento-card-interactive mid-card text-left" onclick={() => navStore.navigate("progress")}>
          <div class="card-top">
            <span class="card-title">Categorie</span>
            <span class="tag-pill">5 aree</span>
          </div>
          <div class="cat-pills-mini">
            {#each CORE_FIVE_CATEGORIES as cat (cat.id)}
              {@const stat = snap.telemetry.categoryTotals.find((x) => x.category === cat.id)}
              <div class="cat-mini-row">
                <span class="cat-dot" style:background={getCategoryColor(cat.id, themeStore.settings)}></span>
                <span class="cat-name">{cat.shortLabel}</span>
                <span class="cat-val">{stat?.completionPct ?? 0}%</span>
              </div>
            {/each}
          </div>
        </button>
      </div>

      <!-- Riga inferiore a 2 card (Statistiche + Card Scura AI/Piano) -->
      <div class="bot-grid">
        <div class="bento-card stats-card">
          <div class="card-top">
            <span class="card-title">Statistiche</span>
            <div class="seg-switch">
              <button type="button" class:active={statsScope === "week"} onclick={() => (statsScope = "week")}>
                Settimana
              </button>
              <button type="button" class:active={statsScope === "session"} onclick={() => (statsScope = "session")}>
                Sessione
              </button>
            </div>
          </div>
          <div class="stats-nums">
            <div class="stat-item">
              <span class="stat-big">
                {statsScope === "week" ? weekTasksStats.done : snap.telemetry.tasksCompleted}
              </span>
              <span class="stat-label">
                su {statsScope === "week" ? weekTasksStats.total : snap.telemetry.tasksTotal} task completate
              </span>
            </div>
            <div class="stat-item">
              <span class="stat-big">
                {snap.telemetry.quizzesCompleted}
              </span>
              <span class="stat-label">
                su {snap.telemetry.quizzesTotal} quiz svolti
                {#if snap.telemetry.quizOverallAvgScore !== null}
                  · media {snap.telemetry.quizOverallAvgScore}%
                {/if}
              </span>
            </div>
          </div>
        </div>

        <div class="bento-card-dark ai-promo-card">
          <div class="promo-badge">
            <Sparkles size={14} />
            <span>Piano Operativo & AI</span>
          </div>
          <h3>{todayOpPlan ? todayOpPlan.dailyGoal : "Assistente & Agenti CLI integrati"}</h3>
          <p>
            {todayOpPlan?.strategyNote ||
              "Chiedi modifiche al piano, genera quiz mirati o scansiona foto dell'orario lezioni."}
          </p>
          <div class="promo-actions">
            <button type="button" class="promo-btn" onclick={() => navStore.navigate("ai")}>
              <Bot size={15} />
              Apri Agente AI
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Colonna Destra (Calendario Strip + Oggi + Appelli/Notifiche) -->
    <aside class="right-col">
      <!-- Mini Strip Settimana stile video -->
      <div class="bento-card strip-card">
        <div class="card-top">
          <span class="card-title">{monthHeaderLabel}</span>
          <button type="button" class="link-mini" onclick={() => navStore.navigate("calendar")}>
            <Calendar size={14} />
          </button>
        </div>
        <div class="week-strip">
          {#each weekCells as cell (cell.isoDate)}
            <button
              type="button"
              class="strip-day"
              class:today={cell.isToday}
              onclick={() => navStore.openDayInTodayView(cell.isoDate)}
            >
              <span class="wd">{WEEKDAY_MINI[cell.jsWeekday]}</span>
              <span class="dn">{cell.day}</span>
            </button>
          {/each}
        </div>
      </div>

      <!-- Focus Oggi -->
      <div class="bento-card focus-card">
        <div class="card-top">
          <span class="card-title">Focus di Oggi</span>
          <button type="button" class="link-mini" onclick={() => navStore.openDayInTodayView(todayIso)}>Apri</button>
        </div>
        {#if todayTasks.length === 0 && todayQuizzes.length === 0}
          <p class="empty-soft">Nessuna attività pianificata per oggi.</p>
        {:else}
          <div class="focus-list">
            {#each todayTasks.slice(0, 4) as t (t.id)}
              {@const course = snap.courses.find((c) => c.id === t.courseId)}
              <button
                type="button"
                class="focus-row"
                class:done={t.done}
                onclick={() => sessionStore.toggleTask(t.id, !t.done)}
              >
                <span class="cat-stripe" style:background={getCategoryColor(t.category, themeStore.settings)}></span>
                <div class="focus-text">
                  <strong>{t.title}</strong>
                  <span>
                    {course ? `${course.sigla} · ` : ""}{getCategoryLabel(t.category)}
                    {t.chapter ? ` · ${t.chapter}` : ""}
                  </span>
                </div>
                <CheckCircle2 size={16} class={t.done ? "chk-done" : "chk-todo"} />
              </button>
            {/each}
          </div>
          {#if todayQuizzes.length > 0}
            <button type="button" class="quiz-banner-mini" onclick={() => navStore.navigate("quiz")}>
              <Flame size={15} />
              <span>{todayQuizzes.length} quiz giornalieri disponibili</span>
            </button>
          {/if}
        {/if}
      </div>

      <!-- Notifiche / Prossimi Appelli -->
      <div class="bento-card notif-card">
        <div class="card-top">
          <span class="card-title">Appelli & Piano</span>
          <div class="seg-switch">
            <button type="button" class:active={rightTab === "exams"} onclick={() => (rightTab = "exams")}>
              Esami
            </button>
            <button type="button" class:active={rightTab === "plans"} onclick={() => (rightTab = "plans")}>
              Sync ({sessionStore.pendingPlans.length})
            </button>
          </div>
        </div>

        {#if rightTab === "exams"}
          {#if upcomingExams.length === 0}
            <p class="empty-soft">Nessun appello futuro pianificato.</p>
          {:else}
            <div class="exam-mini-list">
              {#each upcomingExams as ex (ex.id)}
                {@const c = snap.courses.find((x) => x.id === ex.courseId)}
                <button
                  type="button"
                  class="exam-mini-row"
                  onclick={() => c && navStore.openCourseDetail(c.id)}
                >
                  <span class="ex-dot" style:background={c?.colorHex ?? "#7C5CFC"}></span>
                  <div class="ex-info">
                    <strong>{ex.title}</strong>
                    <span>{formatItalianDateShort(ex.date)}{ex.classroom ? ` · ${ex.classroom}` : ""}</span>
                  </div>
                </button>
              {/each}
            </div>
          {/if}
        {:else}
          {#if sessionStore.pendingPlans.length === 0}
            <p class="empty-soft">Tutti i file della cartella sessioni sono sincronizzati.</p>
          {:else}
            <div class="exam-mini-list">
              {#each sessionStore.pendingPlans as p (p.planId)}
                <div class="exam-mini-row">
                  <div class="ex-info">
                    <strong>{p.planName}</strong>
                    <span>Revisione {p.revision} pronta</span>
                  </div>
                  <button
                    type="button"
                    class="pill-btn pill-btn-primary btn-xs"
                    onclick={() => (navStore.pendingPlanModal = p)}
                  >
                    Importa
                  </button>
                </div>
              {/each}
            </div>
          {/if}
        {/if}
      </div>
    </aside>
  </div>
{/if}

<style>
  .home-layout {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 330px;
    gap: 18px;
    height: 100%;
    align-items: start;
  }

  .main-col {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .hero-banner {
    padding: 24px 28px;
    border-radius: var(--radius-card);
    background: var(--hero-gradient);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 18px;
    flex-wrap: wrap;
    box-shadow: var(--shadow-card);
  }

  .hero-eyebrow {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    font-weight: 700;
    opacity: 0.85;
  }

  .hero-chip {
    padding: 3px 10px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.55);
    color: #141418;
    font-weight: 700;
  }

  .hero-copy h1 {
    margin: 6px 0 4px;
    font-size: 24px;
    font-weight: 800;
    letter-spacing: -0.02em;
  }

  .hero-copy p {
    margin: 0;
    font-size: 13.5px;
    opacity: 0.85;
  }

  .hero-actions {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
  }

  .mid-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 16px;
  }

  .mid-card {
    padding: 18px 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-height: 175px;
  }

  .text-left {
    text-align: left;
  }

  .card-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    width: 100%;
  }

  .card-title {
    font-size: 14px;
    font-weight: 700;
  }

  .link-mini {
    font-size: 12px;
    font-weight: 600;
    color: var(--accent);
  }

  .tag-pill {
    font-size: 11px;
    font-weight: 600;
    padding: 3px 9px;
    border-radius: 999px;
    background: var(--bg-surface-muted);
    color: var(--text-secondary);
  }

  .courses-bubbles {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 10px;
  }

  .course-bubble {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    border-radius: 14px;
    background: var(--bg-surface-muted);
    transition: transform 0.15s ease;
  }

  .course-bubble:hover {
    transform: translateY(-1px);
  }

  .bubble-avatar {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    color: #fff;
    font-size: 10px;
    font-weight: 800;
    flex-shrink: 0;
  }

  .bubble-sub {
    font-size: 11.5px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .big-metric {
    font-size: 32px;
    font-weight: 800;
    letter-spacing: -0.03em;
    line-height: 1;
  }

  .bars-7 {
    margin-top: auto;
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 6px;
    height: 54px;
    align-items: end;
  }

  .bar-col {
    height: 100%;
    display: flex;
    align-items: flex-end;
  }

  .bar-track {
    width: 100%;
    height: 100%;
    border-radius: 999px;
    background: var(--bg-surface-muted);
    display: flex;
    align-items: flex-end;
    overflow: hidden;
  }

  .bar-fill {
    width: 100%;
    border-radius: 999px;
    background: linear-gradient(180deg, #9F85FF 0%, var(--accent) 100%);
  }

  .cat-pills-mini {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .cat-mini-row {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 12px;
  }

  .cat-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .cat-name {
    flex: 1;
    color: var(--text-secondary);
    font-weight: 600;
  }

  .cat-val {
    font-weight: 700;
  }

  .bot-grid {
    display: grid;
    grid-template-columns: 1.1fr 0.9fr;
    gap: 16px;
  }

  .stats-card {
    padding: 20px 22px;
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  .seg-switch {
    display: inline-flex;
    padding: 3px;
    border-radius: 999px;
    background: var(--bg-surface-muted);
    gap: 2px;
  }

  .seg-switch button {
    padding: 4px 10px;
    border-radius: 999px;
    font-size: 11.5px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .seg-switch button.active {
    background: var(--bg-surface-dark);
    color: #fff;
  }

  :global(html[data-theme="dark"]) .seg-switch button.active {
    background: var(--accent);
  }

  .stats-nums {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 14px;
  }

  .stat-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 12px 14px;
    border-radius: 16px;
    background: var(--bg-surface-muted);
  }

  .stat-big {
    font-size: 30px;
    font-weight: 800;
    line-height: 1;
  }

  .stat-label {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .ai-promo-card {
    padding: 20px 22px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .promo-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 11.5px;
    font-weight: 700;
    color: #DDD5FA;
  }

  .ai-promo-card h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 700;
    line-height: 1.3;
  }

  .ai-promo-card p {
    margin: 0;
    font-size: 12.5px;
    color: rgba(255, 255, 255, 0.72);
  }

  .promo-actions {
    margin-top: auto;
    padding-top: 6px;
  }

  .promo-btn {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 8px 15px;
    border-radius: 999px;
    background: #DDD5FA;
    color: #141418;
    font-size: 12.5px;
    font-weight: 700;
  }

  .right-col {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .strip-card,
  .focus-card,
  .notif-card {
    padding: 18px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .week-strip {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 4px;
  }

  .strip-day {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 8px 4px;
    border-radius: 14px;
    font-size: 12px;
    color: var(--text-secondary);
    transition: background-color 0.15s ease, color 0.15s ease;
  }

  .strip-day:hover {
    background: var(--bg-surface-muted);
    color: var(--text-primary);
  }

  .strip-day.today {
    background: var(--bg-surface-dark);
    color: #FFFFFF;
  }

  :global(html[data-theme="dark"]) .strip-day.today {
    background: var(--accent);
  }

  .strip-day .wd {
    font-size: 10.5px;
    font-weight: 600;
    opacity: 0.75;
  }

  .strip-day .dn {
    font-weight: 800;
  }

  .focus-list,
  .exam-mini-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .focus-row,
  .exam-mini-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 9px 10px;
    border-radius: 14px;
    background: var(--bg-surface-muted);
    text-align: left;
  }

  .focus-row.done {
    opacity: 0.6;
  }

  .focus-row.done strong {
    text-decoration: line-through;
  }

  .cat-stripe {
    width: 4px;
    height: 26px;
    border-radius: 999px;
    flex-shrink: 0;
  }

  .focus-text,
  .ex-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .focus-text strong,
  .ex-info strong {
    font-size: 12.5px;
    font-weight: 700;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .focus-text span,
  .ex-info span {
    font-size: 11px;
    color: var(--text-secondary);
  }

  :global(.chk-done) {
    color: var(--success);
    flex-shrink: 0;
  }

  :global(.chk-todo) {
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .quiz-banner-mini {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 8px 12px;
    border-radius: 12px;
    background: var(--accent-soft);
    color: var(--accent);
    font-size: 12px;
    font-weight: 700;
  }

  .ex-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .empty-soft {
    margin: 0;
    font-size: 12.5px;
    color: var(--text-secondary);
  }

  .btn-xs {
    padding: 5px 10px;
    font-size: 11.5px;
  }

  @media (max-width: 1140px) {
    .home-layout {
      grid-template-columns: 1fr;
    }
    .mid-grid {
      grid-template-columns: 1fr;
    }
    .bot-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
