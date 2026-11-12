<script lang="ts">
  import DOMPurify from "dompurify";
  import { marked } from "marked";
  import { sessionStore } from "../../stores/session.svelte";
  import { themeStore } from "../../stores/theme.svelte";
  import type { Course, Exam, GenericEvent, StudyUnit, TodoItem, TodoSubItem } from "../../types/domain";
  import {
    formatExamRoundLabel,
    formatItalianDateLong,
    type DayAggregatedData
  } from "../../utils/calendar";
  import { generateCourseColorTokens } from "../../utils/color";

  interface Props {
    isoDate: string;
    dayData: DayAggregatedData | null;
    courseById: Map<string, Course>;
    allStudyUnits: StudyUnit[];
    onClose?: () => void;
    onOpenNewExamForDate?: (isoDate: string) => void;
    onEditExam?: (exam: Exam) => void;
    onOpenNewEventForDate?: (isoDate: string) => void;
    onEditEvent?: (event: GenericEvent) => void;
  }

  let {
    isoDate,
    dayData = null,
    courseById,
    allStudyUnits = [],
    onClose,
    onOpenNewExamForDate,
    onEditExam,
    onOpenNewEventForDate,
    onEditEvent
  }: Props = $props();

  let isDark = $derived(themeStore.effectiveTheme === "dark");

  let newTodoText = $state<string>("");
  let newTodoCourseId = $state<string>("");
  let selectedUnitToAssign = $state<string>("");
  let noteMode = $state<"edit" | "preview">("edit");
  let noteDraft = $state<string>("");
  let noteSavedStatus = $state<string>("");
  let saveTimeout: ReturnType<typeof setTimeout> | null = null;

  // Stato Sotto-elementi e raggruppamento
  let expandedTodoIds = $state<Set<string>>(new Set());
  let newSubText = $state<Record<string, string>>({});
  let editingSubId = $state<string | null>(null);
  let editingSubText = $state<string>("");
  let newCourseTodoText = $state<Record<string, string>>({});

  function isTodoExpanded(id: string): boolean {
    return expandedTodoIds.has(id);
  }

  function toggleExpandTodo(id: string) {
    const next = new Set(expandedTodoIds);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    expandedTodoIds = next;
  }

  async function handleAddSubtask(todoId: string, e?: Event) {
    if (e) e.preventDefault();
    const text = (newSubText[todoId] || "").trim();
    if (!text) return;

    newSubText[todoId] = "";
    const next = new Set(expandedTodoIds);
    next.add(todoId);
    expandedTodoIds = next;

    await sessionStore.upsertTodoSubItem({
      todoId,
      text,
      done: false
    });
  }

  async function handleToggleSubtask(subId: string) {
    await sessionStore.toggleTodoSubItem(subId);
  }

  async function handleDeleteSubtask(subId: string) {
    await sessionStore.deleteTodoSubItem(subId);
  }

  function startEditSubtask(sub: TodoSubItem) {
    editingSubId = sub.id;
    editingSubText = sub.text;
  }

  async function saveEditSubtask(todoId: string, subId: string) {
    const text = editingSubText.trim();
    if (text && text !== "") {
      await sessionStore.upsertTodoSubItem({
        id: subId,
        todoId,
        text
      });
    }
    editingSubId = null;
    editingSubText = "";
  }

  function cancelEditSubtask() {
    editingSubId = null;
    editingSubText = "";
  }

  async function handleAddTodoForCourse(courseId: string | null, e?: Event) {
    if (e) e.preventDefault();
    const key = courseId || "general";
    const text = (newCourseTodoText[key] || "").trim();
    const sessionId = sessionStore.activeSession?.id;
    if (!text || !sessionId) return;

    newCourseTodoText[key] = "";
    await sessionStore.upsertTodo({
      sessionId,
      courseId: courseId || null,
      date: isoDate,
      text,
      priority: 1
    });
  }

  // Sincronizza la bozza della nota quando cambia la data selezionata
  $effect(() => {
    const _date = isoDate;
    const existing = dayData?.note?.contentMarkdown ?? "";
    if (_date) {
      noteDraft = existing;
      noteSavedStatus = "";
    }
  });

  let exams = $derived(dayData?.exams ?? []);
  let events = $derived(dayData?.genericEvents ?? []);
  let assignedUnits = $derived(dayData?.studyUnits ?? []);
  let todos = $derived(dayData?.todos ?? []);
  let todosDoneCount = $derived(todos.filter((t) => t.done).length);

  interface CourseTodoGroup {
    courseId: string | null;
    course: Course | null;
    todos: TodoItem[];
  }

  let groupedTodos = $derived.by(() => {
    const list = todos;
    const map = new Map<string, TodoItem[]>();
    const general: TodoItem[] = [];

    for (const t of list) {
      if (t.courseId) {
        const arr = map.get(t.courseId) ?? [];
        arr.push(t);
        map.set(t.courseId, arr);
      } else {
        general.push(t);
      }
    }

    const groups: CourseTodoGroup[] = [];
    for (const [cid, cTodos] of map.entries()) {
      groups.push({
        courseId: cid,
        course: courseById.get(cid) ?? null,
        todos: cTodos
      });
    }

    if (general.length > 0) {
      groups.push({
        courseId: null,
        course: null,
        todos: general
      });
    }

    return groups;
  });
  let conflicts = $derived(dayData?.conflicts ?? []);

  let assignableUnits = $derived(
    allStudyUnits.filter((u) => u.assignedDate !== isoDate)
  );

  let arrears = $derived.by(() => {
    const snap = sessionStore.activeSnapshot;
    if (!snap) return [];
    return snap.todos
      .filter((t) => {
        if (t.done) return false;
        if (!t.date || t.date >= isoDate) return false;
        if (t.courseId) {
          const c = courseById.get(t.courseId);
          if (c && c.isVisible === false) return false;
        }
        return true;
      })
      .sort((a, b) => (a.date < b.date ? -1 : 1));
  });

  let renderedMarkdownHtml = $derived.by(() => {
    const raw = (noteDraft || "").trim();
    if (!raw) return "<p class='empty-md'>Nessuna nota scritta per questa giornata.</p>";
    const parsed = marked.parse(raw, { async: false }) as string;
    return DOMPurify.sanitize(parsed);
  });

  async function handleAddTodo(e: SubmitEvent) {
    e.preventDefault();
    const clean = newTodoText.trim();
    const sessionId = sessionStore.activeSession?.id;
    if (!clean || !sessionId) return;

    newTodoText = "";
    await sessionStore.upsertTodo({
      sessionId,
      courseId: newTodoCourseId || null,
      date: isoDate,
      text: clean,
      priority: 1
    });
  }

  async function handleAssignUnit() {
    if (!selectedUnitToAssign) return;
    const unitId = selectedUnitToAssign;
    selectedUnitToAssign = "";
    await sessionStore.assignStudyUnitDate(unitId, isoDate);
  }

  function handleNoteInput(e: Event) {
    noteDraft = (e.currentTarget as HTMLTextAreaElement).value;
    noteSavedStatus = "Modifiche in corso…";
    if (saveTimeout) clearTimeout(saveTimeout);
    saveTimeout = setTimeout(() => {
      persistNoteNow();
    }, 650);
  }

  async function persistNoteNow() {
    if (saveTimeout) clearTimeout(saveTimeout);
    await sessionStore.saveDailyNote(isoDate, noteDraft);
    noteSavedStatus = "Salvata in SQLite";
  }
</script>

<aside class="day-inspector-drawer" aria-label="Pannello ispettore giornata {isoDate}">
  <!-- Header Ispettore Giornaliero -->
  <header class="drawer-header">
    <div class="header-titles">
      <span class="drawer-eyebrow">ISPETTORE GIORNATA</span>
      <h2 class="drawer-date-title">{formatItalianDateLong(isoDate)}</h2>
      <div class="header-badges-row">
        {#if dayData?.holiday}
          <span class="holiday-pill" style:color={themeStore.settings.holidayColor}>
            🎉 {dayData.holiday.name}
          </span>
        {/if}
        {#if dayData && dayData.workloadHours > 0}
          <span class="workload-pill">
            ⏱ Studio consigliato: {dayData.workloadHours.toFixed(1)}h
          </span>
        {/if}
      </div>
    </div>

    <button
      type="button"
      class="drawer-close-btn pressable"
      title="Chiudi pannello giornata (Esc)"
      aria-label="Chiudi ispettore giornata"
      onclick={() => onClose?.()}
    >
      ✕
    </button>
  </header>

  <div class="drawer-scroll-content">
    <!-- Alert Conflitti sulla Data -->
    {#if conflicts.length > 0}
      <section class="drawer-conflict-box">
        {#each conflicts as cf (cf.id)}
          <div class="cf-alert-item" class:critical={cf.severity === "critical"}>
            <strong>⚠ {cf.title}</strong>
            <p>{cf.description}</p>
            <small>💡 {cf.suggestion}</small>
          </div>
        {/each}
      </section>
    {/if}

    <!-- 1. APPELLI & PROVE D'ESAME DEL GIORNO -->
    <section class="drawer-section">
      <div class="section-head">
        <h3>Appelli & Prove ({exams.length})</h3>
        <button
          type="button"
          class="section-add-btn pressable"
          onclick={() => onOpenNewExamForDate?.(isoDate)}
        >
          + Appello
        </button>
      </div>

      {#if exams.length === 0}
        <p class="section-empty-hint">Nessun appello fissato in questa data.</p>
      {:else}
        <div class="items-list">
          {#each exams as exam (exam.id)}
            {@const course = courseById.get(exam.courseId)}
            {@const palette = generateCourseColorTokens(course?.colorHex ?? "#007AFF", isDark)}
            <div
              class="drawer-exam-card"
              style:background={palette.bgTint}
              style:border-color={palette.borderTint}
            >
              <div class="dec-main">
                <div class="dec-badges">
                  <span
                    class="dec-sigla"
                    style:background={palette.solid}
                    style:color={palette.badgeText}
                  >
                    {course?.sigla ?? "ESAME"}
                  </span>
                  <span class="dec-round">{formatExamRoundLabel(exam.round)}</span>
                  {#if exam.timeSlot}
                    <span class="dec-time">🕒 {exam.timeSlot}</span>
                  {/if}
                </div>
                <strong class="dec-title">{course?.name ?? exam.title}</strong>
                {#if exam.classroom}
                  <span class="dec-sub">📍 Aula: {exam.classroom}</span>
                {/if}
              </div>

              <div class="dec-actions">
                <button
                  type="button"
                  class="icon-btn pressable"
                  title="Modifica appello"
                  onclick={() => onEditExam?.(exam)}
                >
                  ✎
                </button>
                <button
                  type="button"
                  class="icon-btn danger pressable"
                  title="Elimina appello"
                  onclick={() => sessionStore.deleteExam(exam.id, false)}
                >
                  🗑
                </button>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </section>

    <!-- 2. IMPEGNI & EVENTI GENERICI -->
    <section class="drawer-section">
      <div class="section-head">
        <h3>Impegni & Scadenze ({events.length})</h3>
        <button
          type="button"
          class="section-add-btn pressable"
          onclick={() => onOpenNewEventForDate?.(isoDate)}
        >
          + Impegno
        </button>
      </div>

      {#if events.length === 0}
        <p class="section-empty-hint">Nessun impegno generico registrato.</p>
      {:else}
        <div class="items-list">
          {#each events as ev (ev.id)}
            <div class="drawer-generic-row">
              <div class="dgr-info">
                <span class="dgr-cat">{ev.category}</span>
                <strong>{ev.title}</strong>
                {#if ev.timeSlot}
                  <small>🕒 {ev.timeSlot}</small>
                {/if}
              </div>
              <div class="dec-actions">
                <button
                  type="button"
                  class="icon-btn pressable"
                  title="Modifica impegno"
                  onclick={() => onEditEvent?.(ev)}
                >
                  ✎
                </button>
                <button
                  type="button"
                  class="icon-btn danger pressable"
                  title="Elimina impegno"
                  onclick={() => sessionStore.deleteGenericEvent(ev.id)}
                >
                  🗑
                </button>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </section>

    <!-- 3. TO-DO LIST GIORNALIERA INTERATTIVA -->
    <section class="drawer-section">
      <div class="section-head">
        <h3>To-Do Giornalieri ({todosDoneCount}/{todos.length})</h3>
      </div>

      {#if todos.length > 0}
        <div class="todo-progress-bar">
          <div
            class="todo-progress-fill"
            style:width="{Math.round((todosDoneCount / todos.length) * 100)}%"
          ></div>
        </div>
      {/if}

      {#if todos.length === 0}
        <p class="section-empty-hint">Nessun task registrato per questa data.</p>
      {:else}
        <div class="subject-todo-blocks">
          {#each groupedTodos as grp (grp.courseId ?? "general")}
            {@const c = grp.course}
            {@const palette = c ? generateCourseColorTokens(c.colorHex, isDark) : null}
            {@const grpDone = grp.todos.filter((t) => t.done).length}
            <div
              class="subject-todo-card"
              style:background={palette ? palette.bgTint : "var(--surface-card)"}
              style:border-color={palette ? palette.borderTint : "var(--hairline-border)"}
            >
              <!-- Intestazione Scheda Materia -->
              <div class="stc-header">
                <div class="stc-title-group">
                  {#if c}
                    <span class="stc-bar" style:background={palette?.solid}></span>
                    <span
                      class="stc-sigla"
                      style:background={palette?.solid}
                      style:color={palette?.badgeText}
                    >
                      {c.sigla}
                    </span>
                    <span class="stc-name" style:color={palette?.textContrast}>{c.name}</span>
                  {:else}
                    <span class="stc-bar generic"></span>
                    <span class="stc-sigla generic">GEN</span>
                    <span class="stc-name">Attività Generali</span>
                  {/if}
                </div>
                <span class="stc-count">{grpDone}/{grp.todos.length}</span>
              </div>

              <!-- Lista Task della Materia -->
              <div class="stc-items-list">
                {#each grp.todos as td (td.id)}
                  {@const subCount = td.subtasks?.length ?? 0}
                  {@const subDoneCount = td.subtasks?.filter((s) => s.done).length ?? 0}
                  {@const isExpanded = isTodoExpanded(td.id)}
                  <div class="todo-task-wrapper" class:done={td.done} class:expanded={isExpanded}>
                    <!-- Riga Task Principale -->
                    <div class="todo-item-row" class:done={td.done}>
                      <input
                        type="checkbox"
                        checked={td.done}
                        onchange={(e) =>
                          sessionStore.toggleTodo(
                            td.id,
                            (e.currentTarget as HTMLInputElement).checked
                          )}
                      />
                      <span class="todo-text">{td.text}</span>

                      <!-- Badge Contatore Sotto-elementi / Trigger Fisarmonica -->
                      {#if subCount > 0}
                        <button
                          type="button"
                          class="subtasks-toggle-btn pressable"
                          class:all-done={subDoneCount === subCount}
                          title="{isExpanded ? 'Comprimi' : 'Espandi'} sotto-elementi ({subDoneCount}/{subCount})"
                          onclick={() => toggleExpandTodo(td.id)}
                        >
                          <span class="st-fraction">{subDoneCount}/{subCount}</span>
                          <span class="st-chevron">{isExpanded ? "▾" : "▸"}</span>
                        </button>
                      {:else}
                        <button
                          type="button"
                          class="subtasks-add-action-btn pressable"
                          title="Aggiungi sotto-elementi"
                          onclick={() => toggleExpandTodo(td.id)}
                        >
                          + Sotto-task
                        </button>
                      {/if}

                      <button
                        type="button"
                        class="todo-del-btn"
                        title="Rimuovi task"
                        onclick={() => sessionStore.deleteTodo(td.id)}
                      >
                        ✕
                      </button>
                    </div>

                    <!-- Fisarmonica Sotto-elementi Espansa -->
                    {#if isExpanded}
                      <div class="subtasks-container">
                        {#if td.subtasks && td.subtasks.length > 0}
                          <div class="subtasks-list">
                            {#each td.subtasks as sub (sub.id)}
                              <div class="subtask-row" class:done={sub.done}>
                                <input
                                  type="checkbox"
                                  checked={sub.done}
                                  onchange={() => handleToggleSubtask(sub.id)}
                                />
                                {#if editingSubId === sub.id}
                                  <!-- svelte-ignore a11y_autofocus -->
                                  <input
                                    type="text"
                                    class="subtask-edit-input"
                                    bind:value={editingSubText}
                                    onkeydown={(e) => {
                                      if (e.key === "Enter") saveEditSubtask(td.id, sub.id);
                                      if (e.key === "Escape") cancelEditSubtask();
                                    }}
                                    onblur={() => saveEditSubtask(td.id, sub.id)}
                                    autofocus
                                  />
                                {:else}
                                  <!-- svelte-ignore a11y_no_static_element_interactions -->
                                  <span
                                    class="subtask-text"
                                    title="Doppio-click per modificare"
                                    ondblclick={() => startEditSubtask(sub)}
                                  >
                                    {sub.text}
                                  </span>
                                {/if}
                                <button
                                  type="button"
                                  class="subtask-del-btn"
                                  title="Rimuovi sotto-elemento"
                                  onclick={() => handleDeleteSubtask(sub.id)}
                                >
                                  ✕
                                </button>
                              </div>
                            {/each}
                          </div>
                        {/if}

                        <!-- Input inline rapido per nuovi sotto-elementi -->
                        <form class="subtask-inline-add" onsubmit={(e) => handleAddSubtask(td.id, e)}>
                          <input
                            type="text"
                            class="subtask-inline-input"
                            placeholder="Aggiungi sotto-elemento… (Invio)"
                            bind:value={newSubText[td.id]}
                          />
                          <button type="submit" class="subtask-inline-submit pressable">+</button>
                        </form>
                      </div>
                    {/if}
                  </div>
                {/each}
              </div>

              <!-- Input rapido task dedicato alla materia -->
              <form class="stc-quick-add" onsubmit={(e) => handleAddTodoForCourse(grp.courseId, e)}>
                <input
                  type="text"
                  class="stc-quick-input"
                  placeholder="+ Aggiungi task a {c?.sigla ?? 'Generale'}…"
                  bind:value={newCourseTodoText[grp.courseId || 'general']}
                />
                <button type="submit" class="stc-quick-btn pressable" title="Aggiungi task">+</button>
              </form>
            </div>
          {/each}
        </div>
      {/if}

      <!-- Form rapido per inserire un nuovo To-Do con selettore corso -->
      <form class="todo-quick-form" onsubmit={handleAddTodo}>
        <input
          type="text"
          class="todo-input"
          placeholder="Aggiungi task per questa data…"
          bind:value={newTodoText}
        />
        {#if sessionStore.courses.length > 0}
          <select class="todo-course-select" bind:value={newTodoCourseId} aria-label="Associa materia">
            <option value="">Generale</option>
            {#each sessionStore.courses as c (c.id)}
              <option value={c.id}>{c.sigla}</option>
            {/each}
          </select>
        {/if}
        <button type="submit" class="todo-submit-btn pressable">+</button>
      </form>
    </section>

    <!-- 3.1 ARRETRATI DA RECUPERARE (Task precedenti non completati) -->
    {#if arrears.length > 0}
      <section class="drawer-section drawer-arrears-section">
        <div class="section-head">
          <div class="arrears-head-title">
            <span class="arrears-badge">⚠ {arrears.length}</span>
            <h3>Arretrati da recuperare</h3>
          </div>
        </div>
        <p class="section-sub-hint">Attività pianificate nei giorni scorsi e non ancora completate.</p>

        <div class="items-list">
          {#each arrears as td (td.id)}
            {@const c = td.courseId ? courseById.get(td.courseId) : null}
            {@const subCount = td.subtasks?.length ?? 0}
            {@const subDoneCount = td.subtasks?.filter((s) => s.done).length ?? 0}
            {@const isExpanded = isTodoExpanded(td.id)}
            <div class="todo-task-wrapper arrear-task-wrapper" class:done={td.done} class:expanded={isExpanded}>
              <div class="todo-item-row arrear-item-row">
                <input
                  type="checkbox"
                  checked={td.done}
                  onchange={(e) =>
                    sessionStore.toggleTodo(
                      td.id,
                      (e.currentTarget as HTMLInputElement).checked
                    )}
                />
                {#if c}
                  <span class="todo-course-tag" style:background={c.colorHex}>{c.sigla}</span>
                {/if}
                <div class="arrear-content">
                  <span class="todo-text">{td.text}</span>
                  <span class="arrear-date-tag">📅 {td.date}</span>
                </div>

                {#if subCount > 0}
                  <button
                    type="button"
                    class="subtasks-toggle-btn pressable"
                    class:all-done={subDoneCount === subCount}
                    title="{isExpanded ? 'Comprimi' : 'Espandi'} sotto-elementi ({subDoneCount}/{subCount})"
                    onclick={() => toggleExpandTodo(td.id)}
                  >
                    <span class="st-fraction">{subDoneCount}/{subCount}</span>
                    <span class="st-chevron">{isExpanded ? "▾" : "▸"}</span>
                  </button>
                {:else}
                  <button
                    type="button"
                    class="subtasks-add-action-btn pressable"
                    title="Aggiungi sotto-elementi"
                    onclick={() => toggleExpandTodo(td.id)}
                  >
                    + Sotto-task
                  </button>
                {/if}

                <button
                  type="button"
                  class="todo-del-btn"
                  title="Rimuovi task"
                  onclick={() => sessionStore.deleteTodo(td.id)}
                >
                  ✕
                </button>
              </div>

              <!-- Fisarmonica sotto-elementi negli arretrati -->
              {#if isExpanded}
                <div class="subtasks-container">
                  {#if td.subtasks && td.subtasks.length > 0}
                    <div class="subtasks-list">
                      {#each td.subtasks as sub (sub.id)}
                        <div class="subtask-row" class:done={sub.done}>
                          <input
                            type="checkbox"
                            checked={sub.done}
                            onchange={() => handleToggleSubtask(sub.id)}
                          />
                          {#if editingSubId === sub.id}
                            <!-- svelte-ignore a11y_autofocus -->
                            <input
                              type="text"
                              class="subtask-edit-input"
                              bind:value={editingSubText}
                              onkeydown={(e) => {
                                if (e.key === "Enter") saveEditSubtask(td.id, sub.id);
                                if (e.key === "Escape") cancelEditSubtask();
                              }}
                              onblur={() => saveEditSubtask(td.id, sub.id)}
                              autofocus
                            />
                          {:else}
                            <!-- svelte-ignore a11y_no_static_element_interactions -->
                            <span
                              class="subtask-text"
                              title="Doppio-click per modificare"
                              ondblclick={() => startEditSubtask(sub)}
                            >
                              {sub.text}
                            </span>
                          {/if}
                          <button
                            type="button"
                            class="subtask-del-btn"
                            title="Rimuovi sotto-elemento"
                            onclick={() => handleDeleteSubtask(sub.id)}
                          >
                            ✕
                          </button>
                        </div>
                      {/each}
                    </div>
                  {/if}

                  <form class="subtask-inline-add" onsubmit={(e) => handleAddSubtask(td.id, e)}>
                    <input
                      type="text"
                      class="subtask-inline-input"
                      placeholder="Aggiungi sotto-elemento… (Invio)"
                      bind:value={newSubText[td.id]}
                    />
                    <button type="submit" class="subtask-inline-submit pressable">+</button>
                  </form>
                </div>
              {/if}
            </div>
          {/each}
        </div>
      </section>
    {/if}

    <!-- 4. UNITÀ DIDATTICHE PIANIFICATE -->
    <section class="drawer-section">
      <div class="section-head">
        <h3>Unità Didattiche ({assignedUnits.length})</h3>
      </div>

      {#if assignableUnits.length > 0}
        <div class="assign-unit-bar">
          <select
            class="unit-select"
            bind:value={selectedUnitToAssign}
            aria-label="Assegna unità didattica a questa data"
          >
            <option value="">Assegna unità didattica a questo giorno…</option>
            {#each assignableUnits as u (u.id)}
              {@const c = courseById.get(u.courseId)}
              <option value={u.id}>
                [{c?.sigla ?? "Corso"}] U#{u.unitNumber} — {u.title} ({u.estimatedHours}h)
              </option>
            {/each}
          </select>
          <button
            type="button"
            class="todo-submit-btn pressable"
            disabled={!selectedUnitToAssign}
            onclick={handleAssignUnit}
          >
            Assegna
          </button>
        </div>
      {/if}

      {#if assignedUnits.length === 0}
        <p class="section-empty-hint">Nessuna unità didattica assegnata a questa giornata.</p>
      {:else}
        <div class="items-list">
          {#each assignedUnits as u (u.id)}
            {@const c = courseById.get(u.courseId)}
            <div class="unit-drawer-row" class:done={u.isCompleted}>
              <input
                type="checkbox"
                checked={u.isCompleted}
                onchange={(e) =>
                  sessionStore.toggleStudyUnit(
                    u.id,
                    (e.currentTarget as HTMLInputElement).checked
                  )}
              />
              <div class="unit-info">
                <span class="unit-badge" style:background={c?.colorHex ?? "#007AFF"}>
                  {c?.sigla ?? "U"} #{u.unitNumber}
                </span>
                <span class="unit-title">{u.title}</span>
                <small>{u.estimatedHours}h</small>
              </div>
              <button
                type="button"
                class="todo-del-btn"
                title="Scollega da questa data"
                onclick={() => sessionStore.assignStudyUnitDate(u.id, null)}
              >
                ✕
              </button>
            </div>
          {/each}
        </div>
      {/if}
    </section>

    <!-- 5. NOTE PERSONALI IN MARKDOWN -->
    <section class="drawer-section">
      <div class="section-head">
        <h3>Note Giornaliere (Markdown)</h3>
        <div class="md-mode-tabs">
          <button
            type="button"
            class="md-tab"
            class:active={noteMode === "edit"}
            onclick={() => (noteMode = "edit")}
          >
            Editor
          </button>
          <button
            type="button"
            class="md-tab"
            class:active={noteMode === "preview"}
            onclick={() => (noteMode = "preview")}
          >
            Anteprima
          </button>
        </div>
      </div>

      {#if noteMode === "edit"}
        <textarea
          class="markdown-textarea"
          rows="5"
          placeholder="Scrivi appunti in Markdown (es. **ripasso**, liste `- argomento`, formule)…"
          value={noteDraft}
          oninput={handleNoteInput}
          onblur={persistNoteNow}
        ></textarea>
      {:else}
        <div class="markdown-preview-box">
          {@html renderedMarkdownHtml}
        </div>
      {/if}

      <div class="note-footer-bar">
        <span class="note-status">{noteSavedStatus}</span>
        <button type="button" class="save-note-btn pressable" onclick={persistNoteNow}>
          Salva Nota
        </button>
      </div>
    </section>
  </div>
</aside>

<style>
  .day-inspector-drawer {
    position: fixed;
    top: 3.8rem;
    right: 1.25rem;
    bottom: 1.25rem;
    width: min(410px, calc(100vw - 2.5rem));
    height: auto;
    max-height: calc(100vh - 5.05rem);
    display: flex;
    flex-direction: column;
    background: var(--popover-bg);
    backdrop-filter: blur(36px) saturate(200%);
    -webkit-backdrop-filter: blur(36px) saturate(200%);
    border: 0.5px solid var(--hairline-border-strong);
    border-radius: var(--radius-xl);
    box-shadow: 0 16px 44px -6px rgba(0, 0, 0, 0.36), 0 0 0 0.5px rgba(255, 255, 255, 0.08);
    z-index: 125;
    overflow: hidden;
    animation: floatingDrawerEnter 240ms cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes floatingDrawerEnter {
    from {
      opacity: 0;
      transform: translateX(24px) scale(0.97);
    }
    to {
      opacity: 1;
      transform: translateX(0) scale(1);
    }
  }

  .drawer-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.95rem 1.15rem 0.85rem;
    border-bottom: 0.5px solid var(--hairline-border);
  }

  .drawer-eyebrow {
    font-size: 0.62rem;
    font-weight: 700;
    letter-spacing: var(--tracking-caps);
    color: var(--text-tertiary);
  }

  .drawer-date-title {
    margin: 0.15rem 0 0.35rem;
    font-size: 1.05rem;
    font-weight: 800;
    letter-spacing: var(--tracking-tight);
    text-transform: capitalize;
  }

  .header-badges-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
  }

  .holiday-pill,
  .workload-pill {
    font-size: 0.65rem;
    font-weight: 700;
    padding: 0.14rem 0.48rem;
    border-radius: var(--radius-pill);
    background: var(--surface-pill);
    border: 0.5px solid var(--hairline-border);
    letter-spacing: var(--tracking-normal);
  }

  .workload-pill {
    color: var(--accent-primary);
  }

  .drawer-close-btn {
    width: 1.85rem;
    height: 1.85rem;
    border-radius: var(--radius-pill);
    border: 0.5px solid var(--hairline-border);
    background: var(--surface-pill);
    color: var(--text-secondary);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    flex-shrink: 0;
    transition: transform 120ms var(--spring-press), background-color 130ms ease;
  }

  .drawer-close-btn:hover {
    background: var(--surface-pill-hover);
    color: var(--text-primary);
  }

  .drawer-close-btn:active {
    transform: scale(0.92);
  }

  .drawer-scroll-content {
    flex: 1;
    overflow-y: auto;
    padding: 0.95rem 1.15rem 1.35rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .drawer-conflict-box {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .cf-alert-item {
    padding: 0.6rem 0.75rem;
    border-radius: var(--radius-sm);
    background: rgba(255, 159, 10, 0.14);
    border: 0.5px solid rgba(255, 159, 10, 0.4);
    font-size: 0.73rem;
    display: flex;
    flex-direction: column;
    gap: 0.22rem;
  }

  .cf-alert-item.critical {
    background: rgba(255, 69, 58, 0.16);
    border-color: rgba(255, 69, 58, 0.45);
  }

  .cf-alert-item p {
    margin: 0;
    color: var(--text-secondary);
  }

  .drawer-section {
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
    padding-bottom: 0.95rem;
    border-bottom: 0.5px solid var(--hairline-border);
  }

  .section-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .section-head h3 {
    margin: 0;
    font-size: 0.72rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: var(--tracking-caps);
    color: var(--text-tertiary);
  }

  .section-add-btn {
    padding: 0.2rem 0.6rem;
    border-radius: var(--radius-xs);
    border: 0.5px solid var(--hairline-border);
    background: var(--surface-pill);
    box-shadow: var(--shadow-pill);
    color: var(--accent-primary);
    font-size: 0.68rem;
    font-weight: 700;
    cursor: pointer;
  }

  .section-empty-hint {
    margin: 0;
    font-size: 0.74rem;
    color: var(--text-tertiary);
  }

  .items-list {
    display: flex;
    flex-direction: column;
    gap: 0.42rem;
  }

  .drawer-exam-card,
  .drawer-generic-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.6rem 0.75rem;
    border-radius: var(--radius-sm);
    border: 0.5px solid var(--hairline-border);
    box-shadow: var(--shadow-pill);
    background: var(--surface-card);
  }

  .dec-main,
  .dgr-info {
    display: flex;
    flex-direction: column;
    gap: 0.18rem;
    min-width: 0;
  }

  .dec-badges {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    flex-wrap: wrap;
  }

  .dec-sigla {
    font-size: 0.62rem;
    font-weight: 800;
    padding: 0.1rem 0.38rem;
    border-radius: 0.28rem;
    letter-spacing: var(--tracking-wide);
  }

  .dec-round,
  .dec-time,
  .dgr-cat {
    font-size: 0.64rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .dec-title {
    font-size: 0.8rem;
    font-weight: 700;
    letter-spacing: var(--tracking-normal);
  }

  .dec-sub {
    font-size: 0.68rem;
    color: var(--text-secondary);
  }

  .dec-actions {
    display: flex;
    gap: 0.25rem;
    flex-shrink: 0;
  }

  .icon-btn {
    width: 1.75rem;
    height: 1.75rem;
    border-radius: var(--radius-xs);
    border: 0.5px solid var(--hairline-border);
    background: var(--surface-pill);
    color: var(--text-secondary);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    font-size: 0.74rem;
  }

  .icon-btn.danger:hover {
    background: rgba(255, 69, 58, 0.2);
    color: #ff453a;
  }

  /* To-Do List */
  .todo-progress-bar {
    width: 100%;
    height: 4px;
    border-radius: var(--radius-pill);
    background: var(--surface-pill);
    overflow: hidden;
  }

  .todo-progress-fill {
    height: 100%;
    background: #34c759;
    transition: width 240ms var(--ease-out);
  }

  .todo-quick-form,
  .assign-unit-bar {
    display: flex;
    gap: 0.35rem;
  }

  .todo-input,
  .todo-course-select,
  .unit-select {
    height: 1.95rem;
    padding: 0 0.6rem;
    border-radius: var(--radius-xs);
    border: 0.5px solid var(--hairline-border-strong);
    background: var(--surface-card);
    color: var(--text-primary);
    font-size: 0.75rem;
  }

  .todo-course-select option,
  .unit-select option {
    background: var(--popover-bg);
    color: var(--text-primary);
  }

  .todo-input,
  .unit-select {
    flex: 1;
    min-width: 0;
  }

  .todo-submit-btn {
    height: 1.95rem;
    padding: 0 0.75rem;
    border-radius: var(--radius-xs);
    border: none;
    background: var(--accent-primary);
    color: #fff;
    font-size: 0.74rem;
    font-weight: 700;
    cursor: pointer;
    box-shadow: 0 1px 3px rgba(10, 132, 255, 0.3);
  }

  .todo-item-row,
  .unit-drawer-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.45rem 0.55rem;
    border-radius: var(--radius-xs);
    background: var(--surface-card);
    border: 0.5px solid var(--hairline-border);
    box-shadow: var(--shadow-pill);
    font-size: 0.75rem;
  }

  .todo-item-row.done .todo-text,
  .unit-drawer-row.done .unit-title {
    text-decoration: line-through;
    opacity: 0.55;
  }

  .todo-course-tag,
  .unit-badge {
    font-size: 0.6rem;
    font-weight: 800;
    color: #fff;
    padding: 0.08rem 0.35rem;
    border-radius: 0.28rem;
    flex-shrink: 0;
    letter-spacing: var(--tracking-wide);
  }

  .todo-text,
  .unit-info {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }

  .unit-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .todo-del-btn {
    border: none;
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
    font-size: 0.74rem;
  }

  /* Schede Materie To-Do */
  .subject-todo-blocks {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    margin-top: 0.35rem;
    margin-bottom: 0.5rem;
  }

  .subject-todo-card {
    border-radius: var(--radius-sm);
    border: 0.5px solid var(--hairline-border);
    padding: 0.65rem 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    box-shadow: var(--shadow-card, 0 1px 3px rgba(0, 0, 0, 0.05));
    transition: background 180ms ease, border-color 180ms ease;
  }

  .stc-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .stc-title-group {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    min-width: 0;
  }

  .stc-bar {
    width: 3.5px;
    height: 1.1rem;
    border-radius: 2px;
    flex-shrink: 0;
  }

  .stc-bar.generic {
    background: var(--text-tertiary);
  }

  .stc-sigla {
    font-size: 0.64rem;
    font-weight: 800;
    padding: 0.12rem 0.4rem;
    border-radius: 0.3rem;
    letter-spacing: var(--tracking-wide);
    flex-shrink: 0;
  }

  .stc-sigla.generic {
    background: var(--surface-pill);
    color: var(--text-secondary);
    border: 0.5px solid var(--hairline-border);
  }

  .stc-name {
    font-size: 0.8rem;
    font-weight: 700;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .stc-count {
    font-size: 0.68rem;
    font-weight: 600;
    color: var(--text-tertiary);
    flex-shrink: 0;
  }

  .stc-items-list {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .stc-quick-add {
    display: flex;
    gap: 0.35rem;
    margin-top: 0.2rem;
  }

  .stc-quick-input {
    flex: 1;
    height: 1.8rem;
    padding: 0 0.55rem;
    font-size: 0.72rem;
    border-radius: var(--radius-xs);
    border: 0.5px solid var(--hairline-border);
    background: var(--surface-card);
    color: var(--text-primary);
  }

  .stc-quick-btn {
    height: 1.8rem;
    padding: 0 0.65rem;
    border-radius: var(--radius-xs);
    border: none;
    background: var(--accent-primary);
    color: #fff;
    font-size: 0.72rem;
    font-weight: 700;
    cursor: pointer;
  }

  /* Task Wrapper & Sotto-elementi */
  .todo-task-wrapper {
    display: flex;
    flex-direction: column;
    border-radius: var(--radius-xs);
    border: 0.5px solid var(--hairline-border);
    background: var(--surface-card);
    overflow: hidden;
    transition: border-color 150ms ease;
  }

  .todo-task-wrapper.expanded {
    border-color: var(--hairline-border-strong);
  }

  .subtasks-toggle-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.64rem;
    font-weight: 700;
    padding: 0.12rem 0.45rem;
    border-radius: var(--radius-pill);
    border: 0.5px solid var(--hairline-border);
    background: var(--surface-pill);
    color: var(--text-secondary);
    cursor: pointer;
    flex-shrink: 0;
    transition: all 140ms ease;
  }

  .subtasks-toggle-btn:hover {
    background: var(--surface-card);
    color: var(--text-primary);
  }

  .subtasks-toggle-btn.all-done {
    background: rgba(52, 199, 89, 0.16);
    color: #34c759;
    border-color: rgba(52, 199, 89, 0.35);
  }

  .st-chevron {
    font-size: 0.68rem;
  }

  .subtasks-add-action-btn {
    font-size: 0.64rem;
    font-weight: 600;
    color: var(--text-tertiary);
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 0.1rem 0.35rem;
    border-radius: var(--radius-xs);
    flex-shrink: 0;
    transition: color 140ms ease;
  }

  .subtasks-add-action-btn:hover {
    color: var(--accent-primary);
    background: var(--surface-pill);
  }

  .subtasks-container {
    margin-left: 1.45rem;
    padding-left: 0.65rem;
    padding-right: 0.55rem;
    border-left: 1.5px solid var(--hairline-border-strong);
    margin-top: 0.25rem;
    margin-bottom: 0.45rem;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .subtasks-list {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .subtask-row {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    padding: 0.28rem 0.45rem;
    border-radius: var(--radius-xs);
    background: var(--surface-pill);
    font-size: 0.72rem;
    transition: background 150ms ease;
  }

  .subtask-row.done .subtask-text {
    text-decoration: line-through;
    opacity: 0.55;
  }

  .subtask-text {
    flex: 1;
    min-width: 0;
    color: var(--text-primary);
    cursor: pointer;
    user-select: text;
  }

  .subtask-edit-input {
    flex: 1;
    height: 1.55rem;
    font-size: 0.72rem;
    padding: 0 0.4rem;
    border-radius: 0.25rem;
    border: 1px solid var(--accent-primary);
    background: var(--surface-card);
    color: var(--text-primary);
  }

  .subtask-del-btn {
    background: transparent;
    border: none;
    font-size: 0.65rem;
    color: var(--text-tertiary);
    cursor: pointer;
    padding: 0.1rem;
    flex-shrink: 0;
  }

  .subtask-del-btn:hover {
    color: #ff453a;
  }

  .subtask-inline-add {
    display: flex;
    gap: 0.25rem;
    margin-top: 0.15rem;
  }

  .subtask-inline-input {
    flex: 1;
    height: 1.6rem;
    padding: 0 0.45rem;
    font-size: 0.7rem;
    border-radius: var(--radius-xs);
    border: 0.5px dashed var(--hairline-border-strong);
    background: transparent;
    color: var(--text-primary);
  }

  .subtask-inline-input:focus {
    border-style: solid;
    border-color: var(--accent-primary);
    background: var(--surface-card);
  }

  .subtask-inline-submit {
    height: 1.6rem;
    width: 1.6rem;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-xs);
    border: none;
    background: var(--surface-pill);
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 0.72rem;
  }

  .subtask-inline-submit:hover {
    background: var(--accent-primary);
    color: #fff;
  }

  /* Arretrati */
  .drawer-arrears-section {
    background: rgba(255, 159, 10, 0.06);
    border: 0.5px solid rgba(255, 159, 10, 0.28);
    border-radius: var(--radius-sm);
    padding: 0.75rem;
    margin-top: 0.5rem;
  }

  .arrears-head-title {
    display: flex;
    align-items: center;
    gap: 0.45rem;
  }

  .arrears-badge {
    background: #ff9f0a;
    color: #000;
    font-size: 0.65rem;
    font-weight: 800;
    padding: 0.1rem 0.4rem;
    border-radius: var(--radius-pill);
    letter-spacing: var(--tracking-wide);
  }

  .section-sub-hint {
    font-size: 0.7rem;
    color: var(--text-secondary);
    margin-top: -0.35rem;
    margin-bottom: 0.45rem;
  }

  .arrear-item-row {
    border-left: 2.5px solid #ff9f0a;
  }

  .arrear-content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }

  .arrear-date-tag {
    font-size: 0.64rem;
    color: #ff9f0a;
    font-weight: 600;
  }

  /* Note Markdown */
  .md-mode-tabs {
    display: inline-flex;
    gap: 0.2rem;
    padding: 0.12rem;
    border-radius: var(--radius-xs);
    background: var(--surface-pill);
    border: 0.5px solid var(--hairline-border);
  }

  .md-tab {
    border: none;
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.66rem;
    font-weight: 600;
    padding: 0.16rem 0.5rem;
    border-radius: 0.28rem;
    cursor: pointer;
  }

  .md-tab.active {
    background: var(--segment-active-bg);
    color: var(--text-primary);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.12);
  }

  .markdown-textarea {
    width: 100%;
    padding: 0.65rem 0.75rem;
    border-radius: var(--radius-sm);
    border: 0.5px solid var(--hairline-border-strong);
    background: var(--surface-card);
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: 0.78rem;
    line-height: 1.5;
    resize: vertical;
  }

  .markdown-preview-box {
    min-height: 5rem;
    padding: 0.65rem 0.75rem;
    border-radius: var(--radius-sm);
    background: var(--surface-card);
    border: 0.5px solid var(--hairline-border);
    font-size: 0.78rem;
    line-height: 1.5;
  }

  .note-footer-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .note-status {
    font-size: 0.66rem;
    color: var(--text-tertiary);
  }

  .save-note-btn {
    padding: 0.28rem 0.75rem;
    border-radius: var(--radius-xs);
    border: 0.5px solid var(--hairline-border);
    background: var(--surface-pill);
    color: var(--text-primary);
    font-size: 0.7rem;
    font-weight: 600;
    cursor: pointer;
  }
</style>
