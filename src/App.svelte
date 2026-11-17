<script lang="ts">
  import { onMount } from "svelte";
  import UpdateNotificationBanner from "./lib/components/common/UpdateNotificationBanner.svelte";
  import FloatingPillSidebar from "./lib/components/layout/FloatingPillSidebar.svelte";
  import SessionPopover from "./lib/components/layout/SessionPopover.svelte";
  import WelcomeEmptyState from "./lib/components/layout/WelcomeEmptyState.svelte";
  import WindowControlsOverlay from "./lib/components/layout/WindowControlsOverlay.svelte";
  import NewSessionModal from "./lib/components/modals/NewSessionModal.svelte";
  import PlanUpdateModal from "./lib/components/modals/PlanUpdateModal.svelte";
  import QuizPlayerModal from "./lib/components/modals/QuizPlayerModal.svelte";
  import SettingsModal from "./lib/components/modals/SettingsModal.svelte";
  import TimetableModal from "./lib/components/modals/TimetableModal.svelte";
  import AiAgentView from "./lib/components/views/AiAgentView.svelte";
  import CalendarView from "./lib/components/views/CalendarView.svelte";
  import CoursesView from "./lib/components/views/CoursesView.svelte";
  import HomeView from "./lib/components/views/HomeView.svelte";
  import ProgressView from "./lib/components/views/ProgressView.svelte";
  import QuizView from "./lib/components/views/QuizView.svelte";
  import TodayView from "./lib/components/views/TodayView.svelte";
  import { navStore } from "./lib/stores/navigation.svelte";
  import { sessionStore } from "./lib/stores/session.svelte";
  import { themeStore } from "./lib/stores/theme.svelte";

  onMount(() => {
    void sessionStore.bootstrap();
    return () => {
      sessionStore.destroy();
      themeStore.destroy();
    };
  });
</script>

<WindowControlsOverlay />
<UpdateNotificationBanner />

<div class="app-shell">
  <FloatingPillSidebar />
  <SessionPopover />

  <main class="main-canvas">
    {#if sessionStore.isBootstrapping}
      <div class="morph-grid">
        <div class="skeleton-block sk-hero"></div>
        <div class="sk-row">
          <div class="skeleton-block sk-card"></div>
          <div class="skeleton-block sk-card"></div>
          <div class="skeleton-block sk-card"></div>
        </div>
      </div>
    {:else if !sessionStore.activeSnapshot}
      <WelcomeEmptyState />
    {:else if navStore.isMorphing}
      <div class="morph-grid">
        <div class="skeleton-block sk-hero"></div>
        <div class="sk-row">
          <div class="skeleton-block sk-card"></div>
          <div class="skeleton-block sk-card"></div>
          <div class="skeleton-block sk-card"></div>
        </div>
        <div class="sk-row-2">
          <div class="skeleton-block sk-card-lg"></div>
          <div class="skeleton-block sk-card-lg"></div>
        </div>
      </div>
    {:else if navStore.activeSection === "home"}
      <HomeView />
    {:else if navStore.activeSection === "calendar"}
      <CalendarView />
    {:else if navStore.activeSection === "today"}
      <TodayView />
    {:else if navStore.activeSection === "courses"}
      <CoursesView />
    {:else if navStore.activeSection === "quiz"}
      <QuizView />
    {:else if navStore.activeSection === "progress"}
      <ProgressView />
    {:else if navStore.activeSection === "ai"}
      <AiAgentView />
    {/if}
  </main>
</div>

<SettingsModal />
<PlanUpdateModal />
<TimetableModal />
<NewSessionModal />
<QuizPlayerModal />

{#if navStore.toast}
  <div class="toast-pill" class:error={navStore.toast.tone === "error"} class:success={navStore.toast.tone === "success"}>
    {navStore.toast.message}
  </div>
{/if}

<style>
  .app-shell {
    display: flex;
    gap: 18px;
    width: 100vw;
    height: 100vh;
    padding: 16px 20px 16px 16px;
    overflow: hidden;
    background: var(--bg-canvas);
  }

  .main-canvas {
    flex: 1;
    min-width: 0;
    height: 100%;
    overflow-y: auto;
    padding-right: 4px;
    padding-top: 22px;
  }

  .morph-grid {
    display: flex;
    flex-direction: column;
    gap: 16px;
    height: 100%;
  }

  .sk-hero {
    height: 136px;
  }

  .sk-row {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 16px;
  }

  .sk-card {
    height: 170px;
  }

  .sk-row-2 {
    display: grid;
    grid-template-columns: 1.1fr 0.9fr;
    gap: 16px;
  }

  .sk-card-lg {
    height: 200px;
  }

  .toast-pill {
    position: fixed;
    bottom: 22px;
    right: 24px;
    z-index: 200;
    padding: 10px 16px;
    border-radius: 999px;
    background: var(--bg-surface-dark);
    color: #FFFFFF;
    font-size: 13px;
    font-weight: 600;
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.25);
  }

  .toast-pill.success {
    background: #059669;
  }

  .toast-pill.error {
    background: #DC2626;
  }
</style>
