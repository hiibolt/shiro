<script lang="ts">
  import { api, type LlmConfig } from "$lib/api";
  import { fade, scale } from "svelte/transition";
  import { cubicOut } from "svelte/easing";

  interface Props {
    onClose: () => void;
  }
  let { onClose }: Props = $props();

  type Provider = "Mock" | "Anthropic" | "Ollama";
  let provider = $state<Provider>("Mock");
  let anthropicKey = $state("");
  let anthropicModel = $state("claude-sonnet-4-6");
  let ollamaHost = $state("http://localhost:11434");
  let ollamaModel = $state("llama3.2");
  let saving = $state(false);
  let error = $state<string | null>(null);
  let loaded = $state(false);

  $effect(() => {
    api.getLlmConfig().then((cfg) => {
      provider = cfg.type;
      if (cfg.type === "Anthropic") {
        anthropicKey = cfg.api_key; // "***" if already set — treated as sentinel by backend
        anthropicModel = cfg.model;
      } else if (cfg.type === "Ollama") {
        ollamaHost = cfg.host;
        ollamaModel = cfg.model;
      }
      loaded = true;
    }).catch((e) => { error = String(e); loaded = true; });
  });

  async function save() {
    saving = true;
    error = null;
    try {
      let cfg: LlmConfig;
      if (provider === "Anthropic") {
        cfg = { type: "Anthropic", api_key: anthropicKey, model: anthropicModel.trim() };
      } else if (provider === "Ollama") {
        cfg = { type: "Ollama", host: ollamaHost.trim(), model: ollamaModel.trim() };
      } else {
        cfg = { type: "Mock" };
      }
      await api.setLlmConfig(cfg);
      onClose();
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
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
    aria-label="Settings"
    tabindex="-1"
    transition:scale={{ start: 0.94, duration: 220, easing: cubicOut }}
  >
    <header>
      <h2>Settings</h2>
      <button class="close" onclick={onClose} aria-label="Close">×</button>
    </header>

    {#if !loaded}
      <p class="muted">Loading…</p>
    {:else}
      <section>
        <h3>AI Provider</h3>
        <p class="hint">Choose which AI will generate your learning graphs and grade your answers.</p>

        <div class="provider-choice">
          {#each ["Mock", "Anthropic", "Ollama"] as p (p)}
            <label class="option" class:selected={provider === p}>
              <input type="radio" name="provider" value={p} bind:group={provider} />
              <div class="option-body">
                <div class="option-title">{p === "Mock" ? "Built-in demo" : p}</div>
                <div class="option-desc">
                  {#if p === "Mock"}Fixed sample graphs. No account needed.
                  {:else if p === "Anthropic"}Claude by Anthropic. Best quality. Requires an API key.
                  {:else}Local models on your machine. Not implemented yet.
                  {/if}
                </div>
              </div>
            </label>
          {/each}
        </div>
      </section>

      {#if provider === "Anthropic"}
        <section transition:fade={{ duration: 140 }}>
          <label class="field">
            <span>API key</span>
            <input
              type="password"
              placeholder="sk-ant-..."
              bind:value={anthropicKey}
              autocomplete="off"
            />
            <small>Stored locally in your app config. Get one at console.anthropic.com.</small>
          </label>
          <label class="field">
            <span>Model</span>
            <input type="text" bind:value={anthropicModel} />
          </label>
        </section>
      {:else if provider === "Ollama"}
        <section transition:fade={{ duration: 140 }}>
          <label class="field">
            <span>Host</span>
            <input type="text" bind:value={ollamaHost} />
            <small>The URL where Ollama is running.</small>
          </label>
          <label class="field">
            <span>Model</span>
            <input type="text" bind:value={ollamaModel} />
          </label>
          <p class="warn">⚠ Ollama support isn't wired up yet — selecting it will fail on use.</p>
        </section>
      {/if}

      {#if error}<p class="error">{error}</p>{/if}

      <footer>
        <button onclick={onClose} disabled={saving}>Cancel</button>
        <button class="primary" onclick={save} disabled={saving}>
          {saving ? "Saving…" : "Save"}
        </button>
      </footer>
    {/if}
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.55);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .modal {
    width: 480px;
    max-width: calc(100vw - 40px);
    max-height: calc(100vh - 40px);
    overflow: auto;
    background: #14171f;
    border: 1px solid #2a2f3c;
    border-radius: 14px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.7);
    padding: 20px 22px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  h2 { margin: 0; font-size: 1.05rem; }
  h3 { margin: 0 0 4px; font-size: 0.85rem; color: #c8ccd6; font-weight: 600; }
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

  section { display: flex; flex-direction: column; gap: 10px; }
  .hint { margin: 0 0 8px; font-size: 0.78rem; color: #9aa0ae; }

  .provider-choice { display: flex; flex-direction: column; gap: 8px; }
  .option {
    display: flex;
    gap: 10px;
    padding: 10px 12px;
    border: 1px solid #2a2f3c;
    border-radius: 10px;
    cursor: pointer;
    background: #1c2030;
    transition: border-color 160ms, background 160ms;
  }
  .option:hover { background: #212637; }
  .option.selected { border-color: #3a5da8; background: rgba(58, 93, 168, 0.12); }
  .option input { margin-top: 3px; accent-color: #8ab4ff; }
  .option-body { display: flex; flex-direction: column; gap: 2px; }
  .option-title { font-size: 0.9rem; font-weight: 600; }
  .option-desc { font-size: 0.75rem; color: #9aa0ae; }

  .field { display: flex; flex-direction: column; gap: 6px; }
  .field > span { font-size: 0.78rem; color: #c8ccd6; }
  .field input {
    padding: 8px 10px;
    background: #0f1219;
    border: 1px solid #2a2f3c;
    border-radius: 6px;
    color: inherit;
    font: inherit;
    outline: none;
  }
  .field input:focus { border-color: #3a5da8; box-shadow: 0 0 0 3px rgba(138, 180, 255, 0.12); }
  .field small { font-size: 0.7rem; color: #6b7180; }

  .warn {
    margin: 0;
    padding: 8px 10px;
    background: rgba(229, 178, 92, 0.08);
    border: 1px solid rgba(229, 178, 92, 0.3);
    border-radius: 6px;
    font-size: 0.75rem;
    color: #e5b25c;
  }
  .error {
    margin: 0;
    padding: 8px 10px;
    background: rgba(229, 117, 117, 0.08);
    border: 1px solid rgba(229, 117, 117, 0.3);
    border-radius: 6px;
    font-size: 0.8rem;
    color: #e57575;
  }
  .muted { color: #6b7180; font-size: 0.85rem; }

  footer { display: flex; justify-content: flex-end; gap: 8px; margin-top: 4px; }
  footer button {
    padding: 7px 14px;
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
</style>
