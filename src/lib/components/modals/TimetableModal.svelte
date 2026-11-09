<script lang="ts">
  import { onMount } from "svelte";
  import { sessionStore } from "../../stores/session.svelte";
  import type { ExtractedLessonItem, WeeklyLesson, UpsertWeeklyLessonInput } from "../../types/domain";
  import { UNIFIED_STANDARD_PALETTE, generateCourseColorTokens } from "../../utils/color";
  import { themeStore } from "../../stores/theme.svelte";
  import { ipc } from "../../api/ipc";
  import {
    Sparkles,
    Calendar,
    Plus,
    Trash2,
    Edit2,
    Upload,
    Check,
    X,
    Clock,
    MapPin,
    User,
    ArrowRight,
    ArrowLeft
  } from "lucide-svelte";
  import CustomSelect, { type SelectOption } from "../common/CustomSelect.svelte";

  interface Props {
    onClose?: () => void;
    onToast?: (msg: string) => void;
  }

  let { onClose, onToast }: Props = $props();

  let activeSession = $derived(sessionStore.activeSession);
  let courses = $derived(sessionStore.courses);
  let lessons = $derived(sessionStore.weeklyLessons);
  let isDark = $derived(themeStore.effectiveTheme === "dark");

  const DAYS = [
    { id: 1, name: "Lunedì", short: "Lun" },
    { id: 2, name: "Martedì", short: "Mar" },
    { id: 3, name: "Mercoledì", short: "Mer" },
    { id: 4, name: "Giovedì", short: "Gio" },
    { id: 5, name: "Venerdì", short: "Ven" },
    { id: 6, name: "Sabato", short: "Sab" },
    { id: 7, name: "Domenica", short: "Dom" }
  ];

  const daySelectOptions: SelectOption<number>[] = DAYS.map((d) => ({
    value: d.id,
    label: d.name
  }));

  const courseSelectOptions = $derived<SelectOption<string>[]>([
    { value: "", label: "-- Materia personalizzata / non in elenco --" },
    ...courses.map((c) => ({
      value: c.id,
      label: `${c.sigla} — ${c.name}`,
      description: c.professor ? `Prof. ${c.professor}` : undefined,
      badge: `${c.cfu} CFU`
    }))
  ]);

  let showWeekend = $state(false);
  let visibleDays = $derived(showWeekend ? DAYS : DAYS.slice(0, 5));

  // Stato sub-pannelli
  let isAiVisionOpen = $state(false);
  let isProjectModalOpen = $state(false);
  let lessonEditingState = $state<{
    open: boolean;
    lesson: WeeklyLesson | null;
    dayOfWeek: number;
    courseId: string;
    courseName: string;
    startTime: string;
    endTime: string;
    classroom: string;
    professor: string;
    colorHex: string;
  }>({
    open: false,
    lesson: null,
    dayOfWeek: 1,
    courseId: "",
    courseName: "",
    startTime: "09:00",
    endTime: "11:00",
    classroom: "",
    professor: "",
    colorHex: UNIFIED_STANDARD_PALETTE[0]
  });

  // Stato Assistente AI (Vision + Prompt / Dettatura Vocale)
  let aiModalTab = $state<"vision" | "prompt">("vision");
  let promptText = $state("");
  let isListening = $state(false);
  let voiceSupported = $state(false);
  let voiceError = $state<string | null>(null);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let recognitionInstance: any = null;

  let aiImagePreview = $state<string | null>(null);
  let aiImageBase64 = $state<string | null>(null);
  let aiImageMime = $state<string>("image/png");
  let isAiAnalyzing = $state(false);
  let aiError = $state<string | null>(null);
  let extractedLessons = $state<ExtractedLessonItem[]>([]);
  let isSavingExtracted = $state(false);

  // Stato Proiezione Calendario
  let projectStartDate = $state(new Date().toISOString().slice(0, 10));
  let projectEndDate = $state(new Date().toISOString().slice(0, 10));
  let isProjecting = $state(false);

  function openAiVisionModal() {
    isAiVisionOpen = true;
    if (themeStore.settings.aiProvider === "local" && themeStore.settings.aiBootStrategy === "contextual") {
      ipc.startAiRuntime(themeStore.settings.aiLocalModelId).catch(() => {});
    }
  }

  $effect(() => {
    if (activeSession) {
      if (activeSession.startDate) projectStartDate = activeSession.startDate;
      if (activeSession.endDate) projectEndDate = activeSession.endDate;
    }
  });

  onMount(() => {
    initSpeechRecognition();
    const handlePaste = (e: ClipboardEvent) => {
      if (!isAiVisionOpen) return;
      if (e.target instanceof HTMLTextAreaElement || e.target instanceof HTMLInputElement) return;
      const items = e.clipboardData?.items;
      if (!items) return;
      for (let i = 0; i < items.length; i++) {
        if (items[i].type.startsWith("image/")) {
          const file = items[i].getAsFile();
          if (file) {
            aiModalTab = "vision";
            handleFileSelected(file);
            break;
          }
        }
      }
    };
    window.addEventListener("paste", handlePaste);
    return () => {
      window.removeEventListener("paste", handlePaste);
      if (isListening) {
        try {
          recognitionInstance?.stop();
        } catch {}
      }
    };
  });

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
      isListening = true;
    };

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    rec.onresult = (event: any) => {
      let transcript = "";
      for (let i = event.resultIndex; i < event.results.length; i++) {
        transcript += event.results[i][0].transcript;
      }
      if (transcript.trim()) {
        if (!promptText || promptText.endsWith(". ") || promptText.endsWith("\n")) {
          promptText += transcript.trim();
        } else {
          promptText += " " + transcript.trim();
        }
      }
    };

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    rec.onerror = (event: any) => {
      isListening = false;
      if (event?.error && event.error !== "no-speech" && event.error !== "aborted") {
        voiceError = `Dettatura vocale non disponibile (${event.error}). Puoi digitare o incollare il testo.`;
      }
    };

    rec.onend = () => {
      isListening = false;
    };

    recognitionInstance = rec;
  }

  function toggleVoiceDictation() {
    voiceError = null;
    if (!voiceSupported || !recognitionInstance) {
      voiceError = "Dettatura vocale non supportata dal runtime di sistema. Puoi digitare o incollare il testo.";
      return;
    }

    if (isListening) {
      try {
        recognitionInstance.stop();
      } catch {}
      isListening = false;
    } else {
      try {
        recognitionInstance.start();
      } catch {
        isListening = false;
      }
    }
  }

  async function handleAnalyzePrompt() {
    if (!promptText.trim()) {
      aiError = "Inserisci o detta a voce l'orario delle tue lezioni prima di avviare l'analisi.";
      return;
    }
    if (isListening) {
      try {
        recognitionInstance?.stop();
      } catch {}
      isListening = false;
    }

    const analysisId = ++currentAnalysisId;
    isAiAnalyzing = true;
    aiError = null;
    extractedLessons = [];

    try {
      const items = await sessionStore.parseTimetablePrompt(promptText.trim());
      if (currentAnalysisId !== analysisId) return;
      if (!items || items.length === 0) {
        aiError = "Nessuna lezione identificata nel testo fornito. Prova a specificare giorni, orari (es. 08:30-10:30) e nomi delle materie.";
      } else {
        extractedLessons = items;
      }
    } catch (e) {
      if (currentAnalysisId !== analysisId) return;
      aiError = e instanceof Error ? e.message : String(e);
    } finally {
      if (currentAnalysisId === analysisId) {
        isAiAnalyzing = false;
      }
    }
  }

  function getLessonsForDay(dayOfWeek: number): WeeklyLesson[] {
    return lessons
      .filter((l) => l.dayOfWeek === dayOfWeek)
      .sort((a, b) => a.startTime.localeCompare(b.startTime));
  }

  function openNewLessonModal(dayOfWeek = 1) {
    lessonEditingState = {
      open: true,
      lesson: null,
      dayOfWeek,
      courseId: courses[0]?.id || "",
      courseName: courses[0]?.name || "",
      startTime: "09:00",
      endTime: "11:00",
      classroom: "",
      professor: courses[0]?.professor || "",
      colorHex: courses[0]?.colorHex || UNIFIED_STANDARD_PALETTE[0]
    };
  }

  function openEditLessonModal(l: WeeklyLesson) {
    lessonEditingState = {
      open: true,
      lesson: l,
      dayOfWeek: l.dayOfWeek,
      courseId: l.courseId || "",
      courseName: l.courseName,
      startTime: l.startTime,
      endTime: l.endTime,
      classroom: l.classroom || "",
      professor: l.professor || "",
      colorHex: l.colorHex || UNIFIED_STANDARD_PALETTE[0]
    };
  }

  function handleCourseSelect(courseId: string) {
    lessonEditingState.courseId = courseId;
    const c = courses.find((x) => x.id === courseId);
    if (c) {
      lessonEditingState.courseName = c.name;
      if (!lessonEditingState.professor) lessonEditingState.professor = c.professor;
      lessonEditingState.colorHex = c.colorHex;
    }
  }

  async function handleSaveLesson() {
    if (!lessonEditingState.courseName.trim()) {
      alert("Inserisci o seleziona una materia.");
      return;
    }
    if (!lessonEditingState.startTime || !lessonEditingState.endTime) {
      alert("Inserisci gli orari di inizio e fine.");
      return;
    }

    try {
      const input: UpsertWeeklyLessonInput = {
        id: lessonEditingState.lesson?.id || null,
        sessionId: activeSession?.id || "",
        courseId: lessonEditingState.courseId || null,
        courseName: lessonEditingState.courseName.trim(),
        dayOfWeek: lessonEditingState.dayOfWeek,
        startTime: lessonEditingState.startTime,
        endTime: lessonEditingState.endTime,
        classroom: lessonEditingState.classroom.trim() || null,
        professor: lessonEditingState.professor.trim() || null,
        colorHex: lessonEditingState.colorHex || null
      };

      await sessionStore.upsertWeeklyLesson(input);
      lessonEditingState.open = false;
      onToast?.(lessonEditingState.lesson ? "Lezione aggiornata!" : "Lezione aggiunta all'orario!");
    } catch (e) {
      alert("Errore nel salvataggio: " + (e instanceof Error ? e.message : String(e)));
    }
  }

  async function handleDeleteLesson(lessonId: string) {
    if (!confirm("Rimuovere questa lezione dall'orario settimanale?")) return;
    try {
      await sessionStore.deleteWeeklyLesson(lessonId);
      onToast?.("Lezione rimossa dall'orario.");
    } catch (e) {
      alert("Errore eliminazione: " + (e instanceof Error ? e.message : String(e)));
    }
  }

  // Vision AI handlers
  function handleFileInputChange(e: Event) {
    const input = e.target as HTMLInputElement;
    if (input.files && input.files[0]) {
      handleFileSelected(input.files[0]);
    }
  }

  function handleFileDrop(e: DragEvent) {
    e.preventDefault();
    if (e.dataTransfer?.files && e.dataTransfer.files[0]) {
      handleFileSelected(e.dataTransfer.files[0]);
    }
  }

  let currentAnalysisId = 0;

  function handleCancelVisionModal() {
    currentAnalysisId++;
    if (isListening) {
      try {
        recognitionInstance?.stop();
      } catch {}
      isListening = false;
    }
    isAiAnalyzing = false;
    isAiVisionOpen = false;
    aiImagePreview = null;
    aiImageBase64 = null;
    aiError = null;
    voiceError = null;
    extractedLessons = [];
  }

  async function optimizeImageForVision(file: File): Promise<{ base64: string; mime: string }> {
    return new Promise((resolve, reject) => {
      if (!file.type.startsWith("image/") || file.type.includes("svg")) {
        const reader = new FileReader();
        reader.onload = () => resolve({ base64: reader.result as string, mime: file.type || "image/png" });
        reader.onerror = reject;
        reader.readAsDataURL(file);
        return;
      }

      const img = new Image();
      const objectUrl = URL.createObjectURL(file);
      img.onload = () => {
        URL.revokeObjectURL(objectUrl);
        const maxDimension = 2560;
        let width = img.width;
        let height = img.height;

        let scale = 1.0;

        // Se l'immagine è un ritaglio o ha altezza ridotta con testo denso (es. ritaglio da portale Esse3/PoliMi 753x222 o height < 600)
        // ingrandiamo proporzionalmente per garantire che ogni riga di testo abbia risoluzione OCR nitida
        if (height < 600 && width < 1800) {
          scale = Math.min(2.5, Math.max(1.5, 750 / height));
        } else if (width > maxDimension || height > maxDimension) {
          scale = maxDimension / Math.max(width, height);
        }

        // Se non c'è bisogno di scalare ed è leggera, mantieni lossless nativo
        if (scale === 1.0 && file.size < 5 * 1024 * 1024) {
          const reader = new FileReader();
          reader.onload = () => resolve({ base64: reader.result as string, mime: file.type || "image/png" });
          reader.onerror = reject;
          reader.readAsDataURL(file);
          return;
        }

        const targetWidth = Math.round(width * scale);
        const targetHeight = Math.round(height * scale);

        const canvas = document.createElement("canvas");
        canvas.width = targetWidth;
        canvas.height = targetHeight;
        const ctx = canvas.getContext("2d");
        if (!ctx) {
          const reader = new FileReader();
          reader.onload = () => resolve({ base64: reader.result as string, mime: file.type || "image/png" });
          reader.onerror = reject;
          reader.readAsDataURL(file);
          return;
        }

        ctx.imageSmoothingEnabled = true;
        ctx.imageSmoothingQuality = "high";
        ctx.drawImage(img, 0, 0, targetWidth, targetHeight);

        const targetMime = "image/png";
        const base64 = canvas.toDataURL(targetMime);
        resolve({ base64, mime: targetMime });
      };
      img.onerror = () => {
        URL.revokeObjectURL(objectUrl);
        const reader = new FileReader();
        reader.onload = () => resolve({ base64: reader.result as string, mime: file.type || "image/png" });
        reader.onerror = reject;
        reader.readAsDataURL(file);
      };
      img.src = objectUrl;
    });
  }

  async function handleFileSelected(file: File) {
    aiError = null;
    extractedLessons = [];

    try {
      const { base64, mime } = await optimizeImageForVision(file);
      aiImageMime = mime;
      aiImagePreview = base64;
      aiImageBase64 = base64;
      await runVisionAiAnalysis(base64, mime);
    } catch {
      aiError = "Impossibile leggere o ottimizzare il file selezionato.";
    }
  }

  async function runVisionAiAnalysis(base64: string, mime: string) {
    const analysisId = ++currentAnalysisId;
    isAiAnalyzing = true;
    aiError = null;
    try {
      const items = await sessionStore.parseTimetableImage(base64, mime);
      if (currentAnalysisId !== analysisId) return; // Annullato dall'utente
      if (!items || items.length === 0) {
        aiError = "Nessuna lezione identificata nell'immagine. Assicurati che l'orario sia ben visibile e leggibile.";
      } else {
        extractedLessons = items;
      }
    } catch (e) {
      if (currentAnalysisId !== analysisId) return; // Annullato dall'utente
      aiError = e instanceof Error ? e.message : String(e);
    } finally {
      if (currentAnalysisId === analysisId) {
        isAiAnalyzing = false;
      }
    }
  }

  async function handleConfirmExtractedLessons() {
    if (extractedLessons.length === 0) return;
    isSavingExtracted = true;
    try {
      let saved = 0;
      for (const item of extractedLessons) {
        // Cerca se esiste già un corso corrispondente per sigla/nome
        const matchedCourse = courses.find(
          (c) =>
            c.name.toLowerCase().includes(item.courseName.toLowerCase()) ||
            item.courseName.toLowerCase().includes(c.name.toLowerCase()) ||
            c.sigla.toLowerCase() === item.courseName.toLowerCase()
        );

        await sessionStore.upsertWeeklyLesson({
          id: null,
          sessionId: activeSession?.id || "",
          courseId: matchedCourse?.id || null,
          courseName: matchedCourse?.name || item.courseName,
          dayOfWeek: item.dayOfWeek,
          startTime: item.startTime,
          endTime: item.endTime,
          classroom: item.classroom || null,
          professor: item.professor || matchedCourse?.professor || null,
          colorHex: matchedCourse?.colorHex || UNIFIED_STANDARD_PALETTE[saved % UNIFIED_STANDARD_PALETTE.length]
        });
        saved++;
      }
      isAiVisionOpen = false;
      aiImagePreview = null;
      extractedLessons = [];
      onToast?.(`Salvate ${saved} lezioni nell'orario settimanale!`);
    } catch (e) {
      alert("Errore salvataggio lezioni: " + (e instanceof Error ? e.message : String(e)));
    } finally {
      isSavingExtracted = false;
    }
  }

  async function handleProjectToCalendar() {
    if (!projectStartDate || !projectEndDate) {
      alert("Seleziona sia la data di inizio che di fine.");
      return;
    }
    isProjecting = true;
    try {
      const count = await sessionStore.projectLessonsToCalendar(projectStartDate, projectEndDate);
      isProjectModalOpen = false;
      onToast?.(`Proiettate con successo ${count} lezioni nel calendario di sessione!`);
    } catch (e) {
      alert("Errore proiezione a calendario: " + (e instanceof Error ? e.message : String(e)));
    } finally {
      isProjecting = false;
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="modal-scrim"
  role="dialog"
  aria-modal="true"
  aria-label="Orario Settimanale delle Lezioni"
  tabindex="-1"
  onclick={(e) => {
    if (e.target === e.currentTarget && !lessonEditingState.open && !isAiVisionOpen && !isProjectModalOpen) {
      onClose?.();
    }
  }}
>
  <div class="timetable-modal-sheet">
    <!-- Header Apple HIG -->
    <header class="sheet-header">
      <div class="header-titles">
        <span class="sheet-eyebrow">ORARIO SETTIMANALE DELLE LEZIONI</span>
        <div class="title-row">
          <h2>Orario Lezioni</h2>
          {#if activeSession}
            <span class="session-pill">{activeSession.name}</span>
          {/if}
        </div>
      </div>

      <div class="header-actions">
        <!-- Toggle Fine Settimana -->
        <button
          type="button"
          class="btn-secondary pressable"
          class:active={showWeekend}
          onclick={() => (showWeekend = !showWeekend)}
          title="Mostra o nascondi Sabato e Domenica"
        >
          {showWeekend ? "Lun – Dom" : "Lun – Ven"}
        </button>

        <!-- AI Timetable Import CTA -->
        <button
          type="button"
          class="btn-ai pressable"
          onclick={openAiVisionModal}
          title="Importa l'orario con Intelligenza Artificiale (da Foto, Screenshot, Testo o Dettatura Vocale)"
        >
          <Sparkles size={14} />
          <span>Importa con IA (Foto / Voce)</span>
        </button>

        <!-- Proietta nel Calendario CTA -->
        <button
          type="button"
          class="btn-project pressable"
          onclick={() => (isProjectModalOpen = true)}
          disabled={lessons.length === 0}
          title="Proietta le lezioni settimanali come impegni nel calendario"
        >
          <Calendar size={14} />
          <span>Proietta a Calendario</span>
        </button>

        <!-- Nuova Lezione CTA -->
        <button
          type="button"
          class="btn-primary pressable"
          onclick={() => openNewLessonModal(1)}
        >
          <Plus size={14} />
          <span>Nuova Lezione</span>
        </button>

        <!-- Chiudi -->
        <button
          type="button"
          class="btn-close pressable"
          onclick={() => onClose?.()}
          aria-label="Chiudi finestra"
        >
          <X size={16} />
        </button>
      </div>
    </header>

    <!-- Griglia Settimanale -->
    <main class="timetable-grid-container">
      <div
        class="timetable-grid"
        style:grid-template-columns="repeat({visibleDays.length}, minmax(180px, 1fr))"
      >
        {#each visibleDays as day (day.id)}
          {@const dayLessons = getLessonsForDay(day.id)}
          <div class="day-column">
            <div class="day-header">
              <span class="day-name">{day.name}</span>
              <span class="day-count-badge" class:has-lessons={dayLessons.length > 0}>
                {dayLessons.length}
              </span>
            </div>

            <div class="day-content">
              {#if dayLessons.length === 0}
                <div class="day-empty-placeholder">
                  <span class="empty-text">Nessuna lezione</span>
                  <button
                    type="button"
                    class="btn-add-day pressable"
                    onclick={() => openNewLessonModal(day.id)}
                    title="Aggiungi lezione a {day.name}"
                  >
                    + Aggiungi
                  </button>
                </div>
              {:else}
                <div class="lesson-cards-list">
                  {#each dayLessons as lesson (lesson.id)}
                    {@const palette = generateCourseColorTokens(lesson.colorHex || "#007AFF", isDark)}
                    <div
                      class="lesson-card pressable"
                      style:--lesson-solid={palette.solid}
                      style:--lesson-tint={palette.bgTint}
                      style:--lesson-border={palette.borderTint}
                      style:--lesson-text={palette.textContrast}
                    >
                      <div class="card-time-row">
                        <span class="time-badge">
                          <Clock size={11} />
                          {lesson.startTime} – {lesson.endTime}
                        </span>
                        <div class="card-actions">
                          <button
                            type="button"
                            class="card-act-btn edit"
                            title="Modifica lezione"
                            onclick={() => openEditLessonModal(lesson)}
                          >
                            <Edit2 size={12} />
                          </button>
                          <button
                            type="button"
                            class="card-act-btn delete"
                            title="Elimina lezione"
                            onclick={() => handleDeleteLesson(lesson.id)}
                          >
                            <Trash2 size={12} />
                          </button>
                        </div>
                      </div>

                      <div class="card-subject-row">
                        <span class="subject-color-strip"></span>
                        <span class="subject-title">{lesson.courseName}</span>
                      </div>

                      {#if lesson.classroom || lesson.professor}
                        <div class="card-meta-row">
                          {#if lesson.classroom}
                            <span class="meta-item">
                              <MapPin size={11} />
                              {lesson.classroom}
                            </span>
                          {/if}
                          {#if lesson.professor}
                            <span class="meta-item">
                              <User size={11} />
                              {lesson.professor}
                            </span>
                          {/if}
                        </div>
                      {/if}
                    </div>
                  {/each}
                  <button
                    type="button"
                    class="btn-add-more pressable"
                    onclick={() => openNewLessonModal(day.id)}
                  >
                    + Altra lezione
                  </button>
                </div>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    </main>

    <!-- Sub-modale Inserisci / Modifica Lezione -->
    {#if lessonEditingState.open}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <div
        class="submodal-scrim"
        role="dialog"
        aria-modal="true"
        aria-label="Editor Lezione"
        tabindex="-1"
        onclick={(e) => {
          if (e.target === e.currentTarget) lessonEditingState.open = false;
        }}
      >
        <div class="submodal-card">
          <header class="submodal-header">
            <h3>{lessonEditingState.lesson ? "Modifica Lezione" : "Nuova Lezione Settimanale"}</h3>
            <button
              type="button"
              class="btn-close-sub pressable"
              onclick={() => (lessonEditingState.open = false)}
            >
              <X size={15} />
            </button>
          </header>

          <form
            class="submodal-form"
            onsubmit={(e) => {
              e.preventDefault();
              handleSaveLesson();
            }}
          >
            <!-- Giorno della settimana -->
            <div class="form-row">
              <span class="field-label">Giorno della settimana</span>
              <CustomSelect
                bind:value={lessonEditingState.dayOfWeek}
                options={daySelectOptions}
                ariaLabel="Giorno della settimana"
              />
            </div>

            <!-- Materia (da corsi esistenti o testo libero) -->
            <div class="form-row">
              <span class="field-label">Materia</span>
              {#if courses.length > 0}
                <div class="course-select-wrap">
                  <CustomSelect
                    value={lessonEditingState.courseId || ""}
                    options={courseSelectOptions}
                    onchange={(val) => handleCourseSelect(val)}
                    ariaLabel="Seleziona materia tra i corsi configurati"
                  />
                </div>
              {/if}
              <input
                id="lesson-course"
                type="text"
                bind:value={lessonEditingState.courseName}
                placeholder="Es. Analisi Matematica II"
                required
                class="input-text"
              />
            </div>

            <!-- Orari Inizio e Fine -->
            <div class="form-row-dual">
              <div class="form-col">
                <label for="lesson-start">Ora Inizio</label>
                <input
                  id="lesson-start"
                  type="time"
                  bind:value={lessonEditingState.startTime}
                  required
                  class="input-text"
                />
              </div>
              <div class="form-col">
                <label for="lesson-end">Ora Fine</label>
                <input
                  id="lesson-end"
                  type="time"
                  bind:value={lessonEditingState.endTime}
                  required
                  class="input-text"
                />
              </div>
            </div>

            <!-- Aula e Docente -->
            <div class="form-row-dual">
              <div class="form-col">
                <label for="lesson-room">Aula (opzionale)</label>
                <input
                  id="lesson-room"
                  type="text"
                  bind:value={lessonEditingState.classroom}
                  placeholder="Es. Aula Magna, Lab 4"
                  class="input-text"
                />
              </div>
              <div class="form-col">
                <label for="lesson-prof">Docente (opzionale)</label>
                <input
                  id="lesson-prof"
                  type="text"
                  bind:value={lessonEditingState.professor}
                  placeholder="Es. Prof. Rossi"
                  class="input-text"
                />
              </div>
            </div>

            <!-- Palette Colore -->
            <div class="form-row">
              <span class="label-palette">Colore Badge</span>
              <div class="palette-swatches">
                {#each UNIFIED_STANDARD_PALETTE as hex}
                  <button
                    type="button"
                    class="swatch-btn pressable"
                    class:selected={lessonEditingState.colorHex.toUpperCase() === hex.toUpperCase()}
                    style:background={hex}
                    onclick={() => (lessonEditingState.colorHex = hex)}
                    aria-label="Colore {hex}"
                  ></button>
                {/each}
              </div>
            </div>

            <footer class="submodal-footer">
              <button
                type="button"
                class="btn-secondary pressable"
                onclick={() => (lessonEditingState.open = false)}
              >
                Annulla
              </button>
              <button type="submit" class="btn-primary pressable">
                {lessonEditingState.lesson ? "Salva Modifiche" : "Aggiungi all'Orario"}
              </button>
            </footer>
          </form>
        </div>
      </div>
    {/if}

    <!-- Sub-modale Vision AI Caricamento Orario con Foto -->
    {#if isAiVisionOpen}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <div
        class="submodal-scrim"
        role="dialog"
        aria-modal="true"
        aria-label="Vision AI Timetable Scanner"
        tabindex="-1"
        onclick={(e) => {
          if (e.target === e.currentTarget && !isSavingExtracted) {
            handleCancelVisionModal();
          }
        }}
      >
        <div class="submodal-card ai-vision-card">
          <header class="submodal-header">
            <div style="display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap;">
              <Sparkles size={18} color="#007AFF" />
              <h3>Assistente Orario AI</h3>
              {#if themeStore.settings.aiProvider === "local"}
                <span class="badge" style="background: rgba(52, 199, 89, 0.15); color: #34C759; font-size: 0.72rem; padding: 0.15rem 0.55rem; border-radius: 999px; font-weight: 700;">
                  ● AI Locale ({themeStore.settings.aiLocalModelId})
                </span>
              {:else}
                <span class="badge" style="background: rgba(0, 122, 255, 0.15); color: #007AFF; font-size: 0.72rem; padding: 0.15rem 0.55rem; border-radius: 999px; font-weight: 700;">
                  Cloud ({themeStore.settings.aiProvider.toUpperCase()})
                </span>
              {/if}
            </div>
            <button
              type="button"
              class="btn-close-sub pressable"
              disabled={isSavingExtracted}
              onclick={handleCancelVisionModal}
              title="Chiudi e annulla"
            >
              <X size={15} />
            </button>
          </header>

          <div class="ai-vision-content">
            {#if extractedLessons.length > 0}
              <!-- REVIEW DELLE LEZIONI ESTRATTE (SIA DA FOTO CHE DA PROMPT/VOCE) -->
              <div class="extracted-table-container">
                <div class="extracted-header">
                  <div style="display: flex; align-items: center; gap: 0.65rem;">
                    <button
                      type="button"
                      class="btn-back-to-input pressable"
                      disabled={isSavingExtracted}
                      onclick={() => { extractedLessons = []; aiError = null; }}
                      title="Torna indietro per modificare l'immagine o il prompt"
                    >
                      <ArrowLeft size={13} />
                      <span>Modifica input</span>
                    </button>
                    <h4>Lezioni Riconosciute ({extractedLessons.length})</h4>
                  </div>
                  <span class="extracted-hint">Modifica i campi se necessario prima di confermare:</span>
                </div>

                <div class="extracted-rows-scroll">
                  {#each extractedLessons as item, idx}
                    <div class="extracted-item-row">
                      <select bind:value={item.dayOfWeek} class="input-select-mini">
                        {#each DAYS as d}
                          <option value={d.id}>{d.short}</option>
                        {/each}
                      </select>
                      <input
                        type="time"
                        bind:value={item.startTime}
                        class="input-time-mini"
                        title="Ora Inizio"
                      />
                      <span class="dash">–</span>
                      <input
                        type="time"
                        bind:value={item.endTime}
                        class="input-time-mini"
                        title="Ora Fine"
                      />
                      <input
                        type="text"
                        bind:value={item.courseName}
                        placeholder="Materia"
                        class="input-text-mini"
                        title="Materia"
                      />
                      <input
                        type="text"
                        bind:value={item.classroom}
                        placeholder="Aula"
                        class="input-text-mini room"
                        title="Aula"
                      />
                      <input
                        type="text"
                        bind:value={item.professor}
                        placeholder="Docente"
                        class="input-text-mini prof"
                        title="Docente"
                      />
                      <button
                        type="button"
                        class="btn-del-mini"
                        title="Rimuovi riga"
                        onclick={() => {
                          extractedLessons = extractedLessons.filter((_, i) => i !== idx);
                        }}
                      >
                        <Trash2 size={13} />
                      </button>
                    </div>
                  {/each}
                </div>

                <button
                  type="button"
                  class="btn-add-extracted-row pressable"
                  onclick={() => {
                    extractedLessons = [
                      ...extractedLessons,
                      {
                        dayOfWeek: 1,
                        startTime: "09:00",
                        endTime: "11:00",
                        courseName: "",
                        classroom: "",
                        professor: ""
                      }
                    ];
                  }}
                >
                  + Aggiungi riga manuale
                </button>
              </div>
            {:else}
              <!-- SELETTORE MODALITÀ (FOTO / SCREENSHOT vs PROMPT & VOCE) -->
              <div class="ai-tabs-container">
                <div class="ai-mode-tabs">
                  <button
                    type="button"
                    class="ai-tab-btn"
                    class:active={aiModalTab === "vision"}
                    disabled={isAiAnalyzing}
                    onclick={() => {
                      aiModalTab = "vision";
                      aiError = null;
                    }}
                  >
                    <Upload size={13} />
                    <span>Foto / Screenshot</span>
                  </button>
                  <button
                    type="button"
                    class="ai-tab-btn"
                    class:active={aiModalTab === "prompt"}
                    disabled={isAiAnalyzing}
                    onclick={() => {
                      aiModalTab = "prompt";
                      aiError = null;
                    }}
                  >
                    <svg width="13" height="13" viewBox="0 0 24 24" fill="none">
                      <rect x="9" y="3" width="6" height="11" rx="3" stroke="currentColor" stroke-width="1.8" />
                      <path d="M5 11a7 7 0 0 0 14 0M12 18v3" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
                    </svg>
                    <span>Prompt & Dettatura Vocale</span>
                  </button>
                </div>
              </div>

              {#if aiModalTab === "vision"}
                {#if !aiImagePreview}
                  <!-- Drag and drop zone -->
                  <div
                    class="dropzone-area"
                    ondrop={handleFileDrop}
                    ondragover={(e) => e.preventDefault()}
                    role="region"
                    aria-label="Trascina foto dell'orario"
                  >
                    <div class="drop-icon-wrap">
                      <Upload size={32} />
                    </div>
                    <h4>Trascina qui l'immagine del tuo orario</h4>
                    <p class="drop-hint">
                      Oppure premi <kbd>Ctrl+V</kbd> per incollare uno screenshot dagli appunti (es. catturato con Win+Shift+S)
                    </p>
                    <label class="btn-file-select pressable">
                      <input
                        type="file"
                        accept="image/png, image/jpeg, image/webp"
                        onchange={handleFileInputChange}
                        style="display: none;"
                      />
                      <span>Sfoglia immagine…</span>
                    </label>
                  </div>
                {:else}
                  <!-- Image preview & AI extraction result -->
                  <div class="vision-results-layout">
                    <div class="vision-preview-col">
                      <div class="thumb-wrapper">
                        <img src={aiImagePreview} alt="Orario caricato" class="vision-thumb" />
                      </div>
                      <button
                        type="button"
                        class="btn-change-image pressable"
                        disabled={isSavingExtracted || isAiAnalyzing}
                        onclick={() => {
                          currentAnalysisId++;
                          isAiAnalyzing = false;
                          aiImagePreview = null;
                          aiImageBase64 = null;
                          extractedLessons = [];
                          aiError = null;
                        }}
                      >
                        Carica un'altra foto
                      </button>
                    </div>

                    <div class="vision-extracted-col">
                      {#if isAiAnalyzing}
                        <div class="analyzing-state">
                          <div class="ai-spinner"></div>
                          <h4>L'Intelligenza Artificiale sta leggendo l'orario…</h4>
                          <p>Riconoscimento automatico di giorni, materie, aule e docenti in corso.</p>
                          <button
                            type="button"
                            class="btn-cancel-analyzing pressable"
                            onclick={handleCancelVisionModal}
                            style="margin-top: 0.85rem; background: rgba(255, 69, 58, 0.12); color: #FF453A; border: 1px solid rgba(255, 69, 58, 0.25); border-radius: 8px; padding: 0.4rem 0.9rem; font-size: 0.82rem; font-weight: 600; cursor: pointer;"
                          >
                            Interrompi analisi
                          </button>
                        </div>
                      {:else if aiError}
                        <div class="ai-error-box">
                          <p><strong>Errore analisi foto:</strong></p>
                          <p>{aiError}</p>
                          <button
                            type="button"
                            class="btn-retry pressable"
                            onclick={() => aiImageBase64 && runVisionAiAnalysis(aiImageBase64, aiImageMime)}
                          >
                            Riprova analisi
                          </button>
                        </div>
                      {/if}
                    </div>
                  </div>
                {/if}
              {:else}
                <!-- Prompt & Voice Area -->
                <div class="ai-prompt-area">
                  <div class="prompt-card">
                    <div class="prompt-header-row">
                      <span class="prompt-label">Descrivi a parole o detta a voce l'orario delle tue lezioni:</span>
                      {#if isListening}
                        <span class="listening-tag">
                          <span class="pulse-dot"></span> In ascolto (parla ora in italiano)…
                        </span>
                      {/if}
                    </div>

                    <textarea
                      class="prompt-textarea"
                      bind:value={promptText}
                      disabled={isAiAnalyzing}
                      placeholder="Es: 'Il mercoledì ho Progettazione dalle 08:15 alle 10:15 in aula BL.28.2.1 e Macchine dalle 10:15 alle 12:15 in BL.28.2.1. Il giovedì ho Macchine dalle 13:15 alle 15:15 in aula B8 0.4. Il venerdì ho Macchine 08:15-10:15 e Progettazione 10:15-12:15 in aula B8 1.1'&#10;&#10;Oppure incolla direttamente l'elenco testuale delle lezioni dal tuo portale Esse3/PoliMi."
                    ></textarea>

                    {#if voiceError}
                      <div class="voice-alert-box">
                        <span>{voiceError}</span>
                      </div>
                    {/if}

                    {#if aiError}
                      <div class="ai-error-box" style="margin: 0.5rem 0 0 0;">
                        <p><strong>Errore analisi prompt:</strong></p>
                        <p>{aiError}</p>
                      </div>
                    {/if}

                    {#if isAiAnalyzing}
                      <div class="analyzing-state" style="padding: 1.25rem 0;">
                        <div class="ai-spinner"></div>
                        <h4>Elaborazione orario dal testo in corso…</h4>
                        <p>Estrazione automatica dei corsi, giorni, aule e orari con Intelligenza Artificiale.</p>
                        <button
                          type="button"
                          class="btn-cancel-analyzing pressable"
                          onclick={handleCancelVisionModal}
                          style="margin-top: 0.85rem; background: rgba(255, 69, 58, 0.12); color: #FF453A; border: 1px solid rgba(255, 69, 58, 0.25); border-radius: 8px; padding: 0.4rem 0.9rem; font-size: 0.82rem; font-weight: 600; cursor: pointer;"
                        >
                          Interrompi analisi
                        </button>
                      </div>
                    {/if}

                    <div class="prompt-actions-bar">
                      <div class="prompt-voice-group">
                        <button
                          type="button"
                          class="mic-trigger-btn"
                          class:listening={isListening}
                          onclick={toggleVoiceDictation}
                          title={isListening ? "Interrompi ascolto vocale" : "Avvia dettatura vocale con microfono (it-IT)"}
                        >
                          <svg width="15" height="15" viewBox="0 0 24 24" fill="none">
                            <rect x="9" y="3" width="6" height="11" rx="3" stroke="currentColor" stroke-width="1.8" />
                            <path d="M5 11a7 7 0 0 0 14 0M12 18v3" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
                          </svg>
                        </button>

                        {#if isListening}
                          <div class="voice-waveform" aria-label="Onda sonora dettatura attiva">
                            <span></span>
                            <span></span>
                            <span></span>
                            <span></span>
                            <span></span>
                          </div>
                          <span class="listening-caption">Parla in italiano…</span>
                        {:else}
                          <span class="mic-hint-text">Premi il microfono per dettare a voce</span>
                        {/if}
                      </div>

                      <div style="display: flex; gap: 0.5rem; align-items: center;">
                        {#if promptText}
                          <button
                            type="button"
                            class="btn-clear-prompt pressable"
                            disabled={isAiAnalyzing}
                            onclick={() => { promptText = ""; aiError = null; }}
                          >
                            Cancella
                          </button>
                        {/if}
                        <button
                          type="button"
                          class="btn-analyze-prompt pressable"
                          disabled={isAiAnalyzing || !promptText.trim()}
                          onclick={handleAnalyzePrompt}
                        >
                          <Sparkles size={14} />
                          <span>{isAiAnalyzing ? "Analisi in corso…" : "Estrai Lezioni con IA"}</span>
                        </button>
                      </div>
                    </div>
                  </div>

                  <!-- Quick Chips Section -->
                  <div class="prompt-chips-section">
                    <span class="chips-title">FORMATI RAPIDI & SUGGERIMENTI:</span>
                    <div class="chips-list">
                      <button
                        type="button"
                        class="chip-btn pressable"
                        onclick={() => {
                          promptText = "Mercoledì dalle 08:15 alle 10:15 in aula BL.28.2.1 ho Progettazione di Sistemi Meccanici, e dalle 10:15 alle 12:15 ho Macchine e Sistemi Energetici in aula BL.28.2.1.\nGiovedì dalle 13:15 alle 15:15 ho Macchine e Sistemi Energetici in aula B8 0.4.\nVenerdì dalle 08:15 alle 10:15 ho Macchine e Sistemi Energetici in aula B8 1.1, e dalle 10:15 alle 12:15 ho Progettazione di Sistemi Meccanici in aula B8 1.1.";
                        }}
                      >
                        <span>📝 Inserisci esempio orario completo (5 lezioni)</span>
                      </button>
                    </div>
                  </div>
                </div>
              {/if}
            {/if}
          </div>

          <footer class="submodal-footer">
            <button
              type="button"
              class="btn-secondary pressable"
              disabled={isSavingExtracted}
              onclick={handleCancelVisionModal}
            >
              {isAiAnalyzing ? "Interrompi e Chiudi" : "Annulla"}
            </button>
            {#if extractedLessons.length > 0}
              <button
                type="button"
                class="btn-primary pressable"
                disabled={isSavingExtracted || isAiAnalyzing}
                onclick={handleConfirmExtractedLessons}
              >
                <Check size={14} />
                <span>
                  {isSavingExtracted ? "Salvataggio…" : `Salva ${extractedLessons.length} lezioni nell'orario`}
                </span>
              </button>
            {/if}
          </footer>
        </div>
      </div>
    {/if}

    <!-- Sub-modale Proietta nel Calendario -->
    {#if isProjectModalOpen}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <div
        class="submodal-scrim"
        role="dialog"
        aria-modal="true"
        aria-label="Proietta Lezioni a Calendario"
        tabindex="-1"
        onclick={(e) => {
          if (e.target === e.currentTarget && !isProjecting) isProjectModalOpen = false;
        }}
      >
        <div class="submodal-card">
          <header class="submodal-header">
            <h3>Proietta Orario nel Calendario</h3>
            <button
              type="button"
              class="btn-close-sub pressable"
              disabled={isProjecting}
              onclick={() => (isProjectModalOpen = false)}
            >
              <X size={15} />
            </button>
          </header>

          <div class="project-modal-body">
            <p class="project-info-text">
              Questa operazione genererà automaticamente tutti gli impegni di lezione
              settimanali nel calendario della sessione attiva (categoria <code>lezione</code>).
            </p>

            <div class="form-row-dual">
              <div class="form-col">
                <label for="proj-start">Data Inizio Proiezione</label>
                <input
                  id="proj-start"
                  type="date"
                  bind:value={projectStartDate}
                  class="input-text"
                  required
                />
              </div>
              <div class="form-col">
                <label for="proj-end">Data Fine Proiezione</label>
                <input
                  id="proj-end"
                  type="date"
                  bind:value={projectEndDate}
                  class="input-text"
                  required
                />
              </div>
            </div>

            <div class="project-summary-box">
              <span>Lezioni settimanali da proiettare: <strong>{lessons.length}</strong></span>
              <span class="project-note">Se esistono già lezioni uguali nelle date indicate, non verranno duplicate.</span>
            </div>
          </div>

          <footer class="submodal-footer">
            <button
              type="button"
              class="btn-secondary pressable"
              disabled={isProjecting}
              onclick={() => (isProjectModalOpen = false)}
            >
              Annulla
            </button>
            <button
              type="button"
              class="btn-primary pressable"
              disabled={isProjecting || lessons.length === 0}
              onclick={handleProjectToCalendar}
            >
              <ArrowRight size={14} />
              <span>{isProjecting ? "Proiezione in corso…" : "Conferma e Proietta"}</span>
            </button>
          </footer>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .modal-scrim {
    position: fixed;
    inset: 0;
    z-index: 1200;
    background: rgba(0, 0, 0, 0.45);
    backdrop-filter: blur(16px);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1.5rem;
    animation: scrim-in 0.22s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }

  .timetable-modal-sheet {
    width: 100%;
    max-width: 1320px;
    height: 88vh;
    max-height: 860px;
    background: var(--surface-bg, rgba(28, 28, 30, 0.96));
    border: 0.5px solid var(--border-subtle, rgba(255, 255, 255, 0.12));
    border-radius: 20px;
    box-shadow: 0 30px 60px rgba(0, 0, 0, 0.5), 0 0 0 0.5px rgba(255, 255, 255, 0.1);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: sheet-in 0.26s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }

  .sheet-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1.1rem 1.6rem;
    border-bottom: 0.5px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
    background: var(--surface-header, rgba(255, 255, 255, 0.02));
  }

  .sheet-eyebrow {
    font-size: 0.68rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    color: var(--text-tertiary, #8e8e93);
    text-transform: uppercase;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    margin-top: 0.15rem;
  }

  .title-row h2 {
    font-size: 1.35rem;
    font-weight: 700;
    margin: 0;
    color: var(--text-primary, #ffffff);
  }

  .session-pill {
    font-size: 0.72rem;
    font-weight: 600;
    padding: 0.2rem 0.55rem;
    border-radius: 9999px;
    background: rgba(0, 122, 255, 0.15);
    color: #007aff;
    border: 0.5px solid rgba(0, 122, 255, 0.3);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .btn-secondary {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.45rem 0.85rem;
    border-radius: 10px;
    background: var(--surface-card);
    border: 0.5px solid var(--hairline-border-strong);
    color: var(--text-primary);
    font-size: 0.82rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .btn-secondary:hover {
    background: var(--surface-pill-hover);
    color: var(--text-primary);
  }
  .btn-secondary.active {
    background: var(--accent-soft);
    border-color: var(--accent-primary);
    color: var(--accent-primary);
  }

  .btn-ai {
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
    padding: 0.45rem 0.95rem;
    border-radius: 10px;
    background: linear-gradient(135deg, rgba(124, 58, 237, 0.25), rgba(0, 122, 255, 0.25));
    border: 0.5px solid rgba(168, 85, 247, 0.4);
    color: #c084fc;
    font-size: 0.82rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.18s ease;
  }
  .btn-ai:hover {
    background: linear-gradient(135deg, rgba(124, 58, 237, 0.4), rgba(0, 122, 255, 0.4));
    color: #e9d5ff;
    box-shadow: 0 0 12px rgba(168, 85, 247, 0.35);
  }

  .btn-project {
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
    padding: 0.45rem 0.95rem;
    border-radius: 10px;
    background: rgba(5, 150, 105, 0.18);
    border: 0.5px solid rgba(5, 150, 105, 0.35);
    color: #34d399;
    font-size: 0.82rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .btn-project:hover:not(:disabled) {
    background: rgba(5, 150, 105, 0.3);
    color: #6ee7b7;
  }
  .btn-project:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .btn-primary {
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
    padding: 0.45rem 1rem;
    border-radius: 10px;
    background: #007aff;
    border: none;
    color: #ffffff;
    font-size: 0.82rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
    box-shadow: 0 2px 8px rgba(0, 122, 255, 0.3);
  }
  .btn-primary:hover {
    background: #0066d6;
  }

  .btn-close {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.08);
    border: none;
    color: var(--text-secondary, #cccccc);
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .btn-close:hover {
    background: rgba(255, 255, 255, 0.16);
    color: #ffffff;
  }

  /* Timetable Grid */
  .timetable-grid-container {
    flex: 1;
    overflow-x: auto;
    overflow-y: hidden;
    padding: 1.2rem 1.6rem 1.6rem 1.6rem;
    display: flex;
  }

  .timetable-grid {
    display: grid;
    gap: 1rem;
    width: 100%;
    height: 100%;
  }

  .day-column {
    display: flex;
    flex-direction: column;
    background: rgba(255, 255, 255, 0.025);
    border: 0.5px solid var(--border-subtle, rgba(255, 255, 255, 0.08));
    border-radius: 14px;
    overflow: hidden;
  }

  .day-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 0.95rem;
    background: rgba(255, 255, 255, 0.035);
    border-bottom: 0.5px solid rgba(255, 255, 255, 0.06);
  }

  .day-name {
    font-size: 0.88rem;
    font-weight: 700;
    color: var(--text-primary, #ffffff);
  }

  .day-count-badge {
    font-size: 0.68rem;
    font-weight: 700;
    padding: 0.12rem 0.45rem;
    border-radius: 9999px;
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-tertiary, #888888);
  }
  .day-count-badge.has-lessons {
    background: rgba(0, 122, 255, 0.2);
    color: #38bdf8;
  }

  .day-content {
    flex: 1;
    overflow-y: auto;
    padding: 0.8rem;
    display: flex;
    flex-direction: column;
  }

  .day-empty-placeholder {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.6rem;
    color: var(--text-tertiary, #666666);
    padding: 1.5rem 0;
  }

  .empty-text {
    font-size: 0.76rem;
  }

  .btn-add-day {
    font-size: 0.74rem;
    font-weight: 600;
    color: #007aff;
    background: rgba(0, 122, 255, 0.1);
    border: 0.5px solid rgba(0, 122, 255, 0.25);
    padding: 0.3rem 0.65rem;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .btn-add-day:hover {
    background: rgba(0, 122, 255, 0.2);
  }

  .lesson-cards-list {
    display: flex;
    flex-direction: column;
    gap: 0.7rem;
  }

  .lesson-card {
    background: var(--lesson-tint, rgba(255, 255, 255, 0.05));
    border: 0.5px solid var(--lesson-border, rgba(255, 255, 255, 0.12));
    border-left: 3.5px solid var(--lesson-solid, #007aff);
    border-radius: 10px;
    padding: 0.65rem 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    transition: transform 0.15s ease, box-shadow 0.15s ease;
  }
  .lesson-card:hover {
    transform: translateY(-1.5px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.25);
  }

  .card-time-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .time-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.72rem;
    font-weight: 700;
    color: var(--lesson-text, #ffffff);
    opacity: 0.9;
  }

  .card-actions {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    opacity: 0.7;
    transition: opacity 0.15s ease;
  }
  .lesson-card:hover .card-actions {
    opacity: 1;
  }

  .card-act-btn {
    background: transparent;
    border: none;
    color: var(--text-tertiary, #aaaaaa);
    padding: 0.2rem;
    border-radius: 4px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.12s ease;
  }
  .card-act-btn.edit:hover {
    color: #38bdf8;
    background: rgba(56, 189, 248, 0.15);
  }
  .card-act-btn.delete:hover {
    color: #f87171;
    background: rgba(248, 113, 113, 0.15);
  }

  .card-subject-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .subject-title {
    font-size: 0.85rem;
    font-weight: 700;
    color: var(--text-primary, #ffffff);
    line-height: 1.25;
  }

  .card-meta-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.6rem;
    font-size: 0.7rem;
    color: var(--text-secondary, #bbbbbb);
    margin-top: 0.1rem;
  }

  .meta-item {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
  }

  .btn-add-more {
    margin-top: 0.4rem;
    padding: 0.35rem 0.6rem;
    background: transparent;
    border: 0.5px dashed rgba(255, 255, 255, 0.15);
    border-radius: 8px;
    color: var(--text-tertiary, #888888);
    font-size: 0.72rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .btn-add-more:hover {
    border-color: rgba(0, 122, 255, 0.5);
    color: #007aff;
    background: rgba(0, 122, 255, 0.05);
  }

  /* Submodals */
  .submodal-scrim {
    position: fixed;
    inset: 0;
    z-index: 1300;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(12px);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1.5rem;
    animation: scrim-in 0.18s ease forwards;
  }

  .submodal-card {
    width: 100%;
    max-width: 520px;
    background: var(--surface-bg, rgba(32, 32, 34, 0.98));
    border: 0.5px solid var(--border-subtle, rgba(255, 255, 255, 0.15));
    border-radius: 16px;
    box-shadow: 0 24px 48px rgba(0, 0, 0, 0.6);
    overflow: visible;
    animation: sheet-in 0.2s cubic-bezier(0.16, 1, 0.3, 1) forwards;
  }

  .submodal-card.ai-vision-card {
    max-width: 900px;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .submodal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.35rem;
    border-bottom: 0.5px solid rgba(255, 255, 255, 0.08);
  }

  .submodal-header h3 {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 700;
    color: var(--text-primary, #ffffff);
  }

  .btn-close-sub {
    background: transparent;
    border: none;
    color: var(--text-tertiary, #888888);
    cursor: pointer;
    padding: 0.25rem;
    border-radius: 4px;
    display: flex;
  }
  .btn-close-sub:hover {
    color: #ffffff;
  }

  .submodal-form {
    padding: 1.35rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .form-row {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .form-col label,
  .field-label {
    font-size: 0.74rem;
    font-weight: 600;
    color: var(--text-secondary, #cccccc);
  }

  .course-select-wrap {
    margin-bottom: 0.45rem;
  }

  .form-row-dual {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.85rem;
  }

  .form-col {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .input-text {
    width: 100%;
    padding: 0.55rem 0.75rem;
    background: var(--surface-card);
    border: 0.5px solid var(--hairline-border-strong);
    border-radius: var(--radius-sm);
    color: var(--text-primary);
    font-size: 0.84rem;
    outline: none;
    transition: border-color 0.15s ease, background-color 0.15s ease;
  }
  .input-text:focus {
    border-color: var(--accent-primary);
    background: var(--surface-elevated);
  }

  .label-palette {
    font-size: 0.74rem;
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: 0.4rem;
    display: block;
  }

  .palette-swatches {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .swatch-btn {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
    transition: transform 0.12s ease;
  }
  .swatch-btn:hover {
    transform: scale(1.15);
  }
  .swatch-btn.selected {
    border-color: #ffffff;
    box-shadow: 0 0 8px rgba(255, 255, 255, 0.5);
  }

  .submodal-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 0.65rem;
    padding: 0.9rem 1.35rem;
    border-top: 0.5px solid rgba(255, 255, 255, 0.08);
    background: rgba(255, 255, 255, 0.02);
  }

  /* Vision AI Dropzone & Review Layout */
  .ai-vision-content {
    flex: 1;
    overflow-y: auto;
    padding: 1.35rem;
    display: flex;
    flex-direction: column;
  }

  .dropzone-area {
    border: 2px dashed rgba(168, 85, 247, 0.4);
    border-radius: 14px;
    padding: 2.5rem 1.5rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    gap: 0.8rem;
    background: rgba(168, 85, 247, 0.04);
    transition: all 0.2s ease;
  }
  .dropzone-area:hover {
    border-color: rgba(168, 85, 247, 0.7);
    background: rgba(168, 85, 247, 0.08);
  }

  .drop-icon-wrap {
    color: #c084fc;
  }

  .dropzone-area h4 {
    margin: 0;
    font-size: 1.05rem;
    font-weight: 700;
    color: #ffffff;
  }

  .drop-hint {
    font-size: 0.8rem;
    color: var(--text-secondary, #aaaaaa);
    max-width: 420px;
    margin: 0;
  }
  .drop-hint kbd {
    background: rgba(255, 255, 255, 0.15);
    padding: 0.15rem 0.45rem;
    border-radius: 4px;
    font-size: 0.75rem;
    color: #ffffff;
  }

  .btn-file-select {
    margin-top: 0.5rem;
    padding: 0.55rem 1.25rem;
    border-radius: 10px;
    background: #007aff;
    color: #ffffff;
    font-size: 0.84rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s ease;
  }
  .btn-file-select:hover {
    background: #0066d6;
  }

  .vision-results-layout {
    display: grid;
    grid-template-columns: 240px 1fr;
    gap: 1.25rem;
  }

  .vision-preview-col {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
  }

  .thumb-wrapper {
    width: 100%;
    height: 260px;
    background: #111111;
    border: 0.5px solid rgba(255, 255, 255, 0.1);
    border-radius: 10px;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .vision-thumb {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

  .btn-change-image {
    padding: 0.4rem;
    background: rgba(255, 255, 255, 0.08);
    border: 0.5px solid rgba(255, 255, 255, 0.12);
    border-radius: 8px;
    color: var(--text-secondary, #cccccc);
    font-size: 0.74rem;
    font-weight: 600;
    cursor: pointer;
  }

  .vision-extracted-col {
    display: flex;
    flex-direction: column;
  }

  .analyzing-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: 3rem 1.5rem;
    gap: 0.8rem;
  }

  .ai-spinner {
    width: 36px;
    height: 36px;
    border: 3px solid rgba(168, 85, 247, 0.2);
    border-top-color: #c084fc;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .ai-error-box {
    background: rgba(239, 68, 68, 0.12);
    border: 0.5px solid rgba(239, 68, 68, 0.3);
    border-radius: 10px;
    padding: 1rem;
    color: #fca5a5;
    font-size: 0.84rem;
  }

  .btn-retry {
    margin-top: 0.6rem;
    padding: 0.35rem 0.75rem;
    border-radius: 6px;
    background: #ef4444;
    border: none;
    color: #ffffff;
    font-size: 0.78rem;
    font-weight: 600;
    cursor: pointer;
  }

  .extracted-table-container {
    display: flex;
    flex-direction: column;
    gap: 0.7rem;
  }

  .extracted-header h4 {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 700;
    color: #ffffff;
  }
  .extracted-hint {
    font-size: 0.72rem;
    color: var(--text-tertiary, #888888);
  }

  .extracted-rows-scroll {
    max-height: 380px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    padding-right: 0.4rem;
  }

  .extracted-item-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    background: var(--surface-card);
    border: 0.5px solid var(--hairline-border);
    border-radius: var(--radius-sm);
    padding: 0.4rem 0.55rem;
  }

  .input-select-mini {
    width: 68px;
    padding: 0.25rem 0.45rem;
    background: var(--surface-card);
    border: 0.5px solid var(--hairline-border-strong);
    border-radius: var(--radius-xs);
    color: var(--text-primary);
    font-size: 0.74rem;
    font-weight: 600;
    cursor: pointer;
    outline: none;
    transition: border-color 0.15s ease, background-color 0.15s ease;
  }

  .input-select-mini:focus {
    border-color: var(--accent-primary);
    background: var(--surface-elevated);
  }

  .input-time-mini {
    width: 68px;
    padding: 0.25rem 0.35rem;
    background: var(--surface-card);
    border: 0.5px solid var(--hairline-border-strong);
    border-radius: var(--radius-xs);
    color: var(--text-primary);
    font-size: 0.74rem;
  }

  .dash {
    color: var(--text-tertiary, #666);
    font-size: 0.75rem;
  }

  .input-text-mini {
    flex: 2;
    padding: 0.25rem 0.45rem;
    background: rgba(255, 255, 255, 0.08);
    border: 0.5px solid rgba(255, 255, 255, 0.12);
    border-radius: 6px;
    color: #ffffff;
    font-size: 0.74rem;
  }
  .input-text-mini.room {
    flex: 1;
  }
  .input-text-mini.prof {
    flex: 1.2;
  }

  .btn-del-mini {
    background: transparent;
    border: none;
    color: var(--text-tertiary, #888888);
    cursor: pointer;
    padding: 0.2rem;
    border-radius: 4px;
    display: flex;
  }
  .btn-del-mini:hover {
    color: #f87171;
  }

  .btn-add-extracted-row {
    align-self: flex-start;
    padding: 0.3rem 0.65rem;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.06);
    border: 0.5px dashed rgba(255, 255, 255, 0.15);
    color: var(--text-secondary, #cccccc);
    font-size: 0.72rem;
    font-weight: 600;
    cursor: pointer;
  }

  /* Proiezione Calendario Body */
  .project-modal-body {
    padding: 1.35rem;
    display: flex;
    flex-direction: column;
    gap: 1.1rem;
  }

  .project-info-text {
    font-size: 0.84rem;
    color: var(--text-secondary, #cccccc);
    margin: 0;
    line-height: 1.45;
  }
  .project-info-text code {
    background: rgba(5, 150, 105, 0.2);
    color: #34d399;
    padding: 0.1rem 0.35rem;
    border-radius: 4px;
  }

  .project-summary-box {
    background: rgba(255, 255, 255, 0.035);
    border: 0.5px solid rgba(255, 255, 255, 0.08);
    border-radius: 10px;
    padding: 0.9rem 1.1rem;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    font-size: 0.82rem;
    color: var(--text-primary, #ffffff);
  }

  .project-note {
    font-size: 0.72rem;
    color: var(--text-tertiary, #888888);
  }

  /* Assistente AI Tabs & Prompt Area */
  .ai-tabs-container {
    display: flex;
    justify-content: center;
    margin-bottom: 1.15rem;
  }

  .ai-mode-tabs {
    display: inline-flex;
    padding: 3px;
    background: rgba(255, 255, 255, 0.06);
    border: 0.5px solid rgba(255, 255, 255, 0.1);
    border-radius: 999px;
    gap: 4px;
  }

  .ai-tab-btn {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    padding: 0.35rem 0.95rem;
    border-radius: 999px;
    border: none;
    background: transparent;
    color: var(--text-secondary, #aaaaaa);
    font-size: 0.78rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .ai-tab-btn:hover {
    color: #ffffff;
  }
  .ai-tab-btn.active {
    background: var(--surface-card, #2c2c2e);
    color: #ffffff;
    font-weight: 600;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.25);
  }

  .btn-back-to-input {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.25rem 0.65rem;
    background: rgba(255, 255, 255, 0.08);
    border: 0.5px solid rgba(255, 255, 255, 0.12);
    border-radius: 6px;
    color: var(--text-secondary, #cccccc);
    font-size: 0.74rem;
    font-weight: 600;
    cursor: pointer;
  }
  .btn-back-to-input:hover {
    color: #ffffff;
    background: rgba(255, 255, 255, 0.14);
  }

  .ai-prompt-area {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    width: 100%;
  }

  .prompt-card {
    background: rgba(255, 255, 255, 0.035);
    border: 0.5px solid rgba(255, 255, 255, 0.08);
    border-radius: 12px;
    padding: 1.15rem;
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
  }

  .prompt-header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .prompt-label {
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--text-primary, #ffffff);
  }

  .listening-tag {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    background: rgba(239, 68, 68, 0.15);
    border: 0.5px solid rgba(239, 68, 68, 0.3);
    color: #ef4444;
    padding: 0.2rem 0.6rem;
    border-radius: 999px;
    font-size: 0.72rem;
    font-weight: 600;
  }

  .pulse-dot {
    width: 7px;
    height: 7px;
    background: #ef4444;
    border-radius: 50%;
    animation: pulse 1s infinite alternate;
  }

  @keyframes pulse {
    0% { transform: scale(0.8); opacity: 0.5; }
    100% { transform: scale(1.3); opacity: 1; }
  }

  .prompt-textarea {
    width: 100%;
    min-height: 135px;
    background: rgba(0, 0, 0, 0.25);
    border: 0.5px solid rgba(255, 255, 255, 0.12);
    border-radius: 10px;
    padding: 0.85rem 1rem;
    color: #ffffff;
    font-family: inherit;
    font-size: 0.85rem;
    line-height: 1.5;
    resize: vertical;
    outline: none;
    transition: border-color 0.15s ease;
  }
  .prompt-textarea:focus {
    border-color: #007aff;
    background: rgba(0, 0, 0, 0.35);
  }

  .voice-alert-box {
    background: rgba(255, 149, 0, 0.12);
    border: 0.5px solid rgba(255, 149, 0, 0.25);
    color: #ff9500;
    padding: 0.45rem 0.75rem;
    border-radius: 8px;
    font-size: 0.76rem;
  }

  .prompt-actions-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    flex-wrap: wrap;
    padding-top: 0.3rem;
  }

  .prompt-voice-group {
    display: flex;
    align-items: center;
    gap: 0.65rem;
  }

  .mic-trigger-btn {
    width: 34px;
    height: 34px;
    border-radius: 50%;
    border: 0.5px solid rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-secondary, #cccccc);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .mic-trigger-btn:hover {
    color: #ffffff;
    background: rgba(255, 255, 255, 0.15);
  }
  .mic-trigger-btn.listening {
    background: rgba(239, 68, 68, 0.2);
    border-color: rgba(239, 68, 68, 0.5);
    color: #ef4444;
    box-shadow: 0 0 10px rgba(239, 68, 68, 0.4);
  }

  .voice-waveform {
    display: flex;
    align-items: center;
    gap: 2.5px;
    height: 18px;
  }
  .voice-waveform span {
    width: 3px;
    height: 100%;
    background: #ef4444;
    border-radius: 2px;
    animation: soundWave 0.6s infinite ease-in-out alternate;
  }
  .voice-waveform span:nth-child(1) { animation-delay: 0s; }
  .voice-waveform span:nth-child(2) { animation-delay: 0.12s; }
  .voice-waveform span:nth-child(3) { animation-delay: 0.24s; }
  .voice-waveform span:nth-child(4) { animation-delay: 0.36s; }
  .voice-waveform span:nth-child(5) { animation-delay: 0.48s; }

  @keyframes soundWave {
    0% { transform: scaleY(0.25); }
    100% { transform: scaleY(1); }
  }

  .listening-caption {
    font-size: 0.74rem;
    font-weight: 600;
    color: #ef4444;
  }

  .mic-hint-text {
    font-size: 0.74rem;
    color: var(--text-tertiary, #888888);
  }

  .btn-clear-prompt {
    padding: 0.45rem 0.85rem;
    background: rgba(255, 255, 255, 0.06);
    border: 0.5px solid rgba(255, 255, 255, 0.12);
    border-radius: 8px;
    color: var(--text-secondary, #cccccc);
    font-size: 0.78rem;
    font-weight: 500;
    cursor: pointer;
  }
  .btn-clear-prompt:hover {
    color: #ffffff;
    background: rgba(255, 255, 255, 0.1);
  }

  .btn-analyze-prompt {
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
    padding: 0.45rem 1.15rem;
    background: #007aff;
    border: none;
    border-radius: 8px;
    color: #ffffff;
    font-size: 0.82rem;
    font-weight: 600;
    cursor: pointer;
    box-shadow: 0 1px 4px rgba(0, 122, 255, 0.3);
    transition: background 0.15s ease;
  }
  .btn-analyze-prompt:hover:not(:disabled) {
    background: #0066d6;
  }
  .btn-analyze-prompt:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .prompt-chips-section {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    margin-top: 0.2rem;
  }

  .chips-title {
    font-size: 0.68rem;
    font-weight: 700;
    color: var(--text-tertiary, #888888);
    letter-spacing: 0.5px;
  }

  .chips-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .chip-btn {
    padding: 0.35rem 0.75rem;
    background: rgba(255, 255, 255, 0.04);
    border: 0.5px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    color: var(--text-secondary, #aaaaaa);
    font-size: 0.74rem;
    cursor: pointer;
    text-align: left;
    transition: all 0.15s ease;
  }
  .chip-btn:hover {
    color: #ffffff;
    background: rgba(255, 255, 255, 0.08);
    border-color: rgba(0, 122, 255, 0.4);
  }

  @keyframes scrim-in {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  @keyframes sheet-in {
    from {
      transform: scale(0.97) translateY(12px);
      opacity: 0;
    }
    to {
      transform: scale(1) translateY(0);
      opacity: 1;
    }
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>