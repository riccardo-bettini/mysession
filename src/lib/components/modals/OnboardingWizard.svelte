<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { readTextFile } from "@tauri-apps/plugin-fs";
  import { APP_AUTHOR, APP_VERSION } from "../../constants/version";
  import { sessionStore } from "../../stores/session.svelte";
  import { themeStore } from "../../stores/theme.svelte";
  import type {
    AiProvider,
    BackdropMaterial,
    SessionType,
    ThemeChoice,
    WindowControlsStyle
  } from "../../types/domain";

  interface Props {
    onCompleted: () => void;
  }

  let { onCompleted }: Props = $props();

  let step = $state<1 | 2 | 3>(1);

  // Step 1: Profilo Studente
  let studentName = $state<string>("");
  let university = $state<string>("");
  let faculty = $state<string>("");
  let degreeCourse = $state<string>("");

  // Step 2: Prima Sessione Accademica + Materiali DWM
  const currentYear = new Date().getFullYear();
  let sessionName = $state<string>(`Sessione Invernale ${currentYear}/${currentYear + 1}`);
  let sessionType = $state<SessionType>("invernale");
  let academicYear = $state<string>(`${currentYear}/${currentYear + 1}`);
  let startDate = $state<string>(`${currentYear + 1}-01-07`);
  let endDate = $state<string>(`${currentYear + 1}-02-28`);

  // Step 3: Assistente AI Opzionale
  let aiProvider = $state<AiProvider>("local");
  let aiApiKey = $state<string>("");
  let aiModel = $state<string>("gemini-3.8-flash");
  let aiEndpointUrl = $state<string>("http://localhost:11434/api/generate");

  let isSubmitting = $state<boolean>(false);
  let errorMsg = $state<string | null>(null);
  let fileInputEl = $state<HTMLInputElement | null>(null);

  async function handleFinishOnboarding() {
    isSubmitting = true;
    errorMsg = null;

    try {
      await themeStore.updateSettings({
        aiProvider,
        aiApiKey: aiApiKey.trim(),
        aiModel: aiModel.trim(),
        aiEndpointUrl: aiEndpointUrl.trim()
      });

      await sessionStore.upsertProfile({
        studentName: studentName.trim() || "Studente",
        university: university.trim(),
        faculty: faculty.trim(),
        degreeCourse: degreeCourse.trim(),
        enrollmentYear: currentYear,
        onboardingCompleted: true
      });

      if (sessionStore.sessions.length === 0) {
        await sessionStore.upsertSession({
          id: null,
          name: sessionName.trim() || "Sessione Accademica",
          subtitle: degreeCourse.trim() || faculty.trim() || university.trim(),
          sessionType,
          academicYear: academicYear.trim() || `${currentYear}/${currentYear + 1}`,
          startDate,
          endDate,
          notes: ""
        });
      }

      onCompleted();
    } catch (e) {
      errorMsg = e instanceof Error ? e.message : String(e);
    } finally {
      isSubmitting = false;
    }
  }

  async function handleImportExistingBackup() {
    errorMsg = null;
    try {
      const selected = await open({
        title: "Importa Backup MySession (.json)",
        multiple: false,
        filters: [{ name: "JSON Backup", extensions: ["json"] }]
      });
      if (typeof selected === "string" && selected) {
        const raw = await readTextFile(selected);
        await sessionStore.importBackupJson(raw);
        if (!sessionStore.profile.onboardingCompleted) {
          await sessionStore.upsertProfile({
            ...sessionStore.profile,
            onboardingCompleted: true
          });
        }
        onCompleted();
      }
    } catch {
      fileInputEl?.click();
    }
  }

  async function handleFallbackFileChange(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    try {
      const raw = await file.text();
      await sessionStore.importBackupJson(raw);
      if (!sessionStore.profile.onboardingCompleted) {
        await sessionStore.upsertProfile({
          ...sessionStore.profile,
          onboardingCompleted: true
        });
      }
      onCompleted();
    } catch (err) {
      errorMsg = err instanceof Error ? err.message : String(err);
    } finally {
      input.value = "";
    }
  }
</script>

<div class="onboarding-backdrop" role="dialog" aria-modal="true" aria-label="Benvenuto in MySession">
  <div class="onboarding-card squircle">
    <div class="wizard-top-bar">
      <div class="brand-badge">
        <span class="brand-dot"></span>
        <span>MySession v{APP_VERSION} · Suite Accademica Nativa</span>
      </div>
      <div class="step-dots">
        <span class="step-dot" class:active={step === 1}>1. Profilo</span>
        <span class="step-dot" class:active={step === 2}>2. Finestra & Sessione</span>
        <span class="step-dot" class:active={step === 3}>3. Assistente AI</span>
      </div>
    </div>

    {#if errorMsg}
      <div class="wizard-error">{errorMsg}</div>
    {/if}

    {#if step === 1}
      <div class="wizard-step-body">
        <div class="hero-block">
          <h1 class="hero-title">Benvenuto in MySession</h1>
          <p class="hero-subtitle">
            Ambiente desktop ad altissima efficienza progettato secondo le Apple Human Interface
            Guidelines con materiali nativi Windows 11 Mica/Acrylic e database SQLite isolato.
          </p>
        </div>

        <div class="form-grid-2">
          <label class="field">
            <span>Il tuo Nome (opzionale)</span>
            <input type="text" bind:value={studentName} placeholder="Es. Giulia Bianchi" />
          </label>
          <label class="field">
            <span>Università / Ateneo</span>
            <input
              type="text"
              bind:value={university}
              placeholder="Es. Politecnico / Università degli Studi"
            />
          </label>
          <label class="field">
            <span>Facoltà / Scuola</span>
            <input type="text" bind:value={faculty} placeholder="Es. Ingegneria / Economia / Medicina" />
          </label>
          <label class="field">
            <span>Corso di Laurea</span>
            <input type="text" bind:value={degreeCourse} placeholder="Es. Laurea Triennale / Magistrale" />
          </label>
        </div>

        <div class="import-callout">
          <div>
            <strong>Hai già un file di backup <code>.json</code> da un altro PC?</strong>
            <p>Importalo subito per ripristinare tutte le tue sessioni, i corsi e gli appelli.</p>
          </div>
          <button type="button" class="import-btn pressable" onclick={handleImportExistingBackup}>
            ⬆ Importa Backup .json
          </button>
          <input
            bind:this={fileInputEl}
            type="file"
            accept=".json,application/json"
            style="display: none"
            onchange={handleFallbackFileChange}
          />
        </div>
      </div>

      <footer class="wizard-footer">
        <span class="author-note">Autore: {APP_AUTHOR}</span>
        <button type="button" class="primary-btn pressable" onclick={() => (step = 2)}>
          Continua →
        </button>
      </footer>
    {:else if step === 2}
      <div class="wizard-step-body">
        <div class="hero-block">
          <h2 class="step-heading">Materiali Finestra & Prima Sessione</h2>
          <p class="hero-subtitle">
            Personalizza l'aspetto nativo Windows 11 e imposta l'intervallo temporale della tua
            prima sessione d'esame.
          </p>
        </div>

        <div class="segmented-group">
          <span class="group-label">Tema & Materiale DWM</span>
          <div class="seg-row">
            {#each [
              { id: "auto", label: "Auto (Sistema)" },
              { id: "dark", label: "Scuro" },
              { id: "light", label: "Chiaro" }
            ] as t (t.id)}
              <button
                type="button"
                class="seg-btn pressable"
                class:active={themeStore.settings.theme === t.id}
                onclick={() => themeStore.setTheme(t.id as ThemeChoice)}
              >
                {t.label}
              </button>
            {/each}
          </div>

          <div class="seg-row">
            {#each [
              { id: "mica", label: "Mica" },
              { id: "acrylic", label: "Acrylic" },
              { id: "tabbed", label: "Mica Alt" }
            ] as m (m.id)}
              <button
                type="button"
                class="seg-btn pressable"
                class:active={themeStore.settings.backdropMaterial === m.id}
                onclick={() => themeStore.setBackdropMaterial(m.id as BackdropMaterial)}
              >
                {m.label}
              </button>
            {/each}
          </div>

          <div class="seg-row">
            <button
              type="button"
              class="seg-btn pressable"
              class:active={themeStore.settings.windowControlsStyle === "macos-left"}
              onclick={() =>
                themeStore.setWindowControlsStyle("macos-left" as WindowControlsStyle)}
            >
              Controlli Semaforo Apple (Sinistra)
            </button>
            <button
              type="button"
              class="seg-btn pressable"
              class:active={themeStore.settings.windowControlsStyle === "win11-right"}
              onclick={() =>
                themeStore.setWindowControlsStyle("win11-right" as WindowControlsStyle)}
            >
              Controlli Windows 11 (Destra)
            </button>
          </div>
        </div>

        <div class="form-grid-2">
          <label class="field">
            <span>Nome Prima Sessione</span>
            <input type="text" bind:value={sessionName} />
          </label>
          <label class="field">
            <span>Tipologia</span>
            <select bind:value={sessionType}>
              <option value="invernale">Sessione Invernale</option>
              <option value="estiva">Sessione Estiva</option>
              <option value="autunnale">Sessione Autunnale</option>
              <option value="custom">Personalizzata</option>
            </select>
          </label>
          <label class="field">
            <span>Data Inizio</span>
            <input type="date" bind:value={startDate} />
          </label>
          <label class="field">
            <span>Data Fine</span>
            <input type="date" bind:value={endDate} />
          </label>
        </div>
      </div>

      <footer class="wizard-footer">
        <button type="button" class="secondary-btn pressable" onclick={() => (step = 1)}>
          ← Indietro
        </button>
        <button type="button" class="primary-btn pressable" onclick={() => (step = 3)}>
          Continua →
        </button>
      </footer>
    {:else}
      <div class="wizard-step-body">
        <div class="hero-block">
          <h2 class="step-heading">Assistente AI Multimodale (Tastiera + Voce)</h2>
          <p class="hero-subtitle">
            Attivabile in qualsiasi momento con <kbd>Ctrl+K</kbd> o <kbd>Spazio</kbd>. Scegli se
            utilizzare il motore deterministico offline integrato o collegare un provider LLM.
          </p>
        </div>

        <div class="seg-row">
          {#each [
            { id: "local", label: "Locale Deterministico (Offline)" },
            { id: "gemini", label: "Google Gemini" },
            { id: "openai", label: "OpenAI" },
            { id: "ollama", label: "Ollama / LM Studio" }
          ] as p (p.id)}
            <button
              type="button"
              class="seg-btn pressable"
              class:active={aiProvider === p.id}
              onclick={() => (aiProvider = p.id as AiProvider)}
            >
              {p.label}
            </button>
          {/each}
        </div>

        {#if aiProvider === "gemini" || aiProvider === "openai"}
          <label class="field">
            <span>API Key Personale (opzionale, modificabile in seguito)</span>
            <input
              type="password"
              bind:value={aiApiKey}
              placeholder="Incolla qui la tua API Key..."
            />
          </label>
        {:else if aiProvider === "ollama"}
          <label class="field">
            <span>Endpoint Ollama / LM Studio</span>
            <input type="text" bind:value={aiEndpointUrl} />
          </label>
        {:else}
          <div class="local-engine-card">
            <strong>⚡ Motore Rust Deterministico Pronto all'Uso</strong>
            <p>
              Non richiede alcuna API key né connessione di rete. Permette di inserire corsi,
              appelli e task, interrogare i CFU pianificati, individuare il corso più critico e
              controllare il tema o lo zoom con comandi naturali o dettatura vocale.
            </p>
          </div>
        {/if}
      </div>

      <footer class="wizard-footer">
        <button type="button" class="secondary-btn pressable" onclick={() => (step = 2)}>
          ← Indietro
        </button>
        <button
          type="button"
          class="primary-btn pressable"
          onclick={handleFinishOnboarding}
          disabled={isSubmitting}
        >
          {isSubmitting ? "Inizializzazione..." : "Inizia a usare MySession ✨"}
        </button>
      </footer>
    {/if}
  </div>
</div>

<style>
  .onboarding-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.58);
    backdrop-filter: blur(16px) saturate(180%);
    -webkit-backdrop-filter: blur(16px) saturate(180%);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1.5rem;
    z-index: 420;
  }

  .onboarding-card {
    width: 100%;
    max-width: 640px;
    background: var(--popover-bg);
    border: 0.5px solid var(--hairline-border-strong);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-modal);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .wizard-top-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.85rem 1.25rem;
    border-bottom: 0.5px solid var(--hairline-border);
    background: rgba(128, 128, 128, 0.05);
  }

  .brand-badge {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    font-size: 0.72rem;
    font-weight: 700;
    color: var(--text-secondary);
  }

  .brand-dot {
    width: 8px;
    height: 8px;
    border-radius: 99px;
    background: var(--accent-primary);
  }

  .step-dots {
    display: flex;
    gap: 0.4rem;
  }

  .step-dot {
    font-size: 0.66rem;
    font-weight: 600;
    padding: 0.16rem 0.5rem;
    border-radius: var(--radius-pill);
    background: rgba(128, 128, 128, 0.1);
    color: var(--text-muted);
  }

  .step-dot.active {
    background: color-mix(in srgb, var(--accent-primary) 18%, transparent);
    color: var(--accent-primary);
  }

  .wizard-error {
    margin: 0.75rem 1.25rem 0;
    padding: 0.5rem 0.75rem;
    border-radius: var(--radius-md);
    background: rgba(239, 68, 68, 0.15);
    border: 0.5px solid rgba(239, 68, 68, 0.4);
    color: #f87171;
    font-size: 0.75rem;
  }

  .wizard-step-body {
    padding: 1.35rem 1.4rem;
    display: flex;
    flex-direction: column;
    gap: 1.05rem;
  }

  .hero-block {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .hero-title {
    margin: 0;
    font-size: 1.35rem;
    font-weight: 700;
    letter-spacing: -0.025em;
  }

  .step-heading {
    margin: 0;
    font-size: 1.15rem;
    font-weight: 700;
    letter-spacing: -0.022em;
  }

  .hero-subtitle {
    margin: 0;
    font-size: 0.8rem;
    line-height: 1.48;
    color: var(--text-secondary);
  }

  .form-grid-2 {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.28rem;
    font-size: 0.73rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .field input,
  .field select {
    padding: 0.48rem 0.65rem;
    border-radius: var(--radius-md);
    border: 0.5px solid var(--hairline-border);
    background: rgba(128, 128, 128, 0.08);
    color: var(--text-primary);
    font-size: 0.8rem;
    outline: none;
    transition: border-color 140ms ease, box-shadow 140ms ease;
  }

  .field input:focus-visible,
  .field select:focus-visible {
    border-color: var(--accent-primary);
    box-shadow: 0 0 0 2.5px var(--accent-soft);
  }

  .import-callout {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.75rem 0.95rem;
    border-radius: var(--radius-lg);
    border: 0.5px dashed var(--hairline-border-strong);
    background: rgba(128, 128, 128, 0.05);
    font-size: 0.75rem;
  }

  .import-callout p {
    margin: 0.15rem 0 0;
    font-size: 0.7rem;
    color: var(--text-muted);
  }

  .import-btn {
    padding: 0.42rem 0.8rem;
    border-radius: var(--radius-md);
    border: 0.5px solid var(--accent-primary);
    background: color-mix(in srgb, var(--accent-primary) 14%, transparent);
    color: var(--accent-primary);
    font-size: 0.74rem;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
    transition: transform 120ms cubic-bezier(0.16, 1, 0.3, 1), filter 120ms ease;
  }

  .import-btn:active {
    transform: scale(0.97);
  }

  .segmented-group {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
  }

  .group-label {
    font-size: 0.72rem;
    font-weight: 700;
    color: var(--text-secondary);
  }

  .seg-row {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
    gap: 0.35rem;
    padding: 0.25rem;
    border-radius: var(--radius-md);
    background: rgba(128, 128, 128, 0.1);
    border: 0.5px solid var(--hairline-border);
  }

  .seg-btn {
    padding: 0.4rem 0.55rem;
    border-radius: var(--radius-xs);
    border: none;
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.72rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 140ms cubic-bezier(0.16, 1, 0.3, 1);
  }

  .seg-btn:active {
    transform: scale(0.98);
  }

  .seg-btn.active {
    background: var(--surface-card);
    color: var(--text-primary);
    box-shadow: 0 1.5px 5px rgba(0, 0, 0, 0.14);
  }

  .local-engine-card {
    padding: 0.85rem 1rem;
    border-radius: var(--radius-lg);
    border: 0.5px solid rgba(16, 185, 129, 0.35);
    background: rgba(16, 185, 129, 0.1);
    font-size: 0.76rem;
    line-height: 1.5;
  }

  .local-engine-card p {
    margin: 0.25rem 0 0;
    color: var(--text-secondary);
  }

  .wizard-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.85rem 1.4rem;
    border-top: 0.5px solid var(--hairline-border);
  }

  .author-note {
    font-size: 0.7rem;
    color: var(--text-muted);
  }

  .secondary-btn {
    padding: 0.45rem 0.9rem;
    border-radius: var(--radius-md);
    border: 0.5px solid var(--hairline-border);
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.78rem;
    font-weight: 600;
    cursor: pointer;
    transition: transform 120ms cubic-bezier(0.16, 1, 0.3, 1), background 120ms ease;
  }

  .secondary-btn:active {
    transform: scale(0.97);
  }

  .primary-btn {
    padding: 0.48rem 1.15rem;
    border-radius: var(--radius-md);
    border: none;
    background: var(--accent-primary);
    color: #fff;
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    transition: transform 120ms cubic-bezier(0.16, 1, 0.3, 1), filter 120ms ease;
  }

  .primary-btn:active {
    transform: scale(0.97);
  }
</style>
