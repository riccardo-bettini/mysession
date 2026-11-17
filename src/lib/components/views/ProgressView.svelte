<script lang="ts">
  import { Calendar, Clock, TrendingUp } from "lucide-svelte";
  import { navStore } from "../../stores/navigation.svelte";
  import { sessionStore } from "../../stores/session.svelte";
  import { themeStore } from "../../stores/theme.svelte";
  import { diffDaysIso, formatItalianDateShort, getTodayIso } from "../../utils/calendar";
  import { CORE_FIVE_CATEGORIES, getCategoryColor } from "../../utils/categories";

  let snap = $derived(sessionStore.activeSnapshot);
  let todayIso = $derived(getTodayIso());

  let upcomingExams = $derived.by(() => {
    if (!snap) return [];
    return snap.exams
      .filter((e) => e.status === "planned")
      .map((e) => ({
        exam: e,
        course: snap.courses.find((c) => c.id === e.courseId),
        daysLeft: diffDaysIso(todayIso, e.date)
      }))
      .sort((a, b) => a.exam.date.localeCompare(b.exam.date));
  });
</script>

{#if snap}
  <div class="progress-wrap bento-enter">
    <header class="page-head">
      <div>
        <h1>Progressione & Scadenze</h1>
        <p>Avanzamento del piano materia per materia, per categoria e giorni rimanenti agli esami</p>
      </div>
      <div class="global-pill">
        <TrendingUp size={16} />
        <span>Globale: <strong>{snap.telemetry.overallProgressPct}%</strong> ({snap.telemetry.completedHoursTotal}/{snap.telemetry.plannedHoursTotal}h)</span>
      </div>
    </header>

    <!-- Timeline Giorni Rimanenti agli Esami -->
    <section class="bento-card exams-countdown-card">
      <div class="sec-head">
        <h3>Giorni Rimanenti agli Esami</h3>
      </div>
      {#if upcomingExams.length === 0}
        <p class="empty-soft">Nessun appello pianificato nella sessione.</p>
      {:else}
        <div class="countdown-grid">
          {#each upcomingExams as item (item.exam.id)}
            <button
              type="button"
              class="cd-card"
              onclick={() => item.course && navStore.openCourseDetail(item.course.id)}
            >
              <div class="cd-top">
                <span class="course-pill" style:background={item.course?.colorHex ?? "#7C5CFC"}>
                  {item.course?.sigla ?? "ESAME"}
                </span>
                <span class="cd-date"><Calendar size={12} /> {formatItalianDateShort(item.exam.date)}</span>
              </div>
              <div class="cd-days">
                {#if item.daysLeft > 0}
                  <strong>-{item.daysLeft}</strong> <span>giorni</span>
                {:else if item.daysLeft === 0}
                  <strong>OGGI</strong>
                {:else}
                  <strong>Passato</strong>
                {/if}
              </div>
              <small>{item.exam.title}</small>
            </button>
          {/each}
        </div>
      {/if}
    </section>

    <!-- Percentuali di completamento materia per materia + 5 Categorie -->
    <section class="courses-prog-grid">
      {#each snap.telemetry.courseProgress as m (m.courseId)}
        <div class="bento-card cp-card">
          <div class="cp-head">
            <div class="cp-title">
              <span class="course-pill" style:background={m.colorHex}>{m.sigla}</span>
              <div>
                <h3>{m.name}</h3>
                <small>
                  {m.completedHoursTotal}/{m.plannedHoursTotal}h · {m.completedTasks}/{m.totalTasks} task
                  {#if m.daysUntilExam !== null}
                    · <Clock size={11} /> -{m.daysUntilExam} gg ({m.recommendedDailyHours}h/giorno cons.)
                  {/if}
                </small>
              </div>
            </div>
            <button type="button" class="pct-big" onclick={() => navStore.openCourseDetail(m.courseId)}>
              {m.completionPct}%
            </button>
          </div>

          <div class="main-track">
            <div class="main-fill" style:width="{m.completionPct}%" style:background={m.colorHex}></div>
          </div>

          <div class="cats-matrix">
            {#each CORE_FIVE_CATEGORIES as cat (cat.id)}
              {@const cstat = m.categoryBreakdown.find((x) => x.category === cat.id)}
              <div class="cm-item">
                <div class="cm-row">
                  <span>{cat.shortLabel}</span>
                  <strong>{cstat?.completionPct ?? 0}%</strong>
                </div>
                <div class="cm-track">
                  <div
                    class="cm-fill"
                    style:width="{cstat?.completionPct ?? 0}%"
                    style:background={getCategoryColor(cat.id, themeStore.settings)}
                  ></div>
                </div>
                <small>{cstat?.completedTasks ?? 0}/{cstat?.totalTasks ?? 0}</small>
              </div>
            {/each}
          </div>
        </div>
      {/each}
    </section>
  </div>
{/if}

<style>
  .progress-wrap {
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

  .global-pill {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 9px 16px;
    border-radius: 999px;
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    font-size: 13px;
  }

  .exams-countdown-card {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .sec-head h3 {
    margin: 0;
    font-size: 15.5px;
  }

  .countdown-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(195px, 1fr));
    gap: 12px;
  }

  .cd-card {
    padding: 14px;
    border-radius: 16px;
    background: var(--bg-surface-muted);
    display: flex;
    flex-direction: column;
    gap: 8px;
    text-align: left;
    transition: transform 0.15s ease;
  }

  .cd-card:hover {
    transform: translateY(-2px);
  }

  .cd-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
  }

  .course-pill {
    padding: 3px 9px;
    border-radius: 999px;
    color: #fff;
    font-size: 11px;
    font-weight: 800;
  }

  .cd-date {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 11.5px;
    color: var(--text-secondary);
    font-weight: 600;
  }

  .cd-days strong {
    font-size: 24px;
    font-weight: 800;
  }

  .cd-days span {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .cd-card small {
    font-size: 11.5px;
    color: var(--text-secondary);
  }

  .courses-prog-grid {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .cp-card {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .cp-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
  }

  .cp-title {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .cp-title h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 800;
  }

  .cp-title small {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .pct-big {
    font-size: 26px;
    font-weight: 800;
    color: var(--accent);
  }

  .main-track {
    height: 9px;
    border-radius: 999px;
    background: var(--bg-surface-muted);
    overflow: hidden;
  }

  .main-fill {
    height: 100%;
    border-radius: 999px;
    transition: width 0.25s ease;
  }

  .cats-matrix {
    display: grid;
    grid-template-columns: repeat(5, minmax(0, 1fr));
    gap: 10px;
  }

  .cm-item {
    padding: 10px;
    border-radius: 12px;
    background: var(--bg-surface-muted);
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .cm-row {
    display: flex;
    justify-content: space-between;
    font-size: 12px;
  }

  .cm-track {
    height: 6px;
    border-radius: 999px;
    background: var(--bg-surface);
    overflow: hidden;
  }

  .cm-fill {
    height: 100%;
    border-radius: 999px;
  }

  .cm-item small {
    font-size: 11px;
    color: var(--text-secondary);
  }

  .empty-soft {
    margin: 0;
    font-size: 13px;
    color: var(--text-secondary);
  }

  @media (max-width: 980px) {
    .cats-matrix {
      grid-template-columns: repeat(2, 1fr);
    }
  }
</style>
