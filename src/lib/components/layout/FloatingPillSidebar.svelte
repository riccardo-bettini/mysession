<script lang="ts">
  import {
    Bot,
    CalendarDays,
    CheckSquare,
    GraduationCap,
    HelpCircle,
    Home,
    Layers,
    Settings,
    TrendingUp
  } from "lucide-svelte";
  import { onMount, tick } from "svelte";
  import { SpringScalar } from "../../motion/spring.svelte";
  import { navStore } from "../../stores/navigation.svelte";
  import { sessionStore } from "../../stores/session.svelte";
  import { themeStore } from "../../stores/theme.svelte";
  import type { SectionId } from "../../types/domain";
  import { SECTION_LABELS } from "../../utils/categories";

  const ALL_SECTIONS: SectionId[] = ["home", "calendar", "today", "courses", "quiz", "progress", "ai"];

  let orderedVisibleSections = $derived.by<SectionId[]>(() => {
    const hidden = new Set(themeStore.settings.hiddenSections ?? []);
    const base = themeStore.settings.sidebarSections?.length
      ? themeStore.settings.sidebarSections
      : ALL_SECTIONS;
    const filtered = base.filter((id) => ALL_SECTIONS.includes(id) && !hidden.has(id));
    return filtered.length > 0 ? filtered : ["home"];
  });

  let navContainerEl = $state<HTMLDivElement | null>(null);
  let btnEls = $state<Record<string, HTMLButtonElement | null>>({});
  let hoveredId = $state<string | null>(null);
  let briefTooltipId = $state<string | null>(null);
  let tooltipY = $state<number>(0);

  const indicatorY = new SpringScalar(0, { dampingRatio: 0.78, response: 0.28 });
  let hasIndicatorInitialized = false;
  let briefTimer: number | null = null;

  function computeButtonCenterTop(id: string): number | null {
    const container = navContainerEl;
    const btn = btnEls[id];
    if (!container || !btn) return null;
    const cRect = container.getBoundingClientRect();
    const bRect = btn.getBoundingClientRect();
    return bRect.top - cRect.top + (bRect.height - 42) / 2;
  }

  async function syncIndicator(animate = true): Promise<void> {
    await tick();
    const targetTop = computeButtonCenterTop(navStore.activeSection);
    if (targetTop === null) return;
    if (!hasIndicatorInitialized || !animate) {
      indicatorY.snap(targetTop);
      hasIndicatorInitialized = true;
    } else {
      indicatorY.setTarget(targetTop);
    }
  }

  $effect(() => {
    // Re-sync whenever activeSection or section ordering changes
    void navStore.activeSection;
    void orderedVisibleSections;
    void syncIndicator(true);
  });

  onMount(() => {
    void syncIndicator(false);
    const onResize = () => void syncIndicator(false);
    window.addEventListener("resize", onResize);
    return () => {
      window.removeEventListener("resize", onResize);
      if (briefTimer !== null) window.clearTimeout(briefTimer);
    };
  });

  function handleSelectSection(id: SectionId) {
    navStore.navigate(id);
    briefTooltipId = id;
    const top = computeButtonCenterTop(id);
    if (top !== null) tooltipY = top + 21;
    if (briefTimer !== null) window.clearTimeout(briefTimer);
    briefTimer = window.setTimeout(() => {
      if (briefTooltipId === id) briefTooltipId = null;
    }, 950);
  }

  function handleHover(id: string | null) {
    hoveredId = id;
    if (id) {
      const btn = btnEls[id];
      const rail = navContainerEl?.closest(".floating-rail");
      if (btn && rail) {
        const rRect = rail.getBoundingClientRect();
        const bRect = btn.getBoundingClientRect();
        tooltipY = bRect.top - rRect.top + bRect.height / 2;
      }
    }
  }

  let activeTooltipLabel = $derived.by<string | null>(() => {
    const id = hoveredId ?? briefTooltipId;
    if (!id) return null;
    if (id === "sessions") return sessionStore.activeSession?.name || "Sessioni";
    if (id === "settings") return "Impostazioni";
    return SECTION_LABELS[id] ?? null;
  });

  $effect(() => {
    if (hoveredId || briefTooltipId) {
      const id = hoveredId ?? briefTooltipId;
      const btn = id ? btnEls[id] : null;
      const rail = navContainerEl?.closest(".floating-rail");
      if (btn && rail) {
        const rRect = rail.getBoundingClientRect();
        const bRect = btn.getBoundingClientRect();
        tooltipY = bRect.top - rRect.top + bRect.height / 2;
      }
    }
  });
</script>

<aside class="floating-rail" aria-label="Navigazione principale">
  <!-- Logo in alto -->
  <button
    type="button"
    class="rail-logo"
    title="MySession v2.00"
    onclick={() => handleSelectSection("home")}
  >
    <svg viewBox="0 0 28 28" fill="none" class="logo-svg" aria-hidden="true">
      <path d="M7.5 20L12.5 8" stroke="currentColor" stroke-width="2.7" stroke-linecap="round" />
      <path d="M13.5 20L18.5 8" stroke="currentColor" stroke-width="2.7" stroke-linecap="round" />
      <circle cx="21" cy="18.5" r="2.1" fill="#DDD5FA" />
    </svg>
  </button>

  <!-- 7 Sezioni centrali con disco scorrevole -->
  <div class="rail-nav" bind:this={navContainerEl}>
    <div
      class="active-disc"
      style:transform="translate3d(0, {indicatorY.current}px, 0)"
      aria-hidden="true"
    ></div>

    {#each orderedVisibleSections as secId (secId)}
      {@const isActive = navStore.activeSection === secId}
      <button
        type="button"
        bind:this={btnEls[secId]}
        class="rail-btn"
        class:active={isActive}
        aria-label={SECTION_LABELS[secId]}
        aria-current={isActive ? "page" : undefined}
        onmouseenter={() => handleHover(secId)}
        onmouseleave={() => handleHover(null)}
        onclick={() => handleSelectSection(secId)}
      >
        {#if secId === "home"}
          <Home size={19} strokeWidth={2} />
        {:else if secId === "calendar"}
          <CalendarDays size={19} strokeWidth={2} />
        {:else if secId === "today"}
          <CheckSquare size={19} strokeWidth={2} />
        {:else if secId === "courses"}
          <GraduationCap size={19} strokeWidth={2} />
        {:else if secId === "quiz"}
          <HelpCircle size={19} strokeWidth={2} />
        {:else if secId === "progress"}
          <TrendingUp size={19} strokeWidth={2} />
        {:else if secId === "ai"}
          <Bot size={19} strokeWidth={2} />
        {/if}
      </button>
    {/each}
  </div>

  <!-- Footer: Selettore Sessioni sopra + Impostazioni sotto -->
  <div class="rail-footer">
    <button
      type="button"
      bind:this={btnEls["sessions"]}
      class="rail-btn"
      class:active={navStore.isSessionPopoverOpen}
      aria-label="Seleziona sessione"
      onmouseenter={() => handleHover("sessions")}
      onmouseleave={() => handleHover(null)}
      onclick={() => (navStore.isSessionPopoverOpen = !navStore.isSessionPopoverOpen)}
    >
      <Layers size={19} strokeWidth={2} />
      {#if sessionStore.pendingPlans.length > 0}
        <span class="pending-dot" aria-label="Aggiornamenti disponibili"></span>
      {/if}
    </button>

    <button
      type="button"
      bind:this={btnEls["settings"]}
      class="rail-btn"
      class:active={navStore.isSettingsOpen}
      aria-label="Impostazioni"
      onmouseenter={() => handleHover("settings")}
      onmouseleave={() => handleHover(null)}
      onclick={() => navStore.openSettings("appearance")}
    >
      <Settings size={19} strokeWidth={2} />
    </button>
  </div>

  <!-- Tooltip pillola a destra (stile video) -->
  {#if activeTooltipLabel}
    <div
      class="rail-tooltip"
      style:transform="translate3d(0, {tooltipY}px, 0) translateY(-50%)"
      role="tooltip"
    >
      <span class="tooltip-arrow"></span>
      <span>{activeTooltipLabel}</span>
    </div>
  {/if}
</aside>

<style>
  .floating-rail {
    position: relative;
    width: 62px;
    height: calc(100vh - 32px);
    flex-shrink: 0;
    background: var(--sidebar-bg);
    border-radius: 999px;
    box-shadow: var(--shadow-sidebar);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: space-between;
    padding: 16px 0;
    z-index: 60;
  }

  .rail-logo {
    width: 42px;
    height: 42px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    color: #FFFFFF;
    transition: transform 0.2s ease, opacity 0.2s ease;
  }

  .rail-logo:hover {
    transform: scale(1.06);
  }

  .logo-svg {
    width: 24px;
    height: 24px;
  }

  .rail-nav {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    width: 100%;
  }

  .active-disc {
    position: absolute;
    top: 0;
    left: 10px;
    width: 42px;
    height: 42px;
    border-radius: 50%;
    background: var(--sidebar-disc);
    box-shadow: 0 4px 16px rgba(221, 213, 250, 0.32);
    pointer-events: none;
    z-index: 0;
    will-change: transform;
  }

  .rail-btn {
    position: relative;
    z-index: 1;
    width: 42px;
    height: 42px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    color: var(--sidebar-icon);
    transition: color 0.18s ease, transform 0.16s ease, background-color 0.18s ease;
  }

  .rail-btn:hover {
    color: var(--sidebar-icon-hover);
    transform: scale(1.06);
  }

  .rail-btn.active {
    color: var(--sidebar-disc-ink);
  }

  .rail-footer {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }

  .rail-footer .rail-btn.active {
    background: rgba(255, 255, 255, 0.14);
    color: #FFFFFF;
  }

  .pending-dot {
    position: absolute;
    top: 8px;
    right: 8px;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #A78BFA;
    border: 2px solid var(--sidebar-bg);
  }

  .rail-tooltip {
    position: absolute;
    top: 0;
    left: 74px;
    display: inline-flex;
    align-items: center;
    padding: 6px 13px;
    border-radius: 999px;
    background: #16161C;
    color: #FFFFFF;
    font-size: 12px;
    font-weight: 600;
    white-space: nowrap;
    box-shadow: 0 8px 22px rgba(0, 0, 0, 0.25);
    pointer-events: none;
    z-index: 90;
  }

  .tooltip-arrow {
    position: absolute;
    left: -5px;
    top: 50%;
    transform: translateY(-50%);
    width: 0;
    height: 0;
    border-top: 5px solid transparent;
    border-bottom: 5px solid transparent;
    border-right: 6px solid #16161C;
  }
</style>
