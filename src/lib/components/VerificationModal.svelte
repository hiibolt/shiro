<script lang="ts">
  import { api, type Node, type VerificationQuestion, type VerificationResult } from "$lib/api";
  import { store } from "$lib/store.svelte";
  import { fade, scale, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";

  interface Props {
    node: Node;
    onClose: () => void;
  }
  let { node, onClose }: Props = $props();

  type Stage = "intro" | "loading-question" | "answering" | "grading" | "result";
  let stage = $state<Stage>("intro");
  let question = $state<VerificationQuestion | null>(null);
  let answer = $state("");
  let result = $state<VerificationResult | null>(null);
  let error = $state<string | null>(null);

  function autofocusEl(el: HTMLElement) {
    el.focus();
  }

  async function skip() {
    error = null;
    try {
      await api.updateNodeStatus(node.id, {
        type: "Mastered",
        verified_at: new Date().toISOString(),
      });
      await store.refresh();
      onClose();
    } catch (e) {
      error = String(e);
    }
  }

  async function startAI() {
    stage = "loading-question";
    error = null;
    try {
      question = await api.requestVerification(node.id);
      stage = "answering";
    } catch (e) {
      error = String(e);
      stage = "intro";
    }
  }

  async function submit() {
    if (!question || !answer.trim()) return;
    stage = "grading";
    error = null;
    try {
      result = await api.submitAnswer(node.id, question, answer);
      await store.refresh();
      stage = "result";
    } catch (e) {
      error = String(e);
      stage = "answering";
    }
  }
</script>

<div class="backdrop" transition:fade={{ duration: 160 }} onclick={onClose} role="presentation">
  <div
    class="modal"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.key === "Escape" && onClose()}
    role="dialog"
    aria-modal="true"
    aria-label="Verify understanding"
    tabindex="-1"
    transition:scale={{ start: 0.94, duration: 220, easing: cubicOut }}
  >
    <header>
      <div class="head-icon">✓</div>
      <div class="head-text">
        <div class="head-eyebrow">Verify understanding</div>
        <h2>{node.title}</h2>
      </div>
      <button class="close" onclick={onClose} aria-label="Close">×</button>
    </header>

    {#if stage === "intro"}
      <div class="body" in:fly={{ y: 8, duration: 200, easing: cubicOut }}>
        <p class="desc">{node.description}</p>
        <div class="choices">
          <button class="choice confident" onclick={skip}>
            <div class="choice-icon">⚡</div>
            <div>
              <div class="choice-title">I'm confident</div>
              <div class="choice-desc">Mark as mastered without a check.</div>
            </div>
          </button>
          <button class="choice ai" onclick={startAI}>
            <div class="choice-icon">✦</div>
            <div>
              <div class="choice-title">Verify with AI</div>
              <div class="choice-desc">Answer a short question — AI grades it.</div>
            </div>
          </button>
        </div>
      </div>
    {:else if stage === "loading-question"}
      <div class="body loading-body" in:fade={{ duration: 200 }}>
        <div class="spinner"></div>
        <p>Preparing a question…</p>
      </div>
    {:else if stage === "answering" && question}
      <div class="body" in:fly={{ y: 8, duration: 200, easing: cubicOut }}>
        <div class="question">
          <div class="question-eyebrow">{question.expects}</div>
          <p class="question-prompt">{question.prompt}</p>
        </div>
        <textarea
          rows="6"
          placeholder="Your answer…"
          bind:value={answer}
          use:autofocusEl
        ></textarea>
        <footer>
          <button onclick={onClose}>Cancel</button>
          <button class="primary" onclick={submit} disabled={!answer.trim()}>
            Submit answer
          </button>
        </footer>
      </div>
    {:else if stage === "grading"}
      <div class="body loading-body" in:fade={{ duration: 200 }}>
        <div class="spinner"></div>
        <p>Grading your answer…</p>
      </div>
    {:else if stage === "result" && result}
      <div class="body" in:fly={{ y: 8, duration: 240, easing: cubicOut }}>
        <div class="result" class:passed={result.passed}>
          <div class="result-badge">{result.passed ? "PASSED" : "TRY AGAIN"}</div>
          <p class="result-feedback">{result.feedback}</p>
        </div>
        <footer>
          <button class="primary" onclick={onClose}>Done</button>
        </footer>
      </div>
    {/if}

    {#if error}<p class="error">{error}</p>{/if}
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(6px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
  }
  .modal {
    width: 520px;
    max-width: calc(100vw - 40px);
    max-height: calc(100vh - 40px);
    background: linear-gradient(180deg, #171a24 0%, #12141a 100%);
    border: 1px solid #2a2f3c;
    border-radius: 16px;
    box-shadow: 0 24px 80px rgba(0, 0, 0, 0.75);
    padding: 20px 22px 18px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    overflow: hidden;
  }
  header {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .head-icon {
    width: 36px;
    height: 36px;
    border-radius: 10px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, #3a5da8, #5ecb85);
    color: white;
    font-size: 1.1rem;
    font-weight: 600;
    box-shadow: 0 4px 14px rgba(58, 93, 168, 0.4);
  }
  .head-text { flex: 1; overflow: hidden; }
  .head-eyebrow {
    font-size: 0.7rem;
    color: #6b7180;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    font-weight: 600;
  }
  h2 {
    margin: 2px 0 0;
    font-size: 1.1rem;
    line-height: 1.3;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .close {
    background: transparent;
    border: none;
    color: #6b7180;
    font-size: 1.4rem;
    cursor: pointer;
    padding: 0 4px;
    line-height: 1;
  }
  .close:hover { color: #e7e9ee; }

  .body { display: flex; flex-direction: column; gap: 14px; }
  .desc { margin: 0; font-size: 0.85rem; color: #c8ccd6; line-height: 1.5; }

  .choices { display: flex; flex-direction: column; gap: 8px; }
  .choice {
    display: flex;
    gap: 12px;
    align-items: center;
    padding: 12px 14px;
    background: #1c2030;
    border: 1px solid #2a2f3c;
    border-radius: 10px;
    color: inherit;
    text-align: left;
    cursor: pointer;
    font: inherit;
    transition: transform 140ms, background 140ms, border-color 140ms, box-shadow 200ms;
  }
  .choice:hover { transform: translateY(-1px); background: #232837; }
  .choice.confident:hover {
    border-color: #e5b25c;
    box-shadow: 0 6px 20px rgba(229, 178, 92, 0.15);
  }
  .choice.ai:hover {
    border-color: #8ab4ff;
    box-shadow: 0 6px 20px rgba(138, 180, 255, 0.18);
  }
  .choice-icon {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #0f1219;
    border-radius: 8px;
    font-size: 1.1rem;
    flex-shrink: 0;
  }
  .confident .choice-icon { color: #e5b25c; }
  .ai .choice-icon { color: #8ab4ff; }
  .choice-title { font-size: 0.9rem; font-weight: 600; }
  .choice-desc { font-size: 0.75rem; color: #9aa0ae; margin-top: 2px; }

  .question {
    padding: 12px 14px;
    background: rgba(138, 180, 255, 0.06);
    border: 1px solid rgba(138, 180, 255, 0.25);
    border-radius: 10px;
  }
  .question-eyebrow {
    font-size: 0.68rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: #8ab4ff;
    font-weight: 600;
    margin-bottom: 4px;
  }
  .question-prompt { margin: 0; font-size: 0.9rem; color: #e7e9ee; line-height: 1.5; }

  textarea {
    resize: vertical;
    padding: 10px 12px;
    background: #0f1219;
    border: 1px solid #2a2f3c;
    color: inherit;
    border-radius: 8px;
    font: inherit;
    font-size: 0.85rem;
    outline: none;
    line-height: 1.5;
  }
  textarea:focus {
    border-color: #3a5da8;
    box-shadow: 0 0 0 3px rgba(138, 180, 255, 0.12);
  }

  .loading-body {
    padding: 30px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    color: #9aa0ae;
    font-size: 0.85rem;
  }
  .spinner {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    border: 2px solid #2a2f3c;
    border-top-color: #8ab4ff;
    animation: spin 700ms linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  .result {
    padding: 18px 16px;
    border-radius: 12px;
    background: rgba(229, 117, 117, 0.08);
    border: 1px solid rgba(229, 117, 117, 0.35);
    text-align: center;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .result.passed {
    background: rgba(94, 203, 133, 0.1);
    border-color: rgba(94, 203, 133, 0.4);
    box-shadow: 0 0 40px rgba(94, 203, 133, 0.15);
  }
  .result-badge {
    display: inline-block;
    align-self: center;
    padding: 4px 14px;
    border-radius: 999px;
    font-size: 0.75rem;
    font-weight: 700;
    letter-spacing: 0.1em;
    background: #e57575;
    color: #12141a;
  }
  .result.passed .result-badge { background: #5ecb85; }
  .result-feedback { margin: 0; font-size: 0.9rem; color: #c8ccd6; line-height: 1.5; }

  footer { display: flex; justify-content: flex-end; gap: 8px; }
  footer button {
    padding: 8px 16px;
    background: #1c2030;
    border: 1px solid #2a2f3c;
    color: inherit;
    border-radius: 8px;
    cursor: pointer;
    font: inherit;
    font-size: 0.82rem;
  }
  footer button:hover:not(:disabled) { background: #242a3c; }
  footer button:disabled { opacity: 0.5; cursor: not-allowed; }
  footer button.primary {
    background: linear-gradient(180deg, #3a5da8, #2a4a90);
    border-color: #3a5da8;
    color: #fff;
  }

  .error {
    margin: 0;
    padding: 8px 10px;
    background: rgba(229, 117, 117, 0.1);
    border: 1px solid rgba(229, 117, 117, 0.35);
    border-radius: 6px;
    font-size: 0.8rem;
    color: #e57575;
  }
</style>
