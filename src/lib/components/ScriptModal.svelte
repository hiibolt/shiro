<script lang="ts">
  import { api, type Node } from "$lib/api";
  import { fade, scale, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";

  interface Props {
    node: Node;
    onClose: () => void;
  }
  let { node, onClose }: Props = $props();

  type Stage = "loading" | "ready" | "error";
  let stage = $state<Stage>("loading");
  let script = $state("");
  let error = $state<string | null>(null);
  let copied = $state(false);

  async function generate() {
    stage = "loading";
    error = null;
    try {
      script = await api.createLearningScript(node.id);
      stage = "ready";
    } catch (e) {
      error = String(e);
      stage = "error";
    }
  }

  async function copy() {
    try {
      await navigator.clipboard.writeText(script);
      copied = true;
      setTimeout(() => (copied = false), 1600);
    } catch (e) {
      error = String(e);
    }
  }

  $effect(() => { generate(); });
</script>

<div class="backdrop" transition:fade={{ duration: 160 }} onclick={onClose} role="presentation">
  <div
    class="modal"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.key === "Escape" && onClose()}
    role="dialog"
    aria-modal="true"
    aria-label="Learning script"
    tabindex="-1"
    transition:scale={{ start: 0.94, duration: 220, easing: cubicOut }}
  >
    <header>
      <div class="head-icon">◈</div>
      <div class="head-text">
        <div class="head-eyebrow">Learning script</div>
        <h2>{node.title}</h2>
      </div>
      <button class="close" onclick={onClose} aria-label="Close">×</button>
    </header>

    {#if stage === "loading"}
      <div class="body loading-body" in:fade={{ duration: 200 }}>
        <div class="spinner"></div>
        <p>Crafting a coaching prompt with graph context…</p>
      </div>
    {:else if stage === "error"}
      <div class="body" in:fly={{ y: 8, duration: 200, easing: cubicOut }}>
        <p class="error">{error}</p>
        <footer>
          <button onclick={onClose}>Close</button>
          <button class="primary" onclick={generate}>Retry</button>
        </footer>
      </div>
    {:else if stage === "ready"}
      <div class="body" in:fly={{ y: 8, duration: 220, easing: cubicOut }}>
        <p class="desc">
          Drop this into Claude Code, ChatGPT, or any other harness. It's fully self-contained.
        </p>
        <div class="code-wrap">
          <div class="code-bar">
            <span class="code-label">{script.split("\n").length} lines · {script.length} chars</span>
            <button class="copy-btn" class:copied onclick={copy}>
              {copied ? "✓ Copied" : "Copy"}
            </button>
          </div>
          <pre class="code"><code>{script}</code></pre>
        </div>
        <footer>
          <button onclick={generate}>Regenerate</button>
          <button class="primary" onclick={onClose}>Done</button>
        </footer>
      </div>
    {/if}
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
    width: 720px;
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
  header { display: flex; align-items: center; gap: 12px; }
  .head-icon {
    width: 36px;
    height: 36px;
    border-radius: 10px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, #8a5cf6, #3a5da8);
    color: white;
    font-size: 1.1rem;
    font-weight: 600;
    box-shadow: 0 4px 14px rgba(138, 92, 246, 0.4);
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

  .body { display: flex; flex-direction: column; gap: 12px; min-height: 0; overflow: hidden; }
  .desc { margin: 0; font-size: 0.85rem; color: #9aa0ae; line-height: 1.5; }

  .code-wrap {
    display: flex;
    flex-direction: column;
    background: #0b0d13;
    border: 1px solid #2a2f3c;
    border-radius: 10px;
    overflow: hidden;
    min-height: 0;
    flex: 1;
  }
  .code-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 10px;
    background: #14171f;
    border-bottom: 1px solid #2a2f3c;
    font-size: 0.72rem;
    color: #6b7180;
  }
  .copy-btn {
    padding: 4px 12px;
    background: #1c2030;
    border: 1px solid #2a2f3c;
    color: #c8ccd6;
    border-radius: 6px;
    cursor: pointer;
    font: inherit;
    font-size: 0.75rem;
    transition: background 140ms, color 140ms, border-color 140ms;
  }
  .copy-btn:hover { background: #242a3c; color: #e7e9ee; border-color: #3a4053; }
  .copy-btn.copied {
    background: rgba(94, 203, 133, 0.15);
    color: #5ecb85;
    border-color: rgba(94, 203, 133, 0.4);
  }
  pre.code {
    margin: 0;
    padding: 14px 16px;
    overflow: auto;
    max-height: 55vh;
    color: #d4d8e0;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
    font-size: 0.78rem;
    line-height: 1.55;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .loading-body {
    padding: 40px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
    color: #9aa0ae;
    font-size: 0.85rem;
  }
  .spinner {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    border: 2px solid #2a2f3c;
    border-top-color: #8a5cf6;
    animation: spin 700ms linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

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
  footer button.primary {
    background: linear-gradient(180deg, #8a5cf6, #6a3fd8);
    border-color: #8a5cf6;
    color: #fff;
  }
  footer button.primary:hover { background: linear-gradient(180deg, #9a6cff, #7a4fe8); }

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
