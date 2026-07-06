<script lang="ts">
  import { store } from "$lib/store.svelte";
  import { api, type Graph, type Node } from "$lib/api";
  import GraphView from "$lib/components/GraphView.svelte";
  import GraphBrowser from "$lib/components/GraphBrowser.svelte";
  import SettingsModal from "$lib/components/SettingsModal.svelte";
  import VerificationModal from "$lib/components/VerificationModal.svelte";
  import NodeEditor from "$lib/components/NodeEditor.svelte";
  import { fade, scale, fly, slide } from "svelte/transition";
  import { cubicOut } from "svelte/easing";

  import { onMount } from "svelte";

  let selected = $state<Node | null>(null);
  let showSettings = $state(false);
  let sidebarHidden = $state(false);
  let verifyingNode = $state<Node | null>(null);
  let editorMode = $state<null | "create" | "edit">(null);

  onMount(() => { store.restoreMostRecent(); });

  // Clear stale selection whenever the focused graph changes.
  $effect(() => {
    const fid = store.focus?.id;
    if (selected && selected.graph_id !== fid) selected = null;
  });

  // Ancestor chain for breadcrumb (root → … → current).
  const trail = $derived.by<Graph[]>(() => {
    const out: Graph[] = [];
    let cur = store.focus;
    while (cur) {
      out.unshift(cur);
      if (!cur.parent_node_id) break;
      // Find graph containing that parent node.
      let parentGraphId: string | null = null;
      for (const [gid, ns] of Object.entries(store.nodes)) {
        if (ns.some((n) => n.id === cur!.parent_node_id)) { parentGraphId = gid; break; }
      }
      cur = parentGraphId ? store.graphs[parentGraphId] ?? null : null;
    }
    return out;
  });

  async function onZoomOutTo(g: Graph) {
    if (g.id === store.focus?.id) return;
    selected = null;
    // Walk up until we reach the target.
    while (store.focus && store.focus.id !== g.id && store.canZoomOut) {
      await store.zoomOut();
    }
  }

  function statusLabel(n: Node): string {
    return n.status.type;
  }
  function statusColor(n: Node): string {
    if (n.status.type === "Mastered") return "#5ecb85";
    if (n.status.type === "Learning") return "#e5b25c";
    return "#6b7180";
  }

  async function onDelete(n: Node) {
    if (!confirm(`Delete "${n.title}"? Children will keep their other prerequisites.`)) return;
    await api.deleteNode(n.id, false);
    selected = null;
    await store.refresh();
  }
</script>

<div class="app">
  <header>
    <div class="brand">
      <span class="brand-dot"></span>
      <h1>Learning Graph</h1>
    </div>

    {#if trail.length > 0}
      <nav class="trail" aria-label="Graph hierarchy">
        {#each trail as g, i (g.id)}
          <button
            class="crumb"
            class:current={i === trail.length - 1}
            onclick={() => onZoomOutTo(g)}
            disabled={i === trail.length - 1}
          >
            {g.title}
          </button>
          {#if i < trail.length - 1}<span class="crumb-sep">›</span>{/if}
        {/each}
      </nav>
    {/if}

    <div class="controls">
      {#if sidebarHidden}
        <button
          class="icon-btn"
          onclick={() => (sidebarHidden = false)}
          title="Show graph browser"
          aria-label="Show graph browser"
        >
          <svg viewBox="0 0 12 12" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
            <path d="M5 3 L9 6 L5 9" />
          </svg>
        </button>
      {/if}
      <button class="icon-btn" onclick={() => (showSettings = true)} title="Settings" aria-label="Settings">
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="3"/>
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33h.01a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82v.01a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
        </svg>
      </button>
    </div>
  </header>

  {#if showSettings}
    <SettingsModal onClose={() => (showSettings = false)} />
  {/if}

  {#if store.error}<p class="error">{store.error}</p>{/if}

  <div class="body">
  {#if !sidebarHidden}
    <div class="sidebar-wrap" transition:slide={{ axis: "x", duration: 200, easing: cubicOut }}>
      <GraphBrowser onCollapse={() => (sidebarHidden = true)} />
    </div>
  {/if}

  <main class="stage">
    {#if store.focus}
      {#key store.focus.id}
        <div class="graph-slot" in:scale={{ start: 0.92, duration: 380, easing: cubicOut }} out:scale={{ start: 1.08, duration: 260, easing: cubicOut }}>
          <GraphView
            graph={store.focus}
            selectedId={selected?.id ?? null}
            onSelect={(n) => (selected = n)}
          />
        </div>
      {/key}
    {:else}
      <div class="empty" in:fade={{ duration: 300 }}>
        <div class="empty-icon">◆</div>
        <p>Enter a topic and hit <kbd>Generate</kbd> to build a DAG.</p>
      </div>
    {/if}

    {#if store.loading && store.focus}
      <div class="loading-veil" in:fade={{ duration: 120 }} out:fade={{ duration: 200 }}>
        <div class="spinner"></div>
      </div>
    {/if}

    {#if store.focus}
      <button class="fab" onclick={() => (editorMode = "create")} title="Add node" aria-label="Add node">
        <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round">
          <line x1="12" y1="5" x2="12" y2="19"/>
          <line x1="5" y1="12" x2="19" y2="12"/>
        </svg>
      </button>
    {/if}
  </main>
  </div>

  {#if verifyingNode}
    <VerificationModal
      node={verifyingNode}
      onClose={() => {
        verifyingNode = null;
        if (selected) {
          const ns = store.nodes[selected.graph_id] ?? [];
          selected = ns.find((x) => x.id === selected!.id) ?? null;
        }
      }}
    />
  {/if}

  {#if editorMode && store.focus}
    <NodeEditor
      mode={editorMode}
      graphId={store.focus.id}
      existing={editorMode === "edit" ? selected : null}
      onClose={() => (editorMode = null)}
      onSaved={(n) => { if (editorMode === "edit") selected = n; }}
    />
  {/if}

  {#if selected}
    <aside class="detail" in:fly={{ x: 20, duration: 240, easing: cubicOut }} out:fly={{ x: 20, duration: 180 }}>
      <div class="detail-head">
        <span class="detail-status" style="background: {statusColor(selected)}; box-shadow: 0 0 10px {statusColor(selected)}66;"></span>
        <h2>{selected.title}</h2>
        <button class="close" onclick={() => (selected = null)} aria-label="Close">×</button>
      </div>
      <p class="desc">{selected.description}</p>
      <div class="meta-row">
        <span class="pill" style="color: {statusColor(selected)}; border-color: {statusColor(selected)}55;">
          {statusLabel(selected)}
        </span>
        {#if selected.prerequisite_ids.length > 0}
          <span class="meta-fact">{selected.prerequisite_ids.length} prerequisite{selected.prerequisite_ids.length === 1 ? "" : "s"}</span>
        {/if}
      </div>
      <div class="detail-actions">
        <button class="primary" onclick={() => { const id = selected!.id; selected = null; store.zoom(id); }}>
          Dive in ↓
        </button>
        <button onclick={() => (verifyingNode = selected)}>Verify</button>
      </div>
      <div class="detail-actions secondary">
        <button onclick={() => (editorMode = "edit")}>Edit</button>
        <button class="danger" onclick={() => onDelete(selected!)}>Delete</button>
      </div>
      <p class="tip">Double-click a node to dive in · Scroll to zoom · Drag to pan</p>
    </aside>
  {/if}
</div>

<style>
  :global(html, body) {
    margin: 0;
    background: #0a0c11;
    color: #e7e9ee;
    font-family: ui-sans-serif, system-ui, -apple-system, sans-serif;
    height: 100%;
    overflow: hidden;
  }
  :global(*) { box-sizing: border-box; }

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    padding: 14px 18px 18px;
    gap: 12px;
    position: relative;
  }

  header {
    display: flex;
    align-items: center;
    gap: 16px;
    flex-wrap: wrap;
  }
  .brand { display: flex; align-items: center; gap: 8px; }
  .brand-dot {
    width: 10px;
    height: 10px;
    border-radius: 3px;
    background: linear-gradient(135deg, #8ab4ff, #5ecb85);
    box-shadow: 0 0 12px rgba(138, 180, 255, 0.5);
  }
  h1 {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 600;
    letter-spacing: 0.02em;
    color: #e7e9ee;
  }

  .trail {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 10px;
    background: #14171f;
    border: 1px solid #1e2230;
    border-radius: 999px;
    min-height: 28px;
    overflow: hidden;
  }
  .crumb {
    background: transparent;
    border: none;
    color: #9aa0ae;
    font: inherit;
    font-size: 0.78rem;
    padding: 3px 8px;
    border-radius: 999px;
    cursor: pointer;
    transition: color 160ms, background 160ms;
    max-width: 160px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .crumb:hover:not(:disabled) { color: #e7e9ee; background: #1c2030; }
  .crumb.current { color: #8ab4ff; cursor: default; }
  .crumb-sep { color: #4a5266; font-size: 0.75rem; user-select: none; }

  .controls { display: flex; gap: 8px; margin-left: auto; }

  button {
    padding: 7px 14px;
    background: #1c2030;
    border: 1px solid #2a2f3c;
    color: inherit;
    border-radius: 8px;
    cursor: pointer;
    font: inherit;
    font-size: 0.82rem;
    transition: background 160ms, border-color 160ms, transform 120ms, opacity 160ms;
  }
  button:hover:not(:disabled) { background: #242a3c; border-color: #3a4053; }
  button:active:not(:disabled) { transform: translateY(1px); }
  button:disabled { opacity: 0.5; cursor: not-allowed; }
  button.primary {
    background: linear-gradient(180deg, #3a5da8, #2a4a90);
    border-color: #3a5da8;
    color: #fff;
  }
  button.primary:hover:not(:disabled) { background: linear-gradient(180deg, #456bbd, #345aa8); }
  .icon-btn {
    padding: 7px 8px;
    background: #14171f;
    border: 1px solid #1e2230;
    color: #9aa0ae;
    border-radius: 8px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color 160ms, background 160ms, border-color 160ms;
  }
  .icon-btn:hover { color: #e7e9ee; background: #1c2030; border-color: #2a2f3c; }

  .error {
    color: #e57575;
    margin: 0;
    font-size: 0.85rem;
    padding: 6px 10px;
    background: rgba(229, 117, 117, 0.08);
    border: 1px solid rgba(229, 117, 117, 0.25);
    border-radius: 6px;
  }

  .body {
    flex: 1;
    display: flex;
    gap: 12px;
    min-height: 0;
  }
  .sidebar-wrap {
    display: flex;
    min-height: 0;
    overflow: hidden;
  }
  .stage {
    flex: 1;
    position: relative;
    display: flex;
    min-height: 0;
  }
  .graph-slot {
    position: absolute;
    inset: 0;
    display: flex;
    transform-origin: center center;
  }

  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: #6b7180;
    gap: 12px;
    border: 1px dashed #1e2230;
    border-radius: 12px;
  }
  .empty-icon {
    font-size: 2.2rem;
    color: #2a3350;
  }
  kbd {
    font-family: inherit;
    background: #1c2030;
    border: 1px solid #2a2f3c;
    padding: 1px 6px;
    border-radius: 4px;
    font-size: 0.78rem;
    color: #c8ccd6;
  }

  .loading-veil {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(10, 12, 17, 0.45);
    backdrop-filter: blur(2px);
    pointer-events: none;
    border-radius: 12px;
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

  .detail {
    position: absolute;
    right: 18px;
    bottom: 18px;
    width: 340px;
    background: rgba(20, 23, 31, 0.95);
    backdrop-filter: blur(10px);
    border: 1px solid #2a2f3c;
    border-radius: 12px;
    padding: 16px;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.6);
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .detail-head { display: flex; align-items: center; gap: 10px; }
  .detail-status {
    width: 10px;
    height: 10px;
    border-radius: 999px;
    flex-shrink: 0;
  }
  .detail h2 { margin: 0; font-size: 1rem; flex: 1; line-height: 1.3; }
  .close {
    background: transparent;
    border: none;
    color: #6b7180;
    font-size: 1.3rem;
    padding: 0 4px;
    cursor: pointer;
    line-height: 1;
  }
  .close:hover { color: #e7e9ee; }
  .desc { margin: 0; font-size: 0.85rem; color: #c8ccd6; line-height: 1.5; }
  .meta-row { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
  .pill {
    padding: 2px 10px;
    border-radius: 999px;
    font-size: 0.72rem;
    background: transparent;
    border: 1px solid;
    text-transform: capitalize;
  }
  .meta-fact { font-size: 0.72rem; color: #6b7180; }
  .detail-actions { display: flex; gap: 6px; margin-top: 2px; }
  .detail-actions button { flex: 1; }
  .detail-actions.secondary button {
    background: transparent;
    border: 1px solid #2a2f3c;
    color: #9aa0ae;
    font-size: 0.76rem;
    padding: 5px 10px;
  }
  .detail-actions.secondary button:hover { color: #e7e9ee; background: #1c2030; }
  .detail-actions.secondary .danger { color: #e57575; border-color: rgba(229, 117, 117, 0.3); }
  .detail-actions.secondary .danger:hover { background: rgba(229, 117, 117, 0.1); color: #ff9090; }

  .fab {
    position: absolute;
    left: 18px;
    bottom: 18px;
    width: 46px;
    height: 46px;
    border-radius: 50%;
    background: linear-gradient(135deg, #3a5da8, #2a4a90);
    border: none;
    color: white;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 8px 24px rgba(58, 93, 168, 0.5);
    transition: transform 160ms, box-shadow 200ms;
    z-index: 10;
  }
  .fab:hover { transform: translateY(-2px) scale(1.05); box-shadow: 0 12px 30px rgba(58, 93, 168, 0.6); }
  .fab:active { transform: translateY(0); }
  .tip { margin: 4px 0 0; font-size: 0.7rem; color: #6b7180; line-height: 1.5; }
</style>
