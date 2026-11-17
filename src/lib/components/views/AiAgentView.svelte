<script lang="ts">
  import DOMPurify from "dompurify";
  import { Bot, CalendarClock, ImagePlus, RotateCcw, Send, Terminal, X } from "lucide-svelte";
  import { marked } from "marked";
  import { onMount } from "svelte";
  import { ipc } from "../../api/ipc";
  import { navStore } from "../../stores/navigation.svelte";
  import { sessionStore } from "../../stores/session.svelte";
  import { themeStore } from "../../stores/theme.svelte";
  import type { AiProvider, AiToolCallRecord, CliAgentInfo } from "../../types/domain";

  interface ChatMessage {
    id: string;
    role: "user" | "assistant";
    text: string;
    imagePreview?: string | null;
    provider?: string;
    tools?: AiToolCallRecord[];
    undoSnapshotId?: string | null;
    undone?: boolean;
  }

  let snap = $derived(sessionStore.activeSnapshot);
  let cliAgents = $state<CliAgentInfo[]>([]);
  let messages = $state<ChatMessage[]>([]);
  let promptInput = $state("");
  let attachedImageBase64 = $state<string | null>(null);
  let attachedMimeType = $state<string>("image/png");
  let isSending = $state(false);
  let fileInputEl = $state<HTMLInputElement | null>(null);

  const PROVIDERS: Array<{ id: AiProvider; label: string }> = [
    { id: "local", label: "Locale (Qwen2.5-VL 3B)" },
    { id: "cli_agy", label: "Antigravity CLI (agy)" },
    { id: "cli_claude", label: "Claude Code" },
    { id: "cli_codex", label: "Codex CLI" },
    { id: "cli_custom", label: "CLI Custom" }
  ];

  onMount(async () => {
    try {
      cliAgents = await ipc.getCliAgentsStatus();
    } catch {
      cliAgents = [];
    }
  });

  async function selectProvider(id: AiProvider) {
    await themeStore.updateSettings({ aiProvider: id });
  }

  function handleFileChange(e: Event) {
    const file = (e.currentTarget as HTMLInputElement).files?.[0];
    if (!file) return;
    attachedMimeType = file.type || "image/png";
    const reader = new FileReader();
    reader.onload = () => {
      attachedImageBase64 = String(reader.result || "");
    };
    reader.readAsDataURL(file);
  }

  function handlePaste(e: ClipboardEvent) {
    const item = Array.from(e.clipboardData?.items ?? []).find((i) => i.type.startsWith("image/"));
    if (!item) return;
    const file = item.getAsFile();
    if (!file) return;
    attachedMimeType = file.type || "image/png";
    const reader = new FileReader();
    reader.onload = () => {
      attachedImageBase64 = String(reader.result || "");
    };
    reader.readAsDataURL(file);
  }

  async function sendPrompt(overridePrompt?: string) {
    if (!snap || isSending) return;
    const txt = (overridePrompt ?? promptInput).trim();
    if (!txt && !attachedImageBase64) return;

    const userMsg: ChatMessage = {
      id: `u_${Date.now()}`,
      role: "user",
      text: txt || "Analizza immagine allegata",
      imagePreview: attachedImageBase64
    };
    messages = [...messages, userMsg];
    const b64 = attachedImageBase64;
    const mime = attachedMimeType;
    if (!overridePrompt) promptInput = "";
    attachedImageBase64 = null;
    isSending = true;

    try {
      const res = await ipc.executeAiCommand(snap.session.id, userMsg.text, b64, mime);
      if (res.updatedSnapshot) {
        sessionStore.activeSnapshot = res.updatedSnapshot;
      }
      if (res.navigateToDate) {
        navStore.todayTargetDate = res.navigateToDate;
        navStore.calendarSelectedDate = res.navigateToDate;
      }

      messages = [
        ...messages,
        {
          id: `a_${Date.now()}`,
          role: "assistant",
          text: res.replyMarkdown,
          provider: res.providerUsed,
          tools: res.executedTools,
          undoSnapshotId: res.undoSnapshotId
        }
      ];
    } catch (err) {
      messages = [
        ...messages,
        {
          id: `e_${Date.now()}`,
          role: "assistant",
          text: `**Errore AI:** ${String(err)}`
        }
      ];
    } finally {
      isSending = false;
    }
  }

  async function handleUndo(msgId: string, snapshotId: string) {
    try {
      const restored = await ipc.undoAiAction(snapshotId);
      sessionStore.activeSnapshot = restored;
      messages = messages.map((m) => (m.id === msgId ? { ...m, undone: true } : m));
      navStore.showToast("Modifiche AI annullate!", "info");
    } catch (err) {
      navStore.showToast(`Errore Undo: ${String(err)}`, "error");
    }
  }

  function renderMd(md: string): string {
    const raw = marked.parse(md, { async: false }) as string;
    return DOMPurify.sanitize(raw);
  }
</script>

<div class="ai-layout bento-enter" onpaste={handlePaste}>
  <header class="bento-card ai-top-bar">
    <div class="ai-title">
      <div class="bot-orb"><Bot size={20} /></div>
      <div>
        <h1>Agente AI</h1>
        <p>Modello locale leggero o Agenti CLI installati (Antigravity, Claude Code, Codex)</p>
      </div>
    </div>

    <div class="provider-pills">
      {#each PROVIDERS as p (p.id)}
        {@const cliInfo = cliAgents.find((c) => c.id === p.id)}
        {@const active = themeStore.settings.aiProvider === p.id}
        <button
          type="button"
          class="pill-filter"
          class:active
          onclick={() => selectProvider(p.id)}
        >
          {#if p.id !== "local"}
            <Terminal size={12} />
            <span class="cli-dot" class:ok={cliInfo?.installed}></span>
          {/if}
          {p.label}
        </button>
      {/each}
    </div>
  </header>

  <div class="bento-card ai-chat-box">
    <div class="chat-scroll">
      {#if messages.length === 0}
        <div class="chat-welcome">
          <h3>Cosa vuoi fare oggi?</h3>
          <p>Fai domande sulla tua sessione, chiedi di spostare task, genera un quiz extra oppure scansiona la foto dell'orario lezioni.</p>
          <div class="quick-prompts">
            <button type="button" class="pill-btn pill-btn-soft" onclick={() => sendPrompt("Cosa devo studiare oggi e quali sono le priorità?")}>
              Qual è la priorità di oggi?
            </button>
            <button type="button" class="pill-btn pill-btn-soft" onclick={() => sendPrompt("Sposta a domani le task non completate di oggi")}>
              Sposta task non finite a domani
            </button>
            <button type="button" class="pill-btn pill-btn-soft" onclick={() => (navStore.isTimetableModalOpen = true)}>
              <CalendarClock size={14} />
              Scansiona foto orario lezioni
            </button>
          </div>
        </div>
      {:else}
        {#each messages as msg (msg.id)}
          <div class="msg-row" class:user={msg.role === "user"}>
            <div class="msg-bubble">
              {#if msg.imagePreview}
                <img src={msg.imagePreview} alt="Allegato" class="msg-img" />
              {/if}
              <div class="md-body">{@html renderMd(msg.text)}</div>

              {#if msg.tools && msg.tools.length > 0}
                <div class="tools-executed">
                  {#each msg.tools as t, i (i)}
                    <span class="tool-pill">{t.summary}</span>
                  {/each}
                </div>
              {/if}

              {#if msg.undoSnapshotId && !msg.undone}
                <button
                  type="button"
                  class="undo-btn"
                  onclick={() => handleUndo(msg.id, msg.undoSnapshotId!)}
                >
                  <RotateCcw size={13} />
                  Annulla modifiche (Undo)
                </button>
              {:else if msg.undone}
                <span class="undone-tag">Modifiche annullate</span>
              {/if}
            </div>
          </div>
        {/each}
      {/if}
    </div>

    {#if attachedImageBase64}
      <div class="attach-preview">
        <img src={attachedImageBase64} alt="Anteprima immagine" />
        <button type="button" class="rm-img" onclick={() => (attachedImageBase64 = null)}>
          <X size={14} />
        </button>
      </div>
    {/if}

    <div class="chat-input-bar">
      <input
        type="file"
        accept="image/*"
        class="hidden-file"
        bind:this={fileInputEl}
        onchange={handleFileChange}
      />
      <button
        type="button"
        class="icon-btn"
        title="Allega foto (o incolla con Ctrl+V)"
        onclick={() => fileInputEl?.click()}
      >
        <ImagePlus size={18} />
      </button>
      <input
        type="text"
        class="chat-input"
        placeholder="Chiedi qualcosa o ordina una modifica al piano... (Ctrl+V per incollare foto)"
        bind:value={promptInput}
        onkeydown={(e) => e.key === "Enter" && sendPrompt()}
      />
      <button
        type="button"
        class="pill-btn pill-btn-primary"
        disabled={isSending}
        onclick={() => sendPrompt()}
      >
        <Send size={15} />
        {isSending ? "Elaboro..." : "Invia"}
      </button>
    </div>
  </div>
</div>

<style>
  .ai-layout {
    display: flex;
    flex-direction: column;
    gap: 14px;
    height: calc(100vh - 32px);
  }

  .ai-top-bar {
    padding: 16px 20px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
    flex-wrap: wrap;
  }

  .ai-title {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .bot-orb {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: var(--hero-gradient);
    display: grid;
    place-items: center;
  }

  .ai-title h1 {
    margin: 0;
    font-size: 19px;
    font-weight: 800;
  }

  .ai-title p {
    margin: 2px 0 0;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .provider-pills {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .cli-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--text-muted);
  }

  .cli-dot.ok {
    background: var(--success);
  }

  .ai-chat-box {
    flex: 1;
    padding: 18px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    overflow: hidden;
  }

  .chat-scroll {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding-right: 4px;
  }

  .chat-welcome {
    margin: auto;
    max-width: 560px;
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
  }

  .chat-welcome h3 {
    margin: 0;
    font-size: 20px;
  }

  .chat-welcome p {
    margin: 0;
    font-size: 13.5px;
    color: var(--text-secondary);
  }

  .quick-prompts {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 8px;
    margin-top: 8px;
  }

  .msg-row {
    display: flex;
    justify-content: flex-start;
  }

  .msg-row.user {
    justify-content: flex-end;
  }

  .msg-bubble {
    max-width: 78%;
    padding: 12px 16px;
    border-radius: 18px;
    background: var(--bg-surface-muted);
    display: flex;
    flex-direction: column;
    gap: 8px;
    font-size: 13.5px;
  }

  .msg-row.user .msg-bubble {
    background: var(--bg-surface-dark);
    color: #fff;
  }

  :global(html[data-theme="dark"]) .msg-row.user .msg-bubble {
    background: var(--accent);
  }

  .msg-img {
    max-width: 240px;
    border-radius: 12px;
  }

  .md-body :global(p) {
    margin: 0 0 6px;
  }
  .md-body :global(p:last-child) {
    margin-bottom: 0;
  }

  .tools-executed {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .tool-pill {
    padding: 3px 9px;
    border-radius: 999px;
    background: var(--accent-soft);
    color: var(--accent);
    font-size: 11.5px;
    font-weight: 700;
  }

  .undo-btn {
    align-self: flex-start;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 11px;
    border-radius: 999px;
    background: rgba(239, 68, 68, 0.14);
    color: #DC2626;
    font-size: 12px;
    font-weight: 700;
  }

  .undone-tag {
    font-size: 11.5px;
    color: var(--text-muted);
    font-style: italic;
  }

  .attach-preview {
    position: relative;
    align-self: flex-start;
  }

  .attach-preview img {
    height: 76px;
    border-radius: 12px;
    border: 1px solid var(--border-subtle);
  }

  .rm-img {
    position: absolute;
    top: -6px;
    right: -6px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: var(--bg-surface-dark);
    color: #fff;
    display: grid;
    place-items: center;
  }

  .chat-input-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    border-radius: 18px;
    background: var(--bg-surface-muted);
  }

  .hidden-file {
    display: none;
  }

  .icon-btn {
    width: 34px;
    height: 34px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    color: var(--text-secondary);
  }

  .icon-btn:hover {
    background: var(--bg-surface);
    color: var(--text-primary);
  }

  .chat-input {
    flex: 1;
    border: none;
    background: transparent;
    color: var(--text-primary);
    font-size: 13.5px;
    outline: none;
  }
</style>
