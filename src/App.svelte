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
    tab: "appearance" | "calendar" | "ai" | "profile" | "backup";
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

  function handleNavigateStep(direction: -1 | 1) {
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
            {scrollRequestKey}
            {scrollTargetIso}
            onSelectDay={handleSelectDay}
            onSelectExam={handleSelectExam}
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

  .floating-toast {
    position: fixed;
    bottom: 1.25rem;
    left: 50%;
    transform: translateX(-50%);
    padding: 0.55rem 1rem;
    border-radius: var(--radius-pill);
    background: var(--popover-bg);
    backdrop-filter: blur(25px) saturate(190%);
    -webkit-backdrop-filter: blur(25px) saturate(190%);
    border: 0.5px solid var(--hairline-border-strong);
    color: var(--text-primary);
    font-size: 0.78rem;
    font-weight: 600;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.35);
    z-index: 500;
    pointer-events: none;
  }
</style>
