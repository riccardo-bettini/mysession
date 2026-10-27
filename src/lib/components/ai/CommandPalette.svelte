<script lang="ts">
  import DOMPurify from "dompurify";
  import { marked } from "marked";
  import { onMount } from "svelte";
  import { sessionStore } from "../../stores/session.svelte";
  import { themeStore } from "../../stores/theme.svelte";
  import type { AiAssistantResponse } from "../../types/domain";

  interface Props {
    startWithVoice?: boolean;
    onNavigateDate: (isoDate: string) => void;
    onOpenSettings: () => void;
    onVoiceStateChange?: (listening: boolean) => void;
    onClose: () => void;
  }

  let {
    startWithVoice = false,
    onNavigateDate,
    onOpenSettings,
    onVoiceStateChange,
    onClose
  }: Props = $props();

  let promptText = $state<string>("");
  let isProcessing = $state<boolean>(false);
  let isListening = $state<boolean>(false);
  let voiceSupported = $state<boolean>(false);
  let voiceError = $state<string | null>(null);
  let errorMsg = $state<string | null>(null);
  let lastResponse = $state<AiAssistantResponse | null>(null);
  let history = $state<Array<{ prompt: string; response: AiAssistantResponse }>>([]);

  let inputEl = $state<HTMLInputElement | null>(null);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let recognitionInstance: any = null;

  const quickSuggestions = [
    {
      label: "Riepilogo & Carico",
      prompt: "Dammi un riepilogo della sessione e del carico di studio"
    },
    {
      label: "Corso più critico",
      prompt: "Qual è il corso più critico in questo momento?"
    },
    {
      label: "Controlla conflitti",
      prompt: "Ci sono conflitti o sovrapposizioni tra gli appelli?"
    },
    {
      label: "Totale CFU",
      prompt: "Quanti CFU ho pianificato nella sessione?"
    },
    {
      label: "Prossimo esame",
      prompt: "Quando è il prossimo appello d'esame?"
    },
    {
      label: "Vista Agenda",
      prompt: "Passa a vista agenda"
    },
    {
      label: "Vista Continua",
      prompt: "Passa a vista continua"
    },
    {
      label: "Task per domani",
      prompt: "Aggiungi task domani: Esercitazione e ripasso capitoli"
    }
  ];

  const providerLabel = $derived.by(() => {
    const p = themeStore.settings.aiProvider;
    if (p === "gemini") return "Google Gemini";
    if (p === "openai") return "OpenAI API";
    if (p === "ollama") return "Ollama / LM Studio";
    return "Motore Locale Deterministico";
  });

  const renderedReplyHtml = $derived.by(() => {
    if (!lastResponse?.replyMarkdown) return "";
    const raw = marked.parse(lastResponse.replyMarkdown, { async: false }) as string;
    return DOMPurify.sanitize(raw);
  });

  function setListeningState(active: boolean) {
    isListening = active;
    onVoiceStateChange?.(active);
  }

  function initSpeechRecognition() {
    if (typeof window === "undefined") return;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const SpeechRec = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
    if (!SpeechRec) {
      voiceSupported = false;
      return;
    }

    voiceSupported = true;
    const rec = new SpeechRec();
    rec.lang = "it-IT";
    rec.continuous = false;
    rec.interimResults = true;

    rec.onstart = () => {
      voiceError = null;
      setListeningState(true);
    };

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    rec.onresult = (event: any) => {
      let transcript = "";
      let isFinal = false;
      for (let i = event.resultIndex; i < event.results.length; i++) {
        transcript += event.results[i][0].transcript;
        if (event.results[i].isFinal) {
          isFinal = true;
        }
      }
      promptText = transcript.trim();
      if (isFinal && promptText.length > 1) {
        setListeningState(false);
        void handleExecutePrompt(promptText);
      }
    };

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    rec.onerror = (event: any) => {
      setListeningState(false);
      if (event?.error && event.error !== "no-speech" && event.error !== "aborted") {
        voiceError = `Dettatura vocale non disponibile (${event.error}). Puoi digitare il comando.`;
      }
    };

    rec.onend = () => {
      setListeningState(false);
    };

    recognitionInstance = rec;
  }

  function toggleVoiceDictation() {
    voiceError = null;
    if (!voiceSupported || !recognitionInstance) {
      voiceError =
        "Il runtime corrente non espone Web Speech API nativa. Digita il tuo comando nella barra.";
      inputEl?.focus();
      return;
    }

    if (isListening) {
      try {
        recognitionInstance.stop();
      } catch {
        // Ignora errori di stop
      }
      setListeningState(false);
    } else {
      try {
        recognitionInstance.start();
      } catch {
        setListeningState(false);
      }
    }
  }

  async function handleExecutePrompt(customPrompt?: string) {
    const query = (customPrompt ?? promptText).trim();
    if (!query || isProcessing) return;

    promptText = query;
    errorMsg = null;
    isProcessing = true;

    try {
      const response = await sessionStore.executeAiCommand(query);
      lastResponse = response;
      history = [{ prompt: query, response }, ...history.slice(0, 7)];

      if (response.navigateToDate) {
        onNavigateDate(response.navigateToDate);
      }
    } catch (e) {
      errorMsg = e instanceof Error ? e.message : String(e);
    } finally {
      isProcessing = false;
    }
  }

  onMount(() => {
    initSpeechRecognition();
    setTimeout(() => {
      inputEl?.focus();
      if (startWithVoice && voiceSupported) {
        toggleVoiceDictation();
      }
    }, 40);

    return () => {
      if (recognitionInstance && isListening) {
        try {
          recognitionInstance.stop();
        } catch {
          // Ignora
        }
      }
      onVoiceStateChange?.(false);
    };
  });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="command-backdrop" onclick={onClose}>
  <div
    class="command-sheet squircle"
    onclick={(e) => e.stopPropagation()}
    role="dialog"
    aria-modal="true"
    aria-label="Assistente AI e Command Bar"
    tabindex="-1"
  >
    <!-- Barra di Input Principale stile Spotlight / Raycast -->
    <form
      class="spotlight-bar"
      onsubmit={(e) => {
        e.preventDefault();
        handleExecutePrompt();
      }}
    >
      <div class="sparkle-badge" class:pulsing={isProcessing || isListening}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
          <path
            d="M12 2.5L14.4 9.6L21.5 12L14.4 14.4L12 21.5L9.6 14.4L2.5 12L9.6 9.6L12 2.5Z"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linejoin="round"
          />
        </svg>
      </div>

      <input
        bind:this={inputEl}
        type="text"
        class="spotlight-input"
        bind:value={promptText}
        placeholder={isListening
          ? "In ascolto... Parla ora in italiano (es. 'Aggiungi appello di Fisica il 2027-01-22')"
          : "Chiedi all'AI o esegui un'azione (es. 'Aggiungi corso Fisica 9 CFU', 'Corso più critico?')..."}
      />

      {#if isListening}
        <div class="voice-waveform" aria-label="Onda sonora dettatura attiva">
          <span></span>
          <span></span>
          <span></span>
          <span></span>
          <span></span>
        </div>
      {/if}

      <button
        type="button"
        class="mic-trigger-btn"
        class:listening={isListening}
        onclick={toggleVoiceDictation}
        title={isListening ? "Ferma dettatura vocale" : "Avvia dettatura vocale (it-IT)"}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none">
          <rect
            x="9"
            y="3"
            width="6"
            height="11"
            rx="3"
            stroke="currentColor"
            stroke-width="1.8"
          />
          <path
            d="M5 11a7 7 0 0 0 14 0M12 18v3"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
          />
        </svg>
      </button>

      <button
        type="submit"
        class="run-cmd-btn"
        disabled={isProcessing || !promptText.trim()}
      >
        {isProcessing ? "Analisi..." : "Esegui ↵"}
      </button>
    </form>

    <!-- Barra Stato Provider + Quick Chips -->
    <div class="subbar-meta">
      <div class="provider-pill">
        <span class="dot"></span>
        <span>Provider attivo: <strong>{providerLabel}</strong></span>
        <button
          type="button"
          class="config-link-btn"
          onclick={() => {
            onClose();
            onOpenSettings();
          }}
        >
          Configura AI
        </button>
      </div>
      <span class="shortcut-hint">Premi <kbd>Esc</kbd> per chiudere</span>
    </div>

    {#if voiceError}
      <div class="alert-banner warning">{voiceError}</div>
    {/if}

    {#if errorMsg}
      <div class="alert-banner critical">{errorMsg}</div>
    {/if}

    <!-- Suggerimenti Rapidi di Tool Calling -->
    <div class="suggestions-section">
      <span class="section-caption">AZIONI & INTERROGAZIONI RAPIDE</span>
      <div class="chips-grid">
        {#each quickSuggestions as item (item.label)}
          <button
            type="button"
            class="suggestion-chip"
            onclick={() => handleExecutePrompt(item.prompt)}
          >
            <span class="chip-title">{item.label}</span>
            <span class="chip-sub">{item.prompt}</span>
          </button>
        {/each}
      </div>
    </div>

    <!-- Risposta Strutturata & Tool Calls Eseguiti -->
    {#if lastResponse}
      <div class="response-card">
        <div class="response-header">
          <div class="response-title-group">
            <span class="ai-badge">Risposta Assistente</span>
            <span class="provider-tag">{lastResponse.providerUsed}</span>
          </div>

          {#if lastResponse.navigateToDate}
            <button
              type="button"
              class="jump-date-btn"
              onclick={() => {
                if (lastResponse?.navigateToDate) {
                  onNavigateDate(lastResponse.navigateToDate);
                  onClose();
                }
              }}
            >
              Vai a {lastResponse.navigateToDate} →
            </button>
          {/if}
        </div>

        {#if lastResponse.executedTools.length > 0}
          <div class="tools-executed-row">
            {#each lastResponse.executedTools as tool, idx (idx)}
              <div class="tool-pill">
                <span class="tool-fn">⚡ {tool.toolName}</span>
                <span class="tool-desc">{tool.summary}</span>
              </div>
            {/each}
          </div>
        {/if}

        <div class="markdown-reply">
          <!-- eslint-disable-next-line svelte/no-at-html-tags -->
          {@html renderedReplyHtml}
        </div>
      </div>
    {:else}
      <div class="syntax-guide-box">
        <div class="guide-title">Esempi di Comandi Naturali (Tool Calling in Tempo Reale)</div>
        <ul class="guide-list">
          <li>
            <code>Aggiungi corso Analisi 2 da 10 CFU</code> — Crea la materia nella sessione attiva.
          </li>
          <li>
            <code>Aggiungi appello di Analisi 2 il 2027-01-25 1° Appello alle 09:30</code> — Pianifica
            l'esame e ricalcola il Conflict Engine.
          </li>
          <li>
            <code>Aggiungi task domani: Ripetere teoremi di Gauss</code> — Inserisce un To-Do nella data
            indicata.
          </li>
          <li>
            <code>Qual è il corso più critico?</code> oppure <code>Quanti CFU ho a gennaio?</code> — Interroga
            il motore deterministico di telemetria.
          </li>
          <li>
            <code>Imposta tema scuro</code> / <code>Materiale acrylic</code> / <code>Zoom 115%</code> — Modifica
            in tempo reale il Windowing DWM e l'interfaccia.
          </li>
        </ul>
      </div>
    {/if}
  </div>
</div>

<style>
  .command-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding: 4.5rem 1.25rem 1.5rem;
    z-index: 380;
  }

  .command-sheet {
    width: 100%;
    max-width: 680px;
    background: var(--popover-bg);
    backdrop-filter: blur(30px) saturate(200%);
    -webkit-backdrop-filter: blur(30px) saturate(200%);
    border: 0.5px solid var(--hairline-border-strong);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-modal);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .spotlight-bar {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    padding: 0.85rem 1rem;
    border-bottom: 0.5px solid var(--hairline-border);
    background: rgba(128, 128, 128, 0.05);
  }

  .sparkle-badge {
    width: 32px;
    height: 32px;
    border-radius: 9px;
    background: color-mix(in srgb, var(--accent-primary) 16%, transparent);
    color: var(--accent-primary);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .sparkle-badge.pulsing {
    animation: pulseGlow 1.2s infinite ease-in-out;
  }

  @keyframes pulseGlow {
    0%,
    100% {
      transform: scale(1);
      opacity: 1;
    }
    50% {
      transform: scale(1.08);
      opacity: 0.75;
    }
  }

  .spotlight-input {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--text-primary);
    font-size: 0.95rem;
    font-weight: 500;
    outline: none;
    min-width: 0;
  }

  .spotlight-input::placeholder {
    color: var(--text-muted);
  }

  .voice-waveform {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    height: 18px;
    padding: 0 0.35rem;
  }

  .voice-waveform span {
    width: 3px;
    height: 100%;
    border-radius: 99px;
    background: #ef4444;
    animation: soundWave 0.75s infinite ease-in-out alternate;
  }

  .voice-waveform span:nth-child(2) {
    animation-delay: 0.12s;
  }
  .voice-waveform span:nth-child(3) {
    animation-delay: 0.24s;
  }
  .voice-waveform span:nth-child(4) {
    animation-delay: 0.36s;
  }
  .voice-waveform span:nth-child(5) {
    animation-delay: 0.48s;
  }

  @keyframes soundWave {
    0% {
      transform: scaleY(0.28);
    }
    100% {
      transform: scaleY(1);
    }
  }

  .mic-trigger-btn {
    width: 32px;
    height: 32px;
    border-radius: 8px;
    border: 0.5px solid var(--hairline-border);
    background: rgba(128, 128, 128, 0.08);
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 140ms ease;
  }

  .mic-trigger-btn:hover {
    color: var(--text-primary);
    background: rgba(128, 128, 128, 0.16);
  }

  .mic-trigger-btn.listening {
    background: rgba(239, 68, 68, 0.18);
    border-color: rgba(239, 68, 68, 0.55);
    color: #ef4444;
  }

  .run-cmd-btn {
    padding: 0.42rem 0.85rem;
    border-radius: 8px;
    border: none;
    background: var(--accent-primary);
    color: #ffffff;
    font-size: 0.76rem;
    font-weight: 600;
    cursor: pointer;
    flex-shrink: 0;
  }

  .run-cmd-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .subbar-meta {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.45rem 1rem;
    border-bottom: 0.5px solid var(--hairline-border);
    font-size: 0.7rem;
    color: var(--text-muted);
  }

  .provider-pill {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .provider-pill .dot {
    width: 6px;
    height: 6px;
    border-radius: 99px;
    background: #10b981;
  }

  .config-link-btn {
    border: none;
    background: transparent;
    color: var(--accent-primary);
    font-size: 0.7rem;
    font-weight: 600;
    cursor: pointer;
    padding: 0 0.25rem;
    text-decoration: underline;
  }

  .shortcut-hint kbd {
    padding: 0.08rem 0.32rem;
    border-radius: 4px;
    border: 0.5px solid var(--hairline-border);
    background: rgba(128, 128, 128, 0.1);
    font-size: 0.65rem;
  }

  .alert-banner {
    margin: 0.65rem 1rem 0;
    padding: 0.5rem 0.75rem;
    border-radius: var(--radius-md);
    font-size: 0.75rem;
  }

  .alert-banner.warning {
    background: rgba(245, 158, 11, 0.14);
    border: 0.5px solid rgba(245, 158, 11, 0.4);
    color: #fbbf24;
  }

  .alert-banner.critical {
    background: rgba(239, 68, 68, 0.14);
    border: 0.5px solid rgba(239, 68, 68, 0.4);
    color: #f87171;
  }

  .suggestions-section {
    padding: 0.75rem 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    border-bottom: 0.5px solid var(--hairline-border);
  }

  .section-caption {
    font-size: 0.63rem;
    font-weight: 700;
    letter-spacing: 0.06em;
    color: var(--text-muted);
  }

  .chips-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 0.4rem;
  }

  .suggestion-chip {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.12rem;
    padding: 0.42rem 0.6rem;
    border-radius: var(--radius-md);
    border: 0.5px solid var(--hairline-border);
    background: rgba(128, 128, 128, 0.06);
    text-align: left;
    cursor: pointer;
    transition: all 120ms ease;
  }

  .suggestion-chip:hover {
    background: rgba(128, 128, 128, 0.14);
    border-color: var(--hairline-border-strong);
  }

  .chip-title {
    font-size: 0.73rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .chip-sub {
    font-size: 0.64rem;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }

  .response-card {
    padding: 0.9rem 1rem 1.05rem;
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    max-height: 290px;
    overflow-y: auto;
  }

  .response-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .response-title-group {
    display: flex;
    align-items: center;
    gap: 0.45rem;
  }

  .ai-badge {
    font-size: 0.68rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--accent-primary);
  }

  .provider-tag {
    font-size: 0.64rem;
    padding: 0.1rem 0.42rem;
    border-radius: var(--radius-pill);
    background: rgba(128, 128, 128, 0.12);
    color: var(--text-secondary);
  }

  .jump-date-btn {
    padding: 0.26rem 0.65rem;
    border-radius: var(--radius-pill);
    border: 0.5px solid var(--accent-primary);
    background: color-mix(in srgb, var(--accent-primary) 15%, transparent);
    color: var(--accent-primary);
    font-size: 0.72rem;
    font-weight: 600;
    cursor: pointer;
  }

  .tools-executed-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  .tool-pill {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.24rem 0.58rem;
    border-radius: var(--radius-md);
    background: rgba(16, 185, 129, 0.12);
    border: 0.5px solid rgba(16, 185, 129, 0.35);
    font-size: 0.7rem;
  }

  .tool-fn {
    font-family: var(--font-mono);
    font-weight: 700;
    color: #10b981;
  }

  .tool-desc {
    color: var(--text-primary);
  }

  .markdown-reply {
    font-size: 0.82rem;
    line-height: 1.5;
    color: var(--text-primary);
  }

  .markdown-reply :global(p) {
    margin: 0 0 0.45rem;
  }

  .markdown-reply :global(ul) {
    margin: 0.25rem 0 0.5rem;
    padding-left: 1.2rem;
  }

  .syntax-guide-box {
    padding: 0.85rem 1rem 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
  }

  .guide-title {
    font-size: 0.74rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .guide-list {
    margin: 0;
    padding-left: 1.15rem;
    display: flex;
    flex-direction: column;
    gap: 0.32rem;
    font-size: 0.74rem;
    color: var(--text-muted);
  }

  .guide-list code {
    color: var(--text-primary);
    background: rgba(128, 128, 128, 0.12);
    padding: 0.08rem 0.35rem;
    border-radius: 4px;
    font-size: 0.71rem;
  }
</style>
