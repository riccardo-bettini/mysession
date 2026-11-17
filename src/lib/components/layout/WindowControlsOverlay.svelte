<script lang="ts">
  import { Copy, Minus, Square, X } from "lucide-svelte";
  import { onMount } from "svelte";
  import { themeStore } from "../../stores/theme.svelte";

  let maxBtnEl = $state<HTMLButtonElement | null>(null);

  onMount(() => {
    const sync = () => void themeStore.registerMaximizeBounds(maxBtnEl);
    sync();
    window.addEventListener("resize", sync);
    return () => window.removeEventListener("resize", sync);
  });
</script>

<!-- Zona superiore invisibile di trascinamento finestra -->
<div class="drag-strip" data-tauri-drag-region></div>

<!-- Controlli finestra minimali nell'angolo in alto a destra (senza topbar visibile) -->
<div class="win-controls" role="group" aria-label="Controlli finestra">
  <button
    type="button"
    class="win-btn"
    title="Riduci a icona"
    aria-label="Riduci a icona"
    onclick={() => themeStore.minimizeWindow()}
  >
    <Minus size={13} strokeWidth={2.2} />
  </button>

  <button
    type="button"
    bind:this={maxBtnEl}
    class="win-btn"
    title={themeStore.isMaximized ? "Ripristina" : "Ingrandisci"}
    aria-label={themeStore.isMaximized ? "Ripristina" : "Ingrandisci"}
    onclick={() => themeStore.toggleMaximizeWindow()}
  >
    {#if themeStore.isMaximized}
      <Copy size={12} strokeWidth={2.1} />
    {:else}
      <Square size={12} strokeWidth={2.1} />
    {/if}
  </button>

  <button
    type="button"
    class="win-btn close"
    title="Chiudi"
    aria-label="Chiudi"
    onclick={() => themeStore.closeWindow()}
  >
    <X size={13} strokeWidth={2.2} />
  </button>
</div>

<style>
  .drag-strip {
    position: fixed;
    top: 0;
    left: 90px;
    right: 120px;
    height: 18px;
    z-index: 45;
  }

  .win-controls {
    position: fixed;
    top: 12px;
    right: 16px;
    z-index: 70;
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 4px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.72);
    border: 1px solid var(--border-subtle);
    backdrop-filter: blur(12px);
  }

  :global(html[data-theme="dark"]) .win-controls {
    background: rgba(26, 26, 34, 0.78);
  }

  .win-btn {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    color: var(--text-secondary);
    transition: background-color 0.15s ease, color 0.15s ease;
  }

  .win-btn:hover {
    background: var(--border-subtle);
    color: var(--text-primary);
  }

  .win-btn.close:hover {
    background: #EF4444;
    color: #FFFFFF;
  }
</style>
