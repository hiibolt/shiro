<script lang="ts">
  import type { Graph, Node } from "$lib/api";
  import { store } from "$lib/store.svelte";
  import { layout } from "$lib/layout";
  import { fade } from "svelte/transition";
  import { cubicOut } from "svelte/easing";

  interface Props {
    graph: Graph;
    selectedId: string | null;
    onSelect: (n: Node) => void;
  }
  let { graph, selectedId, onSelect }: Props = $props();

  const nodes = $derived(store.nodes[graph.id] ?? []);
  const g = $derived(layout(nodes));

  // Terminal ("goal") nodes: no other node in this graph depends on them.
  const terminalIds = $derived.by(() => {
    const referenced = new Set<string>();
    for (const n of nodes) for (const p of n.prerequisite_ids) referenced.add(p);
    return new Set(nodes.filter((n) => !referenced.has(n.id) && nodes.length > 1).map((n) => n.id));
  });

  // Neighborhood highlighting for selection.
  const neighborhood = $derived.by(() => {
    if (!selectedId) return null;
    const byId = new Map(nodes.map((n) => [n.id, n]));
    if (!byId.has(selectedId)) return null;
    const ancestors = new Set<string>([selectedId]);
    const descendants = new Set<string>([selectedId]);
    const walkUp = (id: string) => {
      const n = byId.get(id);
      if (!n) return;
      for (const p of n.prerequisite_ids) {
        if (!ancestors.has(p)) { ancestors.add(p); walkUp(p); }
      }
    };
    walkUp(selectedId);
    for (const n of nodes) {
      if (n.prerequisite_ids.some((p) => descendants.has(p))) descendants.add(n.id);
    }
    // Iterate once more to catch multi-hop descendants.
    let changed = true;
    while (changed) {
      changed = false;
      for (const n of nodes) {
        if (descendants.has(n.id)) continue;
        if (n.prerequisite_ids.some((p) => descendants.has(p))) {
          descendants.add(n.id);
          changed = true;
        }
      }
    }
    return { ancestors, descendants };
  });

  function isDim(id: string): boolean {
    if (!neighborhood) return false;
    return !neighborhood.ancestors.has(id) && !neighborhood.descendants.has(id);
  }
  function isEdgeActive(from: string, to: string): boolean {
    if (!neighborhood) return false;
    const a = neighborhood.ancestors, d = neighborhood.descendants;
    return (a.has(from) && a.has(to)) || (d.has(from) && d.has(to));
  }

  function statusStroke(n: Node): string {
    if (n.status.type === "Mastered") return "#5ecb85";
    if (n.status.type === "Learning") return "#e5b25c";
    return "#4a5266";
  }
  function statusGlow(n: Node): string {
    if (n.status.type === "Mastered") return "rgba(94, 203, 133, 0.35)";
    if (n.status.type === "Learning") return "rgba(229, 178, 92, 0.28)";
    return "rgba(80, 90, 110, 0)";
  }

  function childCount(n: Node): number {
    return n.subgraph_id ? (store.nodes[n.subgraph_id]?.length ?? 0) : 0;
  }

  function edgePath(x1: number, y1: number, x2: number, y2: number): string {
    const dy = Math.max(40, (y2 - y1) * 0.55);
    return `M ${x1} ${y1} C ${x1} ${y1 + dy}, ${x2} ${y2 - dy}, ${x2} ${y2}`;
  }

  // Pan + zoom.
  let scale = $state(1);
  let tx = $state(0);
  let ty = $state(0);
  let viewport = $state<HTMLDivElement | null>(null);
  let dragging = $state(false);
  let dragStart = { x: 0, y: 0, tx: 0, ty: 0 };
  let didFitFor = "";

  $effect(() => {
    if (!viewport || !g.width) return;
    if (didFitFor === graph.id + ":" + g.width + "x" + g.height) return;
    const vr = viewport.getBoundingClientRect();
    const pad = 60;
    const s = Math.min(1, (vr.width - pad * 2) / g.width, (vr.height - pad * 2) / g.height);
    scale = s;
    tx = (vr.width - g.width * s) / 2;
    ty = (vr.height - g.height * s) / 2;
    didFitFor = graph.id + ":" + g.width + "x" + g.height;
  });

  function onWheel(ev: WheelEvent) {
    if (!viewport) return;
    ev.preventDefault();
    const vr = viewport.getBoundingClientRect();
    const cx = ev.clientX - vr.left;
    const cy = ev.clientY - vr.top;
    const factor = Math.exp(-ev.deltaY * 0.0015);
    const next = Math.min(2.5, Math.max(0.15, scale * factor));
    // Zoom around cursor: keep world point under cursor fixed.
    tx = cx - (cx - tx) * (next / scale);
    ty = cy - (cy - ty) * (next / scale);
    scale = next;
  }

  function onPointerDown(ev: PointerEvent) {
    if ((ev.target as HTMLElement).closest(".node")) return;
    dragging = true;
    dragStart = { x: ev.clientX, y: ev.clientY, tx, ty };
    (ev.currentTarget as HTMLElement).setPointerCapture(ev.pointerId);
  }
  function onPointerMove(ev: PointerEvent) {
    if (!dragging) return;
    tx = dragStart.tx + (ev.clientX - dragStart.x);
    ty = dragStart.ty + (ev.clientY - dragStart.y);
  }
  function onPointerUp(ev: PointerEvent) {
    if (!dragging) return;
    dragging = false;
    (ev.currentTarget as HTMLElement).releasePointerCapture(ev.pointerId);
  }

  function resetView() {
    didFitFor = "";
    // Trigger re-fit by nudging viewport ref effect.
    const v = viewport;
    viewport = null;
    queueMicrotask(() => (viewport = v));
  }
</script>

<div
  class="viewport"
  role="application"
  bind:this={viewport}
  onwheel={onWheel}
  onpointerdown={onPointerDown}
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
  onpointercancel={onPointerUp}
  class:dragging
  in:fade={{ duration: 220, easing: cubicOut }}
>
  <div class="grid-bg"></div>

  <div class="world" style="transform: translate({tx}px, {ty}px) scale({scale});">
    <svg width={g.width} height={g.height} class="edges" aria-hidden="true">
      <defs>
        <marker id="arrow-idle" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
          <path d="M 0 0 L 10 5 L 0 10 z" fill="#3d4560" />
        </marker>
        <marker id="arrow-active" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
          <path d="M 0 0 L 10 5 L 0 10 z" fill="#8ab4ff" />
        </marker>
      </defs>
      {#each g.edges as e (e.from + "->" + e.to)}
        {@const active = isEdgeActive(e.from, e.to)}
        {@const dim = neighborhood && !active}
        <path
          d={edgePath(e.x1, e.y1, e.x2, e.y2)}
          stroke={active ? "#8ab4ff" : "#3d4560"}
          stroke-width={active ? 2 : 1.4}
          fill="none"
          opacity={dim ? 0.18 : 1}
          marker-end="url(#{active ? 'arrow-active' : 'arrow-idle'})"
          class:flow={active}
        />
      {/each}
    </svg>

    {#each g.nodes as pn (pn.node.id)}
      {@const kids = childCount(pn.node)}
      {@const selected = pn.node.id === selectedId}
      {@const dim = isDim(pn.node.id)}
      {@const terminal = terminalIds.has(pn.node.id)}
      <button
        class="node"
        class:selected
        class:dim
        class:terminal
        style="
          left: {pn.x}px;
          top:  {pn.y}px;
          width: {pn.w}px;
          height: {pn.h}px;
          --stroke: {statusStroke(pn.node)};
          --glow: {statusGlow(pn.node)};
        "
        onclick={(ev) => { ev.stopPropagation(); onSelect(pn.node); }}
        ondblclick={(ev) => { ev.stopPropagation(); if (pn.node.subgraph_id || kids >= 0) store.zoom(pn.node.id); }}
        title="Click to select · Double-click to dive in"
      >
        {#if terminal}<span class="crown" aria-hidden="true">★</span>{/if}
        <span class="status-dot" aria-hidden="true"></span>
        <span class="node-title">{pn.node.title}</span>
        {#if kids > 0}
          <span class="node-sub">
            <svg viewBox="0 0 16 16" width="10" height="10" aria-hidden="true">
              <path d="M2 4l6-3 6 3-6 3-6-3z M2 8l6 3 6-3 M2 12l6 3 6-3" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round"/>
            </svg>
            {kids} subtopic{kids === 1 ? "" : "s"}
          </span>
        {/if}
      </button>
    {/each}
  </div>

  <div class="viewport-hud">
    <button class="hud-btn" onclick={resetView} title="Fit to view">⤢</button>
    <div class="hud-zoom">{Math.round(scale * 100)}%</div>
  </div>
</div>

<style>
  .viewport {
    position: relative;
    flex: 1;
    overflow: hidden;
    background:
      radial-gradient(ellipse at 30% 20%, rgba(138, 180, 255, 0.06), transparent 55%),
      radial-gradient(ellipse at 80% 90%, rgba(94, 203, 133, 0.05), transparent 60%),
      #0f1117;
    cursor: grab;
    border-radius: 12px;
    border: 1px solid #1e2230;
    user-select: none;
    touch-action: none;
  }
  .viewport.dragging { cursor: grabbing; }

  .grid-bg {
    position: absolute;
    inset: 0;
    background-image:
      linear-gradient(rgba(255, 255, 255, 0.025) 1px, transparent 1px),
      linear-gradient(90deg, rgba(255, 255, 255, 0.025) 1px, transparent 1px);
    background-size: 32px 32px;
    pointer-events: none;
    mask-image: radial-gradient(ellipse at center, black 40%, transparent 90%);
  }

  .world {
    position: absolute;
    top: 0;
    left: 0;
    transform-origin: 0 0;
    will-change: transform;
    transition: transform 60ms linear;
  }

  .edges {
    position: absolute;
    top: 0;
    left: 0;
    pointer-events: none;
    overflow: visible;
  }
  .edges path { transition: stroke 200ms ease, opacity 200ms ease, stroke-width 200ms ease; }
  .flow {
    stroke-dasharray: 6 4;
    animation: flow 1.2s linear infinite;
  }
  @keyframes flow {
    to { stroke-dashoffset: -20; }
  }

  .node {
    position: absolute;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: flex-start;
    gap: 6px;
    padding: 10px 14px 10px 16px;
    background: linear-gradient(180deg, #1e2230 0%, #171a24 100%);
    color: #e7e9ee;
    border: 1.5px solid var(--stroke);
    border-radius: 10px;
    font: inherit;
    text-align: left;
    cursor: pointer;
    box-shadow:
      0 0 0 1px rgba(255, 255, 255, 0.02) inset,
      0 6px 16px rgba(0, 0, 0, 0.45),
      0 0 22px var(--glow);
    transition:
      transform 180ms cubic-bezier(0.22, 1, 0.36, 1),
      box-shadow 220ms ease,
      opacity 220ms ease,
      border-color 220ms ease,
      filter 220ms ease;
  }
  .node:hover {
    transform: translateY(-2px);
    box-shadow:
      0 0 0 1px rgba(255, 255, 255, 0.03) inset,
      0 10px 24px rgba(0, 0, 0, 0.55),
      0 0 30px var(--glow);
  }
  .node.selected {
    box-shadow:
      0 0 0 2px rgba(138, 180, 255, 0.55),
      0 12px 30px rgba(0, 0, 0, 0.6),
      0 0 40px var(--glow);
    transform: translateY(-2px);
  }
  .node.dim {
    opacity: 0.28;
    filter: saturate(0.5);
  }

  .node.terminal {
    border-color: transparent;
    background:
      linear-gradient(180deg, #1e2230 0%, #171a24 100%) padding-box,
      linear-gradient(135deg, #f5c76a, #e88bff 40%, #6ac4f5 80%, #f5c76a) border-box;
    border: 1.8px solid transparent;
    box-shadow:
      0 0 0 1px rgba(255, 255, 255, 0.04) inset,
      0 6px 20px rgba(0, 0, 0, 0.5),
      0 0 32px rgba(245, 199, 106, 0.35);
    animation: terminal-glow 3s ease-in-out infinite;
  }
  .node.terminal:hover {
    box-shadow:
      0 0 0 1px rgba(255, 255, 255, 0.06) inset,
      0 10px 28px rgba(0, 0, 0, 0.6),
      0 0 42px rgba(245, 199, 106, 0.5);
  }
  @keyframes terminal-glow {
    0%, 100% { box-shadow: 0 0 0 1px rgba(255,255,255,0.04) inset, 0 6px 20px rgba(0,0,0,0.5), 0 0 28px rgba(245, 199, 106, 0.3); }
    50%      { box-shadow: 0 0 0 1px rgba(255,255,255,0.04) inset, 0 6px 20px rgba(0,0,0,0.5), 0 0 44px rgba(232, 139, 255, 0.45); }
  }

  .crown {
    position: absolute;
    top: -10px;
    left: -6px;
    font-size: 1rem;
    background: linear-gradient(135deg, #f5c76a, #e88bff);
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
    filter: drop-shadow(0 0 6px rgba(245, 199, 106, 0.6));
    animation: sparkle 2.4s ease-in-out infinite;
    pointer-events: none;
  }
  @keyframes sparkle {
    0%, 100% { transform: rotate(-8deg) scale(1); }
    50%      { transform: rotate(-8deg) scale(1.15); filter: drop-shadow(0 0 10px rgba(232, 139, 255, 0.7)); }
  }

  .status-dot {
    position: absolute;
    top: 10px;
    right: 12px;
    width: 8px;
    height: 8px;
    border-radius: 999px;
    background: var(--stroke);
    box-shadow: 0 0 8px var(--glow);
  }
  .node-title {
    font-weight: 600;
    font-size: 0.9rem;
    line-height: 1.25;
    padding-right: 18px;
  }
  .node-sub {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 0.72rem;
    color: #8ab4ff;
    opacity: 0.85;
  }

  .viewport-hud {
    position: absolute;
    right: 12px;
    bottom: 12px;
    display: flex;
    gap: 6px;
    align-items: center;
    background: rgba(20, 24, 34, 0.7);
    backdrop-filter: blur(6px);
    padding: 4px 8px;
    border-radius: 999px;
    border: 1px solid #2a2f3c;
    font-size: 0.72rem;
    color: #9aa0ae;
  }
  .hud-btn {
    background: transparent;
    border: none;
    color: inherit;
    cursor: pointer;
    font-size: 0.9rem;
    padding: 2px 4px;
    line-height: 1;
  }
  .hud-btn:hover { color: #e7e9ee; }
  .hud-zoom { font-variant-numeric: tabular-nums; min-width: 34px; text-align: right; }
</style>
