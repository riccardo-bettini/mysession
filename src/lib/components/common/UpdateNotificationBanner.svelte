<script lang="ts">
  import { updaterStore } from "../../stores/updater.svelte";

  let isVisible = $derived(
    !updaterStore.isBannerDismissed &&
      (updaterStore.status === "available" ||
        updaterStore.status === "downloading" ||
        updaterStore.status === "ready")
  );
</script>

{#if isVisible}
  <aside class="update-banner" role="alert" aria-live="polite">
    <div class="banner-glow-effect"></div>

    <div class="banner-content">
      <!-- Icona di stato -->
      <div class="banner-icon-container" class:ready={updaterStore.status === "ready"}>
        {#if updaterStore.status === "ready"}
          <svg viewBox="0 0 16 16" fill="none" class="banner-svg success-svg">
            <path d="M3 8.5L6.5 12L13 4.5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        {:else if updaterStore.status === "downloading"}
          <svg viewBox="0 0 16 16" fill="none" class="banner-svg spin-svg">
            <circle cx="8" cy="8" r="6" stroke="currentColor" stroke-width="1.8" stroke-dasharray="28" stroke-dashoffset="10"/>
          </svg>
        {:else}
          <span class="rocket-emoji">🚀</span>
        {/if}
      </div>

      <!-- Testo informativo -->
      <div class="banner-text-block">
        {#if updaterStore.status === "available"}
          <div class="banner-title-row">
            <strong class="banner-title">Nuova Versione Disponibile</strong>
            <span class="version-tag">v{updaterStore.availableVersion}</span>
          </div>
          <p class="banner-desc">È disponibile una nuova versione con miglioramenti e correzioni.</p>
        {:else if updaterStore.status === "downloading"}
          <div class="banner-title-row">
            <strong class="banner-title">Download Aggiornamento...</strong>
            <span class="progress-badge">{updaterStore.downloadProgress}%</span>
          </div>
          <div class="progress-track" role="progressbar" aria-valuenow={updaterStore.downloadProgress} aria-valuemin={0} aria-valuemax={100}>
            <div class="progress-bar-fill" style:width="{updaterStore.downloadProgress}%"></div>
          </div>
        {:else if updaterStore.status === "ready"}
          <div class="banner-title-row">
            <strong class="banner-title">Aggiornamento Installato!</strong>
          </div>
          <p class="banner-desc">Riavvia MySession per applicare la nuova versione.</p>
        {/if}
      </div>

      <!-- Azioni -->
      <div class="banner-actions">
        {#if updaterStore.status === "available"}
          <button
            type="button"
            class="btn-update-primary pressable"
            onclick={() => updaterStore.downloadAndInstall()}
          >
            Aggiorna Ora
          </button>
          <button
            type="button"
            class="btn-banner-close pressable"
            title="Nascondi notifica"
            aria-label="Nascondi notifica aggiornamento"
            onclick={() => updaterStore.dismissBanner()}
          >
            ✕
          </button>
        {:else if updaterStore.status === "ready"}
          <button
            type="button"
            class="btn-update-primary restart-btn pressable"
            onclick={() => updaterStore.relaunch()}
          >
            Riavvia Ora
          </button>
        {/if}
      </div>
    </div>
  </aside>
{/if}

<style>
  .update-banner {
    position: fixed;
    top: 4.15rem;
    right: 1.25rem;
    z-index: 124;
    min-width: 320px;
    max-width: 440px;
    padding: 0.75rem 1rem;
    border-radius: var(--radius-lg, 14px);
    background: var(--popover-bg, rgba(30, 30, 30, 0.9));
    backdrop-filter: blur(28px) saturate(190%);
    -webkit-backdrop-filter: blur(28px) saturate(190%);
    border: 0.5px solid var(--hairline-border-strong, rgba(255, 255, 255, 0.16));
    box-shadow: 0 12px 34px -4px rgba(0, 0, 0, 0.38), 0 0 0 0.5px rgba(255, 255, 255, 0.08);
    animation: bannerSlideIn 240ms cubic-bezier(0.16, 1, 0.3, 1);
    overflow: hidden;
  }

  @keyframes bannerSlideIn {
    from {
      opacity: 0;
      transform: translateY(-12px) scale(0.97);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .banner-glow-effect {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 1.5px;
    background: linear-gradient(90deg, transparent, var(--accent-primary, #0A84FF), transparent);
  }

  .banner-content {
    display: flex;
    align-items: center;
    gap: 0.85rem;
  }

  .banner-icon-container {
    width: 2.25rem;
    height: 2.25rem;
    border-radius: 999px;
    background: rgba(10, 132, 255, 0.12);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    color: var(--accent-primary, #0A84FF);
  }

  .banner-icon-container.ready {
    background: rgba(48, 209, 88, 0.15);
    color: #30D158;
  }

  .rocket-emoji {
    font-size: 1.15rem;
    line-height: 1;
  }

  .banner-svg {
    width: 1.15rem;
    height: 1.15rem;
  }

  .spin-svg {
    animation: spinRotate 900ms linear infinite;
  }

  @keyframes spinRotate {
    to {
      transform: rotate(360deg);
    }
  }

  .banner-text-block {
    flex: 1;
    min-width: 0;
  }

  .banner-title-row {
    display: flex;
    align-items: center;
    gap: 0.45rem;
  }

  .banner-title {
    font-size: 0.82rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .version-tag {
    padding: 0.1rem 0.4rem;
    border-radius: 999px;
    background: var(--accent-primary, #0A84FF);
    color: #fff;
    font-size: 0.65rem;
    font-weight: 700;
    letter-spacing: 0.02em;
  }

  .progress-badge {
    font-size: 0.72rem;
    font-weight: 700;
    color: var(--accent-primary, #0A84FF);
    font-variant-numeric: tabular-nums;
  }

  .banner-desc {
    margin: 0.15rem 0 0;
    font-size: 0.73rem;
    color: var(--text-secondary);
    line-height: 1.35;
  }

  .progress-track {
    width: 100%;
    height: 5px;
    background: rgba(128, 128, 128, 0.2);
    border-radius: 999px;
    margin-top: 0.45rem;
    overflow: hidden;
  }

  .progress-bar-fill {
    height: 100%;
    background: var(--accent-primary, #0A84FF);
    border-radius: 999px;
    transition: width 140ms ease;
  }

  .banner-actions {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    flex-shrink: 0;
  }

  .btn-update-primary {
    padding: 0.38rem 0.75rem;
    border-radius: var(--radius-sm, 7px);
    background: var(--accent-primary, #0A84FF);
    color: #fff;
    border: none;
    font-size: 0.74rem;
    font-weight: 700;
    cursor: pointer;
    box-shadow: 0 2px 6px rgba(10, 132, 255, 0.35);
    white-space: nowrap;
    transition: opacity 120ms ease, transform 120ms ease;
  }

  .btn-update-primary.restart-btn {
    background: #30D158;
    box-shadow: 0 2px 6px rgba(48, 209, 88, 0.35);
  }

  .btn-banner-close {
    width: 1.6rem;
    height: 1.6rem;
    border-radius: 999px;
    background: transparent;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.75rem;
    transition: background 120ms ease, color 120ms ease;
  }

  .btn-banner-close:hover {
    background: var(--surface-hover, rgba(255, 255, 255, 0.1));
    color: var(--text-primary);
  }
</style>
