<script lang="ts">
  import { onMount } from "svelte";
  import CommandPalette from "./lib/components/ai/CommandPalette.svelte";
  import AgendaView from "./lib/components/calendar/AgendaView.svelte";
  import HorizonRibbon from "./lib/components/calendar/HorizonRibbon.svelte";
  import InfiniteCalendar from "./lib/components/calendar/InfiniteCalendar.svelte";
  import SingleMonthView from "./lib/components/calendar/SingleMonthView.svelte";
  import WeekView from "./lib/components/calendar/WeekView.svelte";
  import DayInspectorDrawer from "./lib/components/drawer/DayInspectorDrawer.svelte";
  import Sidebar from "./lib/components/layout/Sidebar.svelte";
  import UnifiedToolbar from "./lib/components/layout/UnifiedToolbar.svelte";
  import ConflictPopover from "./lib/components/modals/ConflictPopover.svelte";
  import CourseModal from "./lib/components/modals/CourseModal.svelte";
  import ExamModal from "./lib/components/modals/ExamModal.svelte";
  import GenericEventModal from "./lib/components/modals/GenericEventModal.svelte";
  import OnboardingWizard from "./lib/components/modals/OnboardingWizard.svelte";
  import SessionModal from "./lib/components/modals/SessionModal.svelte";
  import SettingsModal from "./lib/components/modals/SettingsModal.svelte";
  import { sessionStore } from "./lib/stores/session.svelte";
  import { themeStore } from "./lib/stores/theme.svelte";
  import type {
    AcademicSession,
    Course,
    Exam,
    GenericEvent
  } from "./lib/types/domain";
  import {
    addDaysIso,
    addMonths,
    getTodayIso,
    indexSnapshotByDate,
    parseIsoDate,
    toIsoDate
  } from "./lib/utils/calendar";

  let selectedDate = $state<string | null>(getTodayIso());
  let drawerOpen = $state<boolean>(false);
  let selectedCourseId = $state<string | null>(null);

  // Stato modali operativi (Fase 5)
  let sessionModalState = $state<{ open: boolean; editing: AcademicSession | null }>({
    open: false,
    editing: null
  });
  let courseModalState = $state<{ open: boolean; editing: Course | null }>({
    open: false,
    editing: null
  });
  let examModalState = $state<{
    open: boolean;
    editing: Exam | null;
    defaultDate: string | null;
  }>({
    open: false,
    editing: null,
    defaultDate: null
  });
  let eventModalState = $state<{
    open: boolean;
    editing: GenericEvent | null;
    defaultDate: string | null;
  }>({
    open: false,
    editing: null,
    defaultDate: null
  });
  let conflictsModalOpen = $state<boolean>(false);
  let settingsModalState = $state<{
    open: boolean;
    tab: "appearance" | "colors" | "calendar" | "workload" | "ai" | "profile" | "backup";
  }>({
    open: false,
    tab: "appearance"
  });
  let commandPaletteState = $state<{ open: boolean; startWithVoice: boolean }>({
    open: false,
    startWithVoice: false
  });

  // Stato navigazione temporale per le viste (Mese / Settimana / Infinite Scroll)
  const initialParsed = parseIsoDate(getTodayIso());
  let currentMonthYear = $state<{ year: number; month: number }>({
    year: initialParsed.year,
    month: initialParsed.month
  });
  let currentWeekAnchorIso = $state<string>(getTodayIso());
  let scrollRequestKey = $state<number>(0);
  let scrollTargetIso = $state<string | null>(null);

  let isVoiceListening = $state<boolean>(false);
  let toastMessage = $state<string | null>(null);
  let toastTimer: ReturnType<typeof setTimeout> | null = null;

  let indexedCalendar = $derived(
    indexSnapshotByDate(sessionStore.activeSnapshot, sessionStore.holidays)
  );

  function showToast(msg: string) {
    toastMessage = msg;
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      toastMessage = null;
    }, 2600);
  }

  onMount(() => {
    (async () => {
      const payload = await sessionStore.bootstrap();
      if (payload?.activeSnapshot?.session.startDate) {
        const anchorRef = payload.activeSnapshot.session.startDate;
        const parsed = parseIsoDate(anchorRef);
        currentMonthYear = { year: parsed.year, month: parsed.month };
      }
      currentWeekAnchorIso = sessionStore.todayIso;
      selectedDate = sessionStore.todayIso;
    })();

    const handleKeyDown = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      const isEditable =
        target &&
        (target.tagName === "INPUT" ||
          target.tagName === "TEXTAREA" ||
          target.tagName === "SELECT" ||
          target.isContentEditable);

      if (e.key === "Escape") {
        if (commandPaletteState.open) {
          commandPaletteState.open = false;
          isVoiceListening = false;
          return;
        }
        if (settingsModalState.open) {
          settingsModalState.open = false;
          return;
        }
        if (conflictsModalOpen) {
          conflictsModalOpen = false;
          return;
        }
        if (examModalState.open) {
          examModalState.open = false;
          return;
        }
        if (courseModalState.open) {
          courseModalState.open = false;
          return;
        }
        if (sessionModalState.open) {
          sessionModalState.open = false;
          return;
        }
        if (eventModalState.open) {
          eventModalState.open = false;
          return;
        }
        if (drawerOpen) {
          drawerOpen = false;
          return;
        }
      }

      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        triggerCommandPalette(false);
        return;
      }

      if (!isEditable && e.code === "Space" && !e.ctrlKey && !e.altKey && !e.metaKey) {
        e.preventDefault();
        triggerCommandPalette(false);
        return;
      }

      if (!isEditable && !e.ctrlKey && !e.metaKey && !e.altKey) {
        if (e.key.toLowerCase() === "t") {
          e.preventDefault();
          handleNavigateToday();
          return;
        }
      }

      if (e.ctrlKey || e.metaKey) {
        if (e.key === "+" || e.key === "=") {
          e.preventDefault();
          themeStore.zoomIn();
        } else if (e.key === "-") {
          e.preventDefault();
          themeStore.zoomOut();
        } else if (e.key === "0") {
          e.preventDefault();
          themeStore.resetZoom();
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  });

  async function handleSelectSession(sessionId: string) {
    const payload = await sessionStore.setActiveSession(sessionId);
    selectedCourseId = null;
    if (payload.activeSnapshot?.session.startDate) {
      handleJumpToDate(payload.activeSnapshot.session.startDate, false);
    }
  }

  function handleJumpToDate(isoDate: string, openInspector = true) {
    selectedDate = isoDate;
    if (openInspector) {
      drawerOpen = true;
    }
    const { year, month } = parseIsoDate(isoDate);
    currentMonthYear = { year, month };
    currentWeekAnchorIso = isoDate;
    scrollTargetIso = isoDate;
    scrollRequestKey += 1;
  }

  function handleNavigateToday() {
    handleJumpToDate(sessionStore.todayIso, false);
  }

  function handleToggleSessionLock() {
    const isLocked = sessionStore.toggleSessionLock();
    if (isLocked) {
      showToast("Blocco sessione attivo: calendario limitato al periodo di sessione");
      if (sessionStore.activeSession?.startDate) {
        handleJumpToDate(sessionStore.activeSession.startDate, false);
      }
    } else {
      showToast("Scorrimento infinito sbloccato: puoi navigare liberamente in qualsiasi data");
      handleJumpToDate(selectedDate || sessionStore.todayIso, false);
    }
  }

  function handleNavigateStep(direction: -1 | 1) {
    if (sessionStore.isSessionLocked && sessionStore.activeSession) {
      const sStart = sessionStore.activeSession.startDate;
      const sEnd = sessionStore.activeSession.endDate;
      const pStart = parseIsoDate(sStart);
      const pEnd = parseIsoDate(sEnd);

      const view = themeStore.settings.defaultView;
      if (view === "week") {
        const nextWeek = addDaysIso(currentWeekAnchorIso, direction * 7);
        if (direction < 0 && nextWeek < sStart) return;
        if (direction > 0 && nextWeek > sEnd) return;
        currentWeekAnchorIso = nextWeek;
        selectedDate = currentWeekAnchorIso;
      } else {
        const next = addMonths(currentMonthYear.year, currentMonthYear.month, direction);
        if (
          direction < 0 &&
          (next.year < pStart.year || (next.year === pStart.year && next.month < pStart.month))
        ) {
          return;
        }
        if (
          direction > 0 &&
          (next.year > pEnd.year || (next.year === pEnd.year && next.month > pEnd.month))
        ) {
          return;
        }
        currentMonthYear = next;
        const targetIso = toIsoDate(next.year, next.month, 1);
        scrollTargetIso = targetIso;
        scrollRequestKey += 1;
      }
      return;
    }

    const view = themeStore.settings.defaultView;
    if (view === "week") {
      currentWeekAnchorIso = addDaysIso(currentWeekAnchorIso, direction * 7);
      selectedDate = currentWeekAnchorIso;
    } else {
      const next = addMonths(currentMonthYear.year, currentMonthYear.month, direction);
      currentMonthYear = next;
      const targetIso = toIsoDate(next.year, next.month, 1);
      scrollTargetIso = targetIso;
      scrollRequestKey += 1;
    }
  }

  function handleSelectDay(isoDate: string) {
    selectedDate = isoDate;
    drawerOpen = true;
    const { year, month } = parseIsoDate(isoDate);
    currentMonthYear = { year, month };
    currentWeekAnchorIso = isoDate;
  }

  function handleSelectExam(exam: Exam, _e: MouseEvent) {
    examModalState = {
      open: true,
      editing: exam,
      defaultDate: exam.date
    };
  }

  function triggerCommandPalette(startWithVoice = false) {
    commandPaletteState = {
      open: true,
      startWithVoice
    };
  }
</script>

<div class="app-window-frame">
  <!-- Barra Superiore Unificata (Apple HIG + Windows 11 Snap Layouts) -->
  <UnifiedToolbar
    sessions={sessionStore.sessions}
    activeSession={sessionStore.activeSession}
    conflictCount={sessionStore.activeSnapshot?.conflicts.length ?? 0}
    {isVoiceListening}
    isSessionLocked={sessionStore.isSessionLocked}
    onToggleSessionLock={handleToggleSessionLock}
    onSelectSession={handleSelectSession}
    onNewSession={() => (sessionModalState = { open: true, editing: null })}
    onEditSession={() =>
      (sessionModalState = { open: true, editing: sessionStore.activeSession })}
    onOpenConflicts={() => (conflictsModalOpen = true)}
    onNavigateToday={handleNavigateToday}
    onNavigateStep={handleNavigateStep}
    onOpenCommandPalette={triggerCommandPalette}
    onOpenSettings={() => (settingsModalState = { open: true, tab: "appearance" })}
  />

  <!-- Corpo Principale: Sidebar Telemetria + Calendario Infinito + Day Inspector Drawer -->
  <div class="workspace-body">
    <Sidebar
      snapshot={sessionStore.activeSnapshot}
      {selectedCourseId}
      onSelectCourseFilter={(id) => (selectedCourseId = id)}
      onJumpToDate={(d) => handleJumpToDate(d, true)}
      onOpenNewCourse={() => (courseModalState = { open: true, editing: null })}
      onEditCourse={(c) => (courseModalState = { open: true, editing: c })}
      onOpenNewExam={() => {
        if (sessionStore.courses.length === 0) {
          showToast("Crea prima una materia per poter pianificare un appello");
          courseModalState = { open: true, editing: null };
        } else {
          examModalState = { open: true, editing: null, defaultDate: selectedDate };
        }
      }}
      onOpenNewEvent={() =>
        (eventModalState = { open: true, editing: null, defaultDate: selectedDate })}
    />

    <main class="calendar-stage">
      <!-- Barra Orizzonte Temporale di Sessione -->
      <HorizonRibbon
        telemetry={sessionStore.activeSnapshot?.telemetry ?? null}
        {selectedCourseId}
        onSelectDate={(d) => handleJumpToDate(d, true)}
      />

      <!-- Viewport Ortogonale delle 4 Viste Calendario -->
      <div class="calendar-view-container">
        {#if themeStore.settings.defaultView === "continuous"}
          <InfiniteCalendar
            byDate={indexedCalendar.byDate}
            courseById={indexedCalendar.courseById}
            {selectedDate}
            {selectedCourseId}
            todayIso={sessionStore.todayIso}
            anchorIso={sessionStore.activeSnapshot?.session.startDate ?? sessionStore.todayIso}
            sessionStartDate={sessionStore.activeSession?.startDate ?? null}
            sessionEndDate={sessionStore.activeSession?.endDate ?? null}
            lockToSession={sessionStore.isSessionLocked}
            {scrollRequestKey}
            {scrollTargetIso}
            onSelectDay={handleSelectDay}
            onSelectExam={handleSelectExam}
            onVisibleMonthChange={(y, m) => {
              currentMonthYear = { year: y, month: m };
            }}
          />
        {:else if themeStore.settings.defaultView === "month"}
          <SingleMonthView
            year={currentMonthYear.year}
            month={currentMonthYear.month}
            byDate={indexedCalendar.byDate}
            courseById={indexedCalendar.courseById}
            {selectedDate}
            {selectedCourseId}
            todayIso={sessionStore.todayIso}
            lockToSession={sessionStore.isSessionLocked}
            sessionStartDate={sessionStore.activeSession?.startDate ?? null}
            sessionEndDate={sessionStore.activeSession?.endDate ?? null}
            onSelectDay={handleSelectDay}
            onSelectExam={handleSelectExam}
            onStepMonth={handleNavigateStep}
          />
        {:else if themeStore.settings.defaultView === "week"}
          <WeekView
            anchorIso={currentWeekAnchorIso}
            byDate={indexedCalendar.byDate}
            courseById={indexedCalendar.courseById}
            {selectedDate}
            {selectedCourseId}
            todayIso={sessionStore.todayIso}
            lockToSession={sessionStore.isSessionLocked}
            sessionStartDate={sessionStore.activeSession?.startDate ?? null}
            sessionEndDate={sessionStore.activeSession?.endDate ?? null}
            onSelectDay={handleSelectDay}
            onSelectExam={handleSelectExam}
            onStepWeek={handleNavigateStep}
          />
        {:else}
          <AgendaView
            exams={sessionStore.exams}
            conflicts={sessionStore.activeSnapshot?.conflicts ?? []}
            courseById={indexedCalendar.courseById}
            {selectedCourseId}
            todayIso={sessionStore.todayIso}
            onSelectDay={handleSelectDay}
            onSelectExam={handleSelectExam}
            onOpenNewExam={() => {
              if (sessionStore.courses.length === 0) {
                courseModalState = { open: true, editing: null };
              } else {
                examModalState = { open: true, editing: null, defaultDate: selectedDate };
              }
            }}
          />
        {/if}
      </div>
    </main>

    <!-- Pannello Ispettore Giornaliero (Note Markdown, To-Do, Unità Didattiche) -->
    {#if drawerOpen && selectedDate}
      <DayInspectorDrawer
        isoDate={selectedDate}
        dayData={indexedCalendar.byDate.get(selectedDate) ?? null}
        courseById={indexedCalendar.courseById}
        allStudyUnits={sessionStore.activeSnapshot?.studyUnits ?? []}
        onClose={() => (drawerOpen = false)}
        onOpenNewExamForDate={(iso) => {
          if (sessionStore.courses.length === 0) {
            courseModalState = { open: true, editing: null };
          } else {
            examModalState = { open: true, editing: null, defaultDate: iso };
          }
        }}
        onEditExam={(ex) =>
          (examModalState = { open: true, editing: ex, defaultDate: ex.date })}
        onOpenNewEventForDate={(iso) =>
          (eventModalState = { open: true, editing: null, defaultDate: iso })}
        onEditEvent={(ev) =>
          (eventModalState = { open: true, editing: ev, defaultDate: ev.date })}
      />
    {/if}
  </div>

  <!-- Modali Operativi (Fase 5) -->
  {#if sessionModalState.open}
    <SessionModal
      initialSession={sessionModalState.editing}
      onClose={() => (sessionModalState.open = false)}
    />
  {/if}

  {#if courseModalState.open}
    <CourseModal
      initialCourse={courseModalState.editing}
      existingUnits={sessionStore.activeSnapshot?.studyUnits ?? []}
      onClose={() => (courseModalState.open = false)}
    />
  {/if}

  {#if examModalState.open}
    <ExamModal
      initialExam={examModalState.editing}
      defaultDate={examModalState.defaultDate}
      defaultCourseId={selectedCourseId}
      courses={sessionStore.courses}
      onClose={() => (examModalState.open = false)}
    />
  {/if}

  {#if eventModalState.open}
    <GenericEventModal
      initialEvent={eventModalState.editing}
      defaultDate={eventModalState.defaultDate}
      courses={sessionStore.courses}
      onClose={() => (eventModalState.open = false)}
    />
  {/if}

  {#if conflictsModalOpen}
    <ConflictPopover
      conflicts={sessionStore.activeSnapshot?.conflicts ?? []}
      onSelectDate={(iso) => handleJumpToDate(iso, true)}
      onClose={() => (conflictsModalOpen = false)}
    />
  {/if}

  <!-- Assistente AI Multimodale & Impostazioni / Onboarding (Fase 6) -->
  {#if commandPaletteState.open}
    <CommandPalette
      startWithVoice={commandPaletteState.startWithVoice}
      onNavigateDate={(iso) => handleJumpToDate(iso, true)}
      onOpenSettings={() => (settingsModalState = { open: true, tab: "ai" })}
      onVoiceStateChange={(listening) => (isVoiceListening = listening)}
      onClose={() => {
        commandPaletteState.open = false;
        isVoiceListening = false;
      }}
    />
  {/if}

  {#if settingsModalState.open}
    <SettingsModal
      initialTab={settingsModalState.tab}
      onClose={() => (settingsModalState.open = false)}
      onToast={showToast}
    />
  {/if}

  {#if !sessionStore.isLoading && !sessionStore.profile.onboardingCompleted}
    <OnboardingWizard
      onCompleted={() => {
        showToast("Benvenuto in MySession! Il tuo workspace è pronto.");
      }}
    />
  {/if}

    <!-- Barra di Testo AI Fluttuante in Basso sopra la Schermata (Apple HIG Floating Island) -->
  {#if !commandPaletteState.open}
    <aside class="bottom-floating-ai-dock" aria-label="Assistente AI fluttuante">
      <div
        class="floating-ai-pill pressable"
        class:listening={isVoiceListening}
        role="search"
      >
        <button
          type="button"
          class="ai-pill-main-action"
          onclick={() => (commandPaletteState = { open: true, startWithVoice: false })}
          title="Chiedi all'Assistente AI (Ctrl+K oppure Spazio)"
        >
          <span class="ai-sparkle-pill" aria-hidden="true">
            <svg viewBox="0 0 16 16" fill="none">
              <path
                d="M8 1.75L9.45 6.05L13.75 7.5L9.45 8.95L8 13.25L6.55 8.95L2.25 7.5L6.55 6.05L8 1.75Z"
                stroke="currentColor"
                stroke-width="1.35"
                stroke-linejoin="round"
              />
            </svg>
          </span>
          <span class="ai-pill-text">
            {isVoiceListening
              ? "In ascolto vocale… parla ora"
              : "Chiedi all'AI, aggiungi appelli o cerca comandi…"}
          </span>
        </button>

        <div class="ai-pill-controls">
          <button
            type="button"
            class="ai-mic-btn pressable"
            class:active={isVoiceListening}
            title="Dettatura vocale con AI"
            aria-label="Avvia dettatura vocale"
            onclick={() => (commandPaletteState = { open: true, startWithVoice: true })}
          >
            {#if isVoiceListening}
              <span class="voice-wave-mini">
                <span></span><span></span><span></span>
              </span>
            {:else}
              <svg viewBox="0 0 16 16" fill="none">
                <rect x="5.75" y="2.25" width="4.5" height="7.5" rx="2.25" stroke="currentColor" stroke-width="1.35" />
                <path d="M3.5 7.75C3.5 10.2353 5.51472 12.25 8 12.25C10.4853 12.25 12.5 10.2353 12.5 7.75M8 12.25V14.25" stroke="currentColor" stroke-width="1.35" stroke-linecap="round" />
              </svg>
            {/if}
          </button>
          <div class="ai-kbd-pill" aria-hidden="true">
            <kbd>Ctrl+K</kbd>
            <kbd>Spazio</kbd>
          </div>
        </div>
      </div>
    </aside>
  {/if}

  {#if toastMessage}
    <div class="floating-toast" role="status">
      {toastMessage}
    </div>
  {/if}
</div>

<style>
  .app-window-frame {
    position: relative;
    display: flex;
    flex-direction: column;
    width: 100vw;
    height: 100vh;
    background: var(--app-shell-bg);
    color: var(--text-primary);
    overflow: hidden;
  }

  .workspace-body {
    flex: 1;
    display: flex;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  .calendar-stage {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

  .calendar-view-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }

    /* Barra di Testo AI Fluttuante in Basso sopra la Schermata */
  .bottom-floating-ai-dock {
    position: fixed;
    bottom: 1.35rem;
    left: 50%;
    transform: translateX(-50%);
    z-index: 120;
    width: min(580px, calc(100vw - 3rem));
    display: flex;
    justify-content: center;
    pointer-events: auto;
    animation: floatingAiEnter 240ms cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes floatingAiEnter {
    from {
      opacity: 0;
      transform: translate(-50%, 14px) scale(0.96);
    }
    to {
      opacity: 1;
      transform: translate(-50%, 0) scale(1);
    }
  }

  .floating-ai-pill {
    width: 100%;
    height: 2.75rem;
    display: flex;
    align-items: center;
    gap: 0.65rem;
    padding: 0 0.5rem 0 0.85rem;
    border-radius: var(--radius-pill);
    background: var(--popover-bg);
    backdrop-filter: blur(28px) saturate(190%);
    -webkit-backdrop-filter: blur(28px) saturate(190%);
    border: 0.5px solid var(--hairline-border-strong);
    box-shadow:
      0 4px 16px rgba(0, 0, 0, 0.12),
      0 16px 36px rgba(0, 0, 0, 0.22),
      inset 0 0.5px 0 rgba(255, 255, 255, 0.18);
    color: var(--text-primary);
    user-select: none;
    transition:
      border-color 160ms ease,
      box-shadow 160ms ease,
      background-color 160ms ease,
      transform 120ms var(--spring-press);
  }

  .ai-pill-main-action {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 0.65rem;
    height: 100%;
    border: none;
    background: transparent;
    cursor: pointer;
    text-align: left;
    padding: 0;
    min-width: 0;
    user-select: none;
    transition:
      border-color 160ms ease,
      box-shadow 160ms ease,
      background-color 160ms ease,
      transform 120ms var(--spring-press);
  }

  .floating-ai-pill:hover {
    border-color: var(--accent-primary);
    box-shadow:
      0 6px 20px rgba(0, 0, 0, 0.16),
      0 20px 42px rgba(0, 0, 0, 0.26),
      0 0 0 2px var(--accent-soft);
  }

  .floating-ai-pill:active {
    transform: scale(0.98);
  }

  .floating-ai-pill.listening {
    border-color: #38bdf8;
    box-shadow:
      0 0 0 3px rgba(56, 189, 248, 0.28),
      0 8px 24px rgba(56, 189, 248, 0.25);
  }

  .ai-sparkle-pill {
    width: 26px;
    height: 26px;
    border-radius: var(--radius-pill);
    background: var(--accent-soft);
    color: var(--accent-primary);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .ai-sparkle-pill svg {
    width: 15px;
    height: 15px;
  }

  .ai-pill-text {
    flex: 1;
    font-size: 0.8rem;
    font-weight: 500;
    color: var(--text-secondary);
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    letter-spacing: -0.01em;
  }

  .floating-ai-pill:hover .ai-pill-text {
    color: var(--text-primary);
  }

  .ai-pill-controls {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    flex-shrink: 0;
  }

  .ai-mic-btn {
    width: 28px;
    height: 28px;
    border-radius: var(--radius-pill);
    border: 0.5px solid var(--hairline-border);
    background: var(--surface-pill);
    color: var(--text-secondary);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition:
      transform 120ms var(--spring-press),
      background-color 140ms ease,
      color 140ms ease;
  }

  .ai-mic-btn svg {
    width: 14px;
    height: 14px;
  }

  .ai-mic-btn:hover {
    background: var(--surface-pill-hover);
    color: var(--text-primary);
  }

  .ai-mic-btn:active {
    transform: scale(0.92);
  }

  .ai-mic-btn.active {
    background: rgba(239, 68, 68, 0.16);
    border-color: rgba(239, 68, 68, 0.45);
    color: #ef4444;
  }

  .voice-wave-mini {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    height: 14px;
  }

  .voice-wave-mini span {
    width: 2px;
    height: 100%;
    border-radius: var(--radius-pill);
    background: #ef4444;
    animation: soundWave 0.7s infinite ease-in-out alternate;
  }

  .voice-wave-mini span:nth-child(2) {
    animation-delay: 0.15s;
  }
  .voice-wave-mini span:nth-child(3) {
    animation-delay: 0.3s;
  }

  @keyframes soundWave {
    0% { transform: scaleY(0.3); }
    100% { transform: scaleY(1); }
  }

  .ai-kbd-pill {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
  }

  .ai-kbd-pill kbd {
    padding: 0.15rem 0.4rem;
    border-radius: 5px;
    background: var(--surface-pill);
    border: 0.5px solid var(--hairline-border);
    font-family: var(--font-sans);
    font-size: 0.65rem;
    font-weight: 600;
    color: var(--text-tertiary);
  }

  .floating-toast {
    position: fixed;
    bottom: 4.85rem;
    left: 50%;
    transform: translateX(-50%);
    padding: 0.55rem 1.15rem;
    border-radius: var(--radius-pill);
    background: var(--popover-bg);
    backdrop-filter: blur(28px) saturate(190%);
    -webkit-backdrop-filter: blur(28px) saturate(190%);
    border: 0.5px solid var(--hairline-border-strong);
    color: var(--text-primary);
    font-size: 0.78rem;
    font-weight: 600;
    letter-spacing: -0.01em;
    box-shadow: 0 4px 14px rgba(0, 0, 0, 0.12), 0 16px 36px rgba(0, 0, 0, 0.22);
    z-index: 500;
    pointer-events: none;
    animation: toastSpringUp 240ms cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes toastSpringUp {
    from {
      opacity: 0;
      transform: translate(-50%, 10px) scale(0.96);
    }
    to {
      opacity: 1;
      transform: translate(-50%, 0) scale(1);
    }
  }
</style>
