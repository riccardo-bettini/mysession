<script lang="ts">
  import { Check, Eye, X } from "lucide-svelte";
  import { navStore } from "../../stores/navigation.svelte";
  import { sessionStore } from "../../stores/session.svelte";
  import { themeStore } from "../../stores/theme.svelte";
  import { getCategoryColor, getCategoryLabel } from "../../utils/categories";

  let snap = $derived(sessionStore.activeSnapshot);
  let quiz = $derived(snap?.quizzes.find((q) => q.id === navStore.activeQuizId) ?? null);

  let currentIdx = $state(0);
  let selectedOption = $state<number | null>(null);
  let showExplanation = $state(false);
  let results = $state<Record<string, { correct: boolean; choice: number | null }>>({});
  let isFinished = $state(false);

  $effect(() => {
    if (navStore.activeQuizId) {
      currentIdx = 0;
      selectedOption = null;
      showExplanation = false;
      results = {};
      isFinished = false;
    }
  });

  let currentQuestion = $derived(quiz?.questions[currentIdx] ?? null);

  function answerMcq(optIdx: number) {
    if (!currentQuestion || showExplanation) return;
    selectedOption = optIdx;
    showExplanation = true;
    const isCorrect = optIdx === (currentQuestion.correctIndex ?? 0);
    results = { ...results, [currentQuestion.id]: { correct: isCorrect, choice: optIdx } };
  }

  function answerFlashcard(isCorrect: boolean) {
    if (!currentQuestion) return;
    results = { ...results, [currentQuestion.id]: { correct: isCorrect, choice: null } };
    nextStep();
  }

  async function nextStep() {
    if (!quiz) return;
    if (currentIdx + 1 < quiz.questions.length) {
      currentIdx += 1;
      selectedOption = null;
      showExplanation = false;
    } else {
      await finishQuiz();
    }
  }

  async function finishQuiz() {
    if (!quiz || !snap) return;
    const total = quiz.questions.length;
    const correct = quiz.questions.filter((q) => results[q.id]?.correct).length;
    const wrongIds = quiz.questions.filter((q) => !results[q.id]?.correct).map((q) => q.id);
    const scorePct = total > 0 ? Math.round((correct / total) * 100) : 0;

    isFinished = true;
    await sessionStore.submitQuizAttempt({
      quizId: quiz.id,
      sessionId: snap.session.id,
      courseId: quiz.courseId,
      scorePct,
      correctCount: correct,
      totalCount: total,
      answersJson: JSON.stringify(results),
      wrongQuestionIds: wrongIds
    });
  }

  function closeModal() {
    navStore.activeQuizId = null;
  }
</script>

{#if quiz}
  <div class="modal-backdrop" role="dialog" aria-modal="true" aria-label={quiz.title}>
    <div class="bento-card player-card bento-enter">
      <header class="player-head">
        <div>
          <span class="eyebrow">{quiz.title}</span>
          {#if !isFinished && currentQuestion}
            <div class="q-meta">
              <span>Domanda {currentIdx + 1} di {quiz.questions.length}</span>
              <span class="cat-badge" style:background={getCategoryColor(currentQuestion.category, themeStore.settings)}>
                {getCategoryLabel(currentQuestion.category)}
              </span>
              {#if currentQuestion.chapter}
                <span class="ref-chip">{currentQuestion.chapter}</span>
              {/if}
            </div>
          {/if}
        </div>
        <button type="button" class="close-btn" onclick={closeModal} aria-label="Chiudi quiz">
          <X size={18} />
        </button>
      </header>

      {#if !isFinished && currentQuestion}
        <div class="prog-bar">
          <div class="prog-fill" style:width="{Math.round(((currentIdx + 1) / quiz.questions.length) * 100)}%"></div>
        </div>

        <div class="question-body">
          {#if currentQuestion.topic}
            <span class="topic-lbl">{currentQuestion.topic}</span>
          {/if}
          <h2>{currentQuestion.prompt}</h2>

          {#if currentQuestion.questionType === "multiple_choice"}
            <div class="options-stack">
              {#each currentQuestion.options as opt, idx (idx)}
                {@const isCorrectOpt = idx === currentQuestion.correctIndex}
                {@const isChosen = selectedOption === idx}
                <button
                  type="button"
                  class="opt-btn"
                  class:correct={showExplanation && isCorrectOpt}
                  class:wrong={showExplanation && isChosen && !isCorrectOpt}
                  disabled={showExplanation}
                  onclick={() => answerMcq(idx)}
                >
                  <span class="opt-letter">{String.fromCharCode(65 + idx)}</span>
                  <span class="opt-text">{opt}</span>
                </button>
              {/each}
            </div>

            {#if showExplanation}
              <div class="explanation-box">
                <strong>Spiegazione:</strong>
                <p>{currentQuestion.explanation}</p>
              </div>
              <div class="player-foot">
                <button type="button" class="pill-btn pill-btn-primary" onclick={nextStep}>
                  {currentIdx + 1 < quiz.questions.length ? "Prossima domanda" : "Concludi Quiz"}
                </button>
              </div>
            {/if}
          {:else}
            <!-- Flashcard aperta / Orale / Dimostrazione -->
            {#if !showExplanation}
              <div class="flash-prompt">
                <p>Prova a esporre o scrivere i passaggi chiave, poi rivela la soluzione.</p>
                <button type="button" class="pill-btn pill-btn-primary" onclick={() => (showExplanation = true)}>
                  <Eye size={15} />
                  Mostra soluzione
                </button>
              </div>
            {:else}
              <div class="explanation-box">
                <strong>Soluzione / Traccia completa:</strong>
                <p>{currentQuestion.explanation}</p>
              </div>
              <div class="player-foot">
                <button type="button" class="pill-btn pill-btn-soft" onclick={() => answerFlashcard(false)}>
                  Da rivedere
                </button>
                <button type="button" class="pill-btn pill-btn-primary" onclick={() => answerFlashcard(true)}>
                  <Check size={15} />
                  Risposto correttamente
                </button>
              </div>
            {/if}
          {/if}
        </div>
      {:else}
        {@const total = quiz.questions.length}
        {@const correct = quiz.questions.filter((q) => results[q.id]?.correct).length}
        {@const pct = total > 0 ? Math.round((correct / total) * 100) : 0}
        <div class="summary-view">
          <div class="score-circle">{pct}%</div>
          <h2>Quiz completato!</h2>
          <p>Hai risposto correttamente a <strong>{correct} su {total}</strong> domande.</p>
          <div class="player-foot">
            <button type="button" class="pill-btn pill-btn-soft" onclick={closeModal}>Chiudi</button>
            <button
              type="button"
              class="pill-btn pill-btn-primary"
              onclick={() => {
                currentIdx = 0;
                selectedOption = null;
                showExplanation = false;
                results = {};
                isFinished = false;
              }}
            >
              Ripeti Quiz
            </button>
          </div>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 150;
    background: rgba(12, 12, 18, 0.55);
    backdrop-filter: blur(8px);
    display: grid;
    place-items: center;
    padding: 20px;
  }

  .player-card {
    width: min(680px, 100%);
    max-height: 88vh;
    overflow-y: auto;
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .player-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
  }

  .eyebrow {
    font-size: 12.5px;
    font-weight: 700;
    color: var(--accent);
  }

  .q-meta {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 4px;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .cat-badge {
    padding: 2px 8px;
    border-radius: 999px;
    color: #fff;
    font-size: 10.5px;
    font-weight: 700;
  }

  .ref-chip {
    padding: 2px 8px;
    border-radius: 999px;
    background: var(--bg-surface-muted);
    font-weight: 600;
  }

  .close-btn {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: var(--bg-surface-muted);
    display: grid;
    place-items: center;
  }

  .prog-bar {
    height: 6px;
    border-radius: 999px;
    background: var(--bg-surface-muted);
    overflow: hidden;
  }

  .prog-fill {
    height: 100%;
    background: var(--accent);
    transition: width 0.2s ease;
  }

  .question-body {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .topic-lbl {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .question-body h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 700;
    line-height: 1.4;
  }

  .options-stack {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .opt-btn {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    border-radius: 14px;
    background: var(--bg-surface-muted);
    border: 1.5px solid transparent;
    text-align: left;
    font-size: 13.5px;
    transition: border-color 0.15s ease, background-color 0.15s ease;
  }

  .opt-btn:hover:not(:disabled) {
    border-color: var(--accent);
  }

  .opt-btn.correct {
    border-color: var(--success);
    background: rgba(16, 185, 129, 0.14);
  }

  .opt-btn.wrong {
    border-color: var(--danger);
    background: rgba(239, 68, 68, 0.14);
  }

  .opt-letter {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    background: var(--bg-surface);
    display: grid;
    place-items: center;
    font-size: 12px;
    font-weight: 800;
    flex-shrink: 0;
  }

  .explanation-box {
    padding: 14px;
    border-radius: 14px;
    background: var(--accent-soft);
    font-size: 13px;
  }

  .explanation-box p {
    margin: 6px 0 0;
  }

  .flash-prompt {
    padding: 20px;
    border-radius: 16px;
    background: var(--bg-surface-muted);
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .flash-prompt p {
    margin: 0;
    font-size: 13px;
    color: var(--text-secondary);
  }

  .player-foot {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
  }

  .summary-view {
    padding: 24px 10px;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 12px;
  }

  .score-circle {
    width: 96px;
    height: 96px;
    border-radius: 50%;
    background: var(--hero-gradient);
    display: grid;
    place-items: center;
    font-size: 26px;
    font-weight: 800;
  }

  .summary-view h2,
  .summary-view p {
    margin: 0;
  }
</style>
