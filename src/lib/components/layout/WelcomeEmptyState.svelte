<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { FolderOpen, Plus, Sparkles } from "lucide-svelte";
  import { navStore } from "../../stores/navigation.svelte";
  import { sessionStore } from "../../stores/session.svelte";
  import { themeStore } from "../../stores/theme.svelte";

  async function selectFolder() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Seleziona la cartella dei file delle sessioni (.json)"
    });
    if (typeof selected === "string" && selected.trim()) {
      const pending = await sessionStore.setPlanFolder(selected);
      if (pending.length === 0) {
        navStore.showToast("Cartella impostata. Nessun file sessione v3 trovato.", "info");
      }
    }
  }
</script>

<div class="welcome-wrap bento-enter">
  <div class="welcome-hero">
    <span class="badge-v2">MySession v2.00</span>
    <h1>Benvenuto nel tuo nuovo spazio di studio</h1>
    <p>
      Scegli la cartella dove salvi i file delle sessioni generati dal Builder per importarli automaticamente,
      oppure crea una nuova sessione manualmente.
    </p>
    <div class="hero-actions">
      <button type="button" class="pill-btn pill-btn-primary" onclick={selectFolder}>
        <FolderOpen size={16} />
        {themeStore.settings.planFolder ? "Cambia cartella sessioni" : "Scegli cartella sessioni"}
      </button>
      <button type="button" class="pill-btn pill-btn-soft" onclick={() => (navStore.isNewSessionModalOpen = true)}>
        <Plus size={16} />
        Crea sessione manuale
      </button>
    </div>
    {#if themeStore.settings.planFolder}
      <div class="folder-pill">Cartella attiva: <code>{themeStore.settings.planFolder}</code></div>
    {/if}
  </div>

  {#if sessionStore.pendingPlans.length > 0}
    <div class="bento-card pending-grid-card">
      <div class="card-head">
        <Sparkles size={18} />
        <h3>Piani pronti nella cartella</h3>
      </div>
      <div class="pending-cards">
        {#each sessionStore.pendingPlans as plan (plan.planId)}
          <div class="plan-row">
            <div>
              <strong>{plan.planName}</strong>
              <span>Revisione {plan.revision} · Schema v3</span>
            </div>
            <button
              type="button"
              class="pill-btn pill-btn-primary"
              onclick={() => (navStore.pendingPlanModal = plan)}
            >
              Anteprima & Importa
            </button>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .welcome-wrap {
    max-width: 880px;
    margin: 48px auto;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .welcome-hero {
    padding: 36px;
    border-radius: var(--radius-card);
    background: var(--hero-gradient);
    display: flex;
    flex-direction: column;
    gap: 14px;
    box-shadow: var(--shadow-card);
  }

  .badge-v2 {
    align-self: flex-start;
    padding: 5px 12px;
    border-radius: 999px;
    background: rgba(20, 20, 24, 0.85);
    color: #fff;
    font-size: 12px;
    font-weight: 700;
  }

  .welcome-hero h1 {
    margin: 0;
    font-size: 28px;
    font-weight: 800;
    letter-spacing: -0.02em;
  }

  .welcome-hero p {
    margin: 0;
    max-width: 60ch;
    font-size: 14.5px;
    color: var(--text-primary);
    opacity: 0.84;
  }

  .hero-actions {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
    margin-top: 6px;
  }

  .folder-pill {
    font-size: 12px;
    color: var(--text-primary);
    opacity: 0.78;
  }

  .pending-grid-card {
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .card-head {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--accent);
  }

  .card-head h3 {
    margin: 0;
    font-size: 16px;
    color: var(--text-primary);
  }

  .pending-cards {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .plan-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 16px;
    border-radius: 16px;
    background: var(--bg-surface-muted);
  }

  .plan-row div {
    display: flex;
    flex-direction: column;
  }

  .plan-row span {
    font-size: 12px;
    color: var(--text-secondary);
  }
</style>
