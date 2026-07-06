<script lang="ts">
  import { store } from "$lib/store.svelte";
  import type { Graph } from "$lib/api";
  import { fade, slide } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import GraphTreeItem from "./GraphTreeItem.svelte";

  let creating = $state(false);
  let goal = $state("");

  async function onCreate() {
    const g = goal.trim();
    if (!g) return;
    creating = false;
    goal = "";
    await store.generate(g);
  }

  function fmtDate(iso: string): string {
    const d = new Date(iso);
    const now = new Date();
    const diffMs = now.getTime() - d.getTime();
    const days = Math.floor(diffMs / 86_400_000);
    if (days === 0) return "today";
    if (days === 1) return "yesterday";
    if (days < 7) return `${days}d ago`;
    return d.toLocaleDateString(undefined, { month: "short", day: "numeric" });
  }

  // The current root: walk up from focus.
  const currentRootId = $derived.by<string | null>(() => {
    let cur: Graph | null = store.focus;
    while (cur?.parent_node_id) {
      let parentGraphId: string | null = null;
      for (const [gid, ns] of Object.entries(store.nodes)) {
        if (ns.some((n) => n.id === cur!.parent_node_id)) { parentGraphId = gid; break; }
      }
      cur = parentGraphId ? store.graphs[parentGraphId] ?? null : null;
    }
    return cur?.id ?? null;
  });
</script>

<aside class="browser">
  <div class="head">
    <span class="head-title">Graphs</span>
    <button class="new-btn" onclick={() => (creating = !creating)} title="New graph">
      {creating ? "×" : "+"}
    </button>
  </div>

  {#if creating}
    <div class="create-panel" transition:slide={{ duration: 180, easing: cubicOut }}>
      <input
        type="text"
        placeholder="What do you want to learn?"
        bind:value={goal}
        onkeydown={(e) => e.key === "Enter" && onCreate()}
        disabled={store.loading}
      />
      <div class="create-actions">
        <button onclick={() => { creating = false; goal = ""; }} disabled={store.loading}>Cancel</button>
        <button class="primary" onclick={onCreate} disabled={store.loading || !goal.trim()}>
          {store.loading ? "…" : "Generate"}
        </button>
      </div>
    </div>
  {/if}

  {#if store.rootGraphs.length === 0}
    <div class="empty" in:fade={{ duration: 200 }}>
      <p>No graphs yet.</p>
      <p class="empty-hint">Click <strong>+</strong> to create one.</p>
    </div>
  {:else}
    <div class="tree">
      {#each store.rootGraphs as g (g.id)}
        <div class="root-block">
          <GraphTreeItem graph={g} />
          <div class="root-meta">{fmtDate(g.created_at)}</div>
        </div>
      {/each}
    </div>
  {/if}
</aside>

<style>
  .browser {
    width: 240px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    background: #10131a;
    border: 1px solid #1e2230;
    border-radius: 12px;
    padding: 10px 8px;
    gap: 8px;
    overflow: hidden;
    min-height: 0;
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 6px 6px;
    border-bottom: 1px solid #1e2230;
  }
  .head-title {
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: #6b7180;
    font-weight: 600;
  }
  .new-btn {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #1c2030;
    border: 1px solid #2a2f3c;
    color: #c8ccd6;
    border-radius: 6px;
    cursor: pointer;
    font-size: 1rem;
    line-height: 1;
    transition: background 160ms, color 160ms, border-color 160ms;
  }
  .new-btn:hover { background: #242a3c; color: #e7e9ee; border-color: #3a4053; }

  .create-panel {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px;
    background: rgba(58, 93, 168, 0.08);
    border: 1px solid rgba(58, 93, 168, 0.35);
    border-radius: 8px;
  }
  .create-panel input {
    padding: 7px 10px;
    background: #0f1219;
    border: 1px solid #2a2f3c;
    border-radius: 6px;
    color: inherit;
    font: inherit;
    font-size: 0.82rem;
    outline: none;
  }
  .create-panel input:focus {
    border-color: #3a5da8;
    box-shadow: 0 0 0 3px rgba(138, 180, 255, 0.12);
  }
  .create-actions { display: flex; gap: 6px; justify-content: flex-end; }
  .create-actions button {
    padding: 5px 12px;
    background: transparent;
    border: 1px solid #2a2f3c;
    color: #9aa0ae;
    border-radius: 6px;
    cursor: pointer;
    font: inherit;
    font-size: 0.78rem;
  }
  .create-actions button:hover:not(:disabled) { background: #1c2030; color: #e7e9ee; }
  .create-actions button:disabled { opacity: 0.5; cursor: not-allowed; }
  .create-actions button.primary {
    background: linear-gradient(180deg, #3a5da8, #2a4a90);
    border-color: #3a5da8;
    color: #fff;
  }
  .create-actions button.primary:hover:not(:disabled) { background: linear-gradient(180deg, #456bbd, #345aa8); }

  .tree {
    display: flex;
    flex-direction: column;
    gap: 6px;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
    padding: 2px 0;
  }
  .root-block {
    display: flex;
    flex-direction: column;
    gap: 0;
  }
  .root-meta {
    font-size: 0.68rem;
    color: #4a5266;
    padding-left: 30px;
    margin-top: -2px;
  }

  .empty {
    padding: 20px 12px;
    text-align: center;
    color: #6b7180;
  }
  .empty p { margin: 0; font-size: 0.8rem; }
  .empty-hint { margin-top: 4px !important; font-size: 0.72rem !important; }
  .empty strong { color: #c8ccd6; }
</style>
