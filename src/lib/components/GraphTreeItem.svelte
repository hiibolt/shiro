<script lang="ts">
  import { store } from "$lib/store.svelte";
  import type { Graph, Node } from "$lib/api";
  import { slide } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import Self from "./GraphTreeItem.svelte";

  interface Props {
    graph: Graph;
    depth?: number;
  }
  let { graph, depth = 0 }: Props = $props();

  let expanded = $state(false);
  let loading = $state(false);

  const nodes = $derived(store.nodes[graph.id] ?? []);
  const childBearing = $derived(nodes.filter((n) => n.subgraph_id));
  const isActive = $derived(store.focus?.id === graph.id);

  // Cheap "might have children" hint before we've fetched: unknown → assume yes.
  const maybeHasChildren = $derived(
    store.nodes[graph.id] === undefined || childBearing.length > 0
  );

  async function toggle(ev: MouseEvent) {
    ev.stopPropagation();
    if (!expanded && !store.nodes[graph.id]) {
      loading = true;
      try {
        await store.ensureNodesLoaded(graph.id);
        // Also resolve child subgraphs so we can display their titles.
        const kids = (store.nodes[graph.id] ?? []).filter((n) => n.subgraph_id);
        await Promise.all(kids.map((n) => store.ensureGraphLoaded(n.subgraph_id!)));
      } finally {
        loading = false;
      }
    }
    expanded = !expanded;
  }

  async function open(ev: MouseEvent) {
    ev.stopPropagation();
    await store.openGraph(graph);
  }

  function childGraph(n: Node): Graph | null {
    return n.subgraph_id ? store.graphs[n.subgraph_id] ?? null : null;
  }
</script>

<div class="row" class:active={isActive} style="padding-left: {6 + depth * 12}px">
  <button
    class="chev"
    onclick={toggle}
    aria-label={expanded ? "Collapse" : "Expand"}
    class:invisible={!maybeHasChildren}
  >
    {#if loading}
      <span class="spin"></span>
    {:else}
      <svg viewBox="0 0 10 10" width="9" height="9" class:rot={expanded} aria-hidden="true">
        <path d="M3 2 L7 5 L3 8" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    {/if}
  </button>
  <button class="label" onclick={open} title={graph.title}>
    <span class="label-text">{graph.title}</span>
  </button>
</div>

{#if expanded}
  <div class="children" transition:slide={{ duration: 160, easing: cubicOut }}>
    {#each childBearing as n (n.id)}
      {@const cg = childGraph(n)}
      {#if cg}
        <Self graph={cg} depth={depth + 1} />
      {/if}
    {/each}
    {#if childBearing.length === 0 && !loading}
      <div class="empty" style="padding-left: {6 + (depth + 1) * 12 + 20}px">No subgraphs</div>
    {/if}
  </div>
{/if}

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 2px;
    padding-right: 4px;
    border-radius: 6px;
    cursor: default;
    transition: background 140ms;
  }
  .row:hover { background: #1a1e29; }
  .row.active { background: rgba(58, 93, 168, 0.16); }

  .chev {
    width: 18px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    color: #6b7180;
    cursor: pointer;
    padding: 0;
    border-radius: 4px;
    flex-shrink: 0;
  }
  .chev:hover { color: #e7e9ee; background: rgba(255, 255, 255, 0.04); }
  .chev.invisible { visibility: hidden; }
  .chev svg { transition: transform 160ms ease; }
  .chev svg.rot { transform: rotate(90deg); }

  .spin {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    border: 1.5px solid #2a2f3c;
    border-top-color: #8ab4ff;
    animation: spin 700ms linear infinite;
    display: inline-block;
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  .label {
    flex: 1;
    text-align: left;
    padding: 5px 6px;
    background: transparent;
    border: none;
    color: #c8ccd6;
    font: inherit;
    font-size: 0.82rem;
    cursor: pointer;
    overflow: hidden;
    border-radius: 4px;
  }
  .row.active .label { color: #8ab4ff; font-weight: 500; }
  .label-text {
    display: block;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .empty {
    font-size: 0.72rem;
    color: #4a5266;
    padding: 3px 0;
    font-style: italic;
  }
</style>
