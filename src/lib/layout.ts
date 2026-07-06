import type { Node } from "./api";

export interface PositionedNode {
  node: Node;
  x: number;
  y: number;
  w: number;
  h: number;
  layer: number;
}

export interface Edge {
  from: string;
  to: string;
  x1: number;
  y1: number;
  x2: number;
  y2: number;
}

export interface LaidOut {
  nodes: PositionedNode[];
  edges: Edge[];
  width: number;
  height: number;
}

const NODE_W = 220;
const NODE_H = 78;
const H_GAP = 40;
const V_GAP = 60;
const PAD = 40;

/**
 * Layer each node at `1 + max(layer(prereq))`. Roots (no prereqs) sit on layer 0.
 * Within a layer, order by a barycentric heuristic against the previous layer
 * to reduce edge crossings — not optimal, but fine for PoC-scale graphs.
 */
export function layout(nodes: Node[]): LaidOut {
  const byId = new Map(nodes.map((n) => [n.id, n]));
  const layer = new Map<string, number>();

  function layerOf(id: string, seen: Set<string>): number {
    const cached = layer.get(id);
    if (cached != null) return cached;
    if (seen.has(id)) return 0;
    seen.add(id);
    const n = byId.get(id);
    if (!n || n.prerequisite_ids.length === 0) {
      layer.set(id, 0);
      return 0;
    }
    const l = 1 + Math.max(...n.prerequisite_ids.map((p) => layerOf(p, seen)));
    layer.set(id, l);
    return l;
  }
  for (const n of nodes) layerOf(n.id, new Set());

  const layers: string[][] = [];
  for (const n of nodes) {
    const l = layer.get(n.id)!;
    (layers[l] ||= []).push(n.id);
  }

  // Barycenter ordering top-down.
  const orderIn = new Map<string, number>();
  for (let li = 0; li < layers.length; li++) {
    const ids = layers[li];
    if (li === 0) {
      ids.sort((a, b) => byId.get(a)!.title.localeCompare(byId.get(b)!.title));
    } else {
      ids.sort((a, b) => {
        const bary = (id: string) => {
          const prereqs = byId.get(id)!.prerequisite_ids;
          if (prereqs.length === 0) return 0;
          const sum = prereqs.reduce((s, p) => s + (orderIn.get(p) ?? 0), 0);
          return sum / prereqs.length;
        };
        return bary(a) - bary(b);
      });
    }
    ids.forEach((id, i) => orderIn.set(id, i));
  }

  const maxCols = Math.max(1, ...layers.map((l) => l.length));
  const width = PAD * 2 + maxCols * NODE_W + (maxCols - 1) * H_GAP;
  const height = PAD * 2 + layers.length * NODE_H + (layers.length - 1) * V_GAP;

  const positioned: PositionedNode[] = [];
  const posById = new Map<string, PositionedNode>();
  for (let li = 0; li < layers.length; li++) {
    const ids = layers[li];
    const rowW = ids.length * NODE_W + (ids.length - 1) * H_GAP;
    const x0 = (width - rowW) / 2;
    const y = PAD + li * (NODE_H + V_GAP);
    ids.forEach((id, i) => {
      const p: PositionedNode = {
        node: byId.get(id)!,
        x: x0 + i * (NODE_W + H_GAP),
        y,
        w: NODE_W,
        h: NODE_H,
        layer: li,
      };
      positioned.push(p);
      posById.set(id, p);
    });
  }

  const edges: Edge[] = [];
  for (const p of positioned) {
    for (const pr of p.node.prerequisite_ids) {
      const src = posById.get(pr);
      if (!src) continue;
      edges.push({
        from: pr,
        to: p.node.id,
        x1: src.x + src.w / 2,
        y1: src.y + src.h,
        x2: p.x + p.w / 2,
        y2: p.y,
      });
    }
  }

  return { nodes: positioned, edges, width, height };
}
