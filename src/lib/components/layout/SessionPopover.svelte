<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { Check, FolderOpen, Plus, RefreshCw, Sparkles, Trash2 } from "lucide-svelte";
  import { onMount } from "svelte";
  import { navStore } from "../../stores/navigation.svelte";
  import { sessionStore } from "../../stores/session.svelte";
  import { themeStore } from "../../stores/theme.svelte";

  let containerEl = $state<HTMLDivElement | null>(null);
  let isScanning = $state(false);
  let confirmDeleteId = $state<string | null>(null);

  async function pickFolder() {
    const selected = await open({ directory: true, multiple: false, title: "Scegli cartella file delle sessioni" });
    if (typeof selected === "string" && selected.trim()) {
      const pending = await sessionStore.setPlanFolder(selected);
      navStore.showToast(
        pending.length > 0
          ? `Trovati ${pending.length} file sessione pronti per l'importazione`
          : "Cartella salvata. Nessun aggiornamento pendente.",
        "info"
      );
    }
  }

  async function rescan() {
    isScanning = true;
    try {
      const pending = await sessionStore.scanPendingPlans();
      navStore.showToast(
        pending.length > 0 ? `${pending.length} aggiornamenti trovati` : "Tutte le sessioni sono aggiornate",
        "info"
      );
    } finally {
      isScanning = false;
    }
  }

  onMount(() => {
    const onPointerDown = (e: MouseEvent) => {
      if (!navStore.isSessionPopoverOpen) return;
      const target = e.target as HTMLElement | null;
      if (target && !containerEl?.contains(target) && !target.closest('[aria-label="Seleziona sessione"]')) {
        navStore.isSessionPopoverOpen = false;
      }
    };
    window.addEventListener("pointerdown", onPointerDown);
    return () => window.removeEventListener("pointerdown", onPointerDown);
  });
</script>

{#if navStore.isSessionPopoverOpen}
  <div class="session-popover bento-card bento-enter" bind:this={containerEl}>
    <div class="pop-header">
      <div>
        <h4>Sessioni Accademiche</h4>
        <p class="pop-sub">Seleziona o importa un piano di studio</p>
      </div>
      <button
        type="button"
        class="pill-btn pill-btn-primary btn-xs"
        onclick={() => {
          navStore.isSessionPopoverOpen = false;
          navStore.isNewSessionModalOpen = true;
        }}
      >
        <Plus size={14} />
        Nuova
      </button>
    </div>

    <div class="session-list">
      {#if sessionStore.sessions.length === 0}
        <div class="empty-note">Nessuna sessione presente. Importa un file JSON o creane una nuova.</div>
      {:else}
        {#each sessionStore.sessions as sess (sess.id)}
          {@const active = sessionStore.activeSession?.id === sess.id}
          <div class="sess-row" class:active>
            <button
              type="button"
              class="sess-select"
              onclick={() => sessionStore.selectSession(sess.id)}
            >
              <div class="sess-info">
                <span class="sess-name">{sess.name}</span>
                <span class="sess-meta">{sess.academicYear || sess.sessionType} · {sess.startDate} → {sess.endDate}</span>
              </div>
              {#if active}
                <span class="check-badge"><Check size={14} /></span>
              {/if}
            </button>

            {#if confirmDeleteId === sess.id}
              <button
                type="button"
                class="del-confirm"
                onclick={() => {
                  confirmDeleteId = null;
                  void sessionStore.deleteSession(sess.id);
                }}
              >
                Elimina
              </button>
            {:else}
              <button
                type="button"
                class="del-btn"
                title="Elimina sessione"
                onclick={() => (confirmDeleteId = sess.id)}
              >
                <Trash2 size={14} />
              </button>
            {/if}
          </div>
        {/each}
      {/if}
    </div>

    {#if sessionStore.pendingPlans.length > 0}
      <div class="pending-box">
        <div class="pending-title">
          <Sparkles size={14} />
          <span>Piani JSON rilevati ({sessionStore.pendingPlans.length})</span>
        </div>
        {#each sessionStore.pendingPlans as p (p.planId)}
          <div class="pending-item">
            <div>
              <strong>{p.planName}</strong>
              <span>Rev. {p.revision} · {p.isNewSession ? "Nuova sessione" : "Aggiornamento"}</span>
            </div>
            <button
              type="button"
              class="pill-btn pill-btn-primary btn-xs"
              onclick={() => {
                navStore.isSessionPopoverOpen = false;
                navStore.pendingPlanModal = p;
              }}
            >
              Apri
            </button>
          </div>
        {/each}
      </div>
    {/if}

    <div class="pop-footer">
      <button type="button" class="pill-btn pill-btn-soft btn-xs" onclick={pickFolder}>
        <FolderOpen size={14} />
        {themeStore.settings.planFolder ? "Cambia cartella" : "Cartella sessioni"}
      </button>
      {#if themeStore.settings.planFolder}
        <button type="button" class="pill-btn pill-btn-soft btn-xs" onclick={rescan} disabled={isScanning}>
          <RefreshCw size={14} class={isScanning ? "spin" : ""} />
          Scansiona
        </button>
      {/if}
    </div>
  </div>
{/if}

<style>
  .session-popover {
    position: fixed;
    left: 92px;
    bottom: 24px;
    width: 356px;
    max-height: 75vh;
    padding: 16px;
    z-index: 110;
    display: flex;
    flex-direction: column;
    gap: 12px;
    box-shadow: 0 22px 54px rgba(12, 12, 20, 0.22);
  }

  .pop-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
  }

  .pop-header h4 {
    margin: 0;
    font-size: 15px;
    font-weight: 700;
  }

  .pop-sub {
    margin: 2px 0 0;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .btn-xs {
    padding: 6px 11px;
    font-size: 12px;
  }

  .session-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-height: 240px;
    overflow-y: auto;
  }

  .sess-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 8px;
    border-radius: 14px;
    background: var(--bg-surface-muted);
    border: 1px solid transparent;
    transition: border-color 0.16s ease, background-color 0.16s ease;
  }

  .sess-row.active {
    border-color: var(--accent);
    background: var(--accent-soft);
  }

  .sess-select {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: space-between;
    text-align: left;
    gap: 8px;
    padding: 4px;
  }

  .sess-info {
    display: flex;
    flex-direction: column;
  }

  .sess-name {
    font-size: 13px;
    font-weight: 700;
  }

  .sess-meta {
    font-size: 11.5px;
    color: var(--text-secondary);
  }

  .check-badge {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: var(--accent);
    color: #fff;
    display: grid;
    place-items: center;
  }

  .del-btn {
    padding: 6px;
    border-radius: 10px;
    color: var(--text-muted);
  }

  .del-btn:hover {
    color: var(--danger);
    background: rgba(239, 68, 68, 0.12);
  }

  .del-confirm {
    padding: 4px 8px;
    border-radius: 8px;
    background: var(--danger);
    color: #fff;
    font-size: 11px;
    font-weight: 700;
  }

  .pending-box {
    padding: 10px;
    border-radius: 16px;
    background: var(--accent-soft);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .pending-title {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    font-weight: 700;
    color: var(--accent);
  }

  .pending-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    background: var(--bg-surface);
    padding: 8px 10px;
    border-radius: 12px;
  }

  .pending-item div {
    display: flex;
    flex-direction: column;
    font-size: 12px;
  }

  .pending-item span {
    font-size: 11px;
    color: var(--text-secondary);
  }

  .pop-footer {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .empty-note {
    font-size: 12.5px;
    color: var(--text-secondary);
    padding: 10px;
    text-align: center;
  }

  :global(.spin) {
    animation: spin360 1s linear infinite;
  }

  @keyframes spin360 {
    to {
      transform: rotate(360deg);
    }
  }
</style>
