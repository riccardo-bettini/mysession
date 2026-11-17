<script lang="ts">
  import { X } from "lucide-svelte";
  import { navStore } from "../../stores/navigation.svelte";
  import { sessionStore } from "../../stores/session.svelte";
  import type { SessionType } from "../../types/domain";
  import { addDaysIso, getTodayIso } from "../../utils/calendar";

  let name = $state("");
  let subtitle = $state("");
  let sessionType = $state<SessionType>("invernale");
  let academicYear = $state("2026/2027");
  let startDate = $state(getTodayIso());
  let endDate = $state(addDaysIso(getTodayIso(), 75));
  let maxHoursPerDay = $state(6);

  async function handleSubmit() {
    if (!name.trim() || !startDate || !endDate) return;
    await sessionStore.createOrUpdateSession({
      name: name.trim(),
      subtitle: subtitle.trim(),
      sessionType,
      academicYear: academicYear.trim(),
      startDate,
      endDate,
      maxHoursPerDay: Number(maxHoursPerDay) || 6
    });
    name = "";
    subtitle = "";
    navStore.isNewSessionModalOpen = false;
  }
</script>

{#if navStore.isNewSessionModalOpen}
  <div class="modal-backdrop" role="dialog" aria-modal="true" aria-label="Nuova Sessione">
    <div class="bento-card ns-modal bento-enter">
      <header class="ns-head">
        <h2>Nuova Sessione Accademica</h2>
        <button type="button" class="close-btn" onclick={() => (navStore.isNewSessionModalOpen = false)}>
          <X size={18} />
        </button>
      </header>

      <div class="ns-form">
        <label>
          <span>Nome sessione *</span>
          <input type="text" class="soft-input" placeholder="es. Sessione Estiva 2027" bind:value={name} />
        </label>
        <div class="row-2">
          <label>
            <span>Tipologia</span>
            <select class="soft-input" bind:value={sessionType}>
              <option value="invernale">Invernale</option>
              <option value="estiva">Estiva</option>
              <option value="autunnale">Autunnale</option>
              <option value="custom">Personalizzata</option>
            </select>
          </label>
          <label>
            <span>Anno Accademico</span>
            <input type="text" class="soft-input" bind:value={academicYear} />
          </label>
        </div>
        <div class="row-2">
          <label>
            <span>Data inizio</span>
            <input type="date" class="soft-input" bind:value={startDate} />
          </label>
          <label>
            <span>Data fine</span>
            <input type="date" class="soft-input" bind:value={endDate} />
          </label>
        </div>
        <label>
          <span>Ore massime al giorno</span>
          <input type="number" class="soft-input" min="1" max="16" step="0.5" bind:value={maxHoursPerDay} />
        </label>
      </div>

      <footer class="ns-foot">
        <button type="button" class="pill-btn pill-btn-soft" onclick={() => (navStore.isNewSessionModalOpen = false)}>
          Annulla
        </button>
        <button type="button" class="pill-btn pill-btn-primary" onclick={handleSubmit}>
          Crea Sessione
        </button>
      </footer>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 145;
    background: rgba(12, 12, 18, 0.55);
    backdrop-filter: blur(8px);
    display: grid;
    place-items: center;
    padding: 20px;
  }

  .ns-modal {
    width: min(460px, 100%);
    padding: 22px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .ns-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .ns-head h2 {
    margin: 0;
    font-size: 18px;
  }

  .close-btn {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    background: var(--bg-surface-muted);
    display: grid;
    place-items: center;
  }

  .ns-form {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .ns-form label {
    display: flex;
    flex-direction: column;
    gap: 5px;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .row-2 {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }

  .soft-input {
    padding: 9px 11px;
    border-radius: 12px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface-muted);
    color: var(--text-primary);
    font-size: 13px;
  }

  .ns-foot {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
