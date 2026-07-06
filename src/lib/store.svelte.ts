import { api, type Graph, type Node } from "./api";

function makeStore() {
  let focus = $state<Graph | null>(null);
  let graphs = $state<Record<string, Graph>>({});
  let nodes = $state<Record<string, Node[]>>({});
  let rootGraphs = $state<Graph[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);

  async function refreshRoots() {
    rootGraphs = await api.listRootGraphs();
    for (const g of rootGraphs) graphs[g.id] = g;
  }

  async function loadNodes(graphId: string) {
    nodes[graphId] = await api.listNodes(graphId);
  }

  /**
   * Load the given graph's nodes plus one degree of subgraph previews —
   * enough for the deck-of-cards affordance on each parent node.
   */
  async function loadWithPreviews(graph: Graph) {
    graphs[graph.id] = graph;
    await loadNodes(graph.id);
    for (const n of nodes[graph.id] ?? []) {
      if (!n.subgraph_id) continue;
      if (!graphs[n.subgraph_id]) {
        const g = await api.getGraph(n.subgraph_id);
        if (g) graphs[g.id] = g;
      }
      await loadNodes(n.subgraph_id);
    }
  }

  function findGraphContainingNode(nodeId: string): string | null {
    for (const [gid, ns] of Object.entries(nodes)) {
      if (ns.some((n) => n.id === nodeId)) return gid;
    }
    return null;
  }

  return {
    get focus() { return focus; },
    get graphs() { return graphs; },
    get nodes() { return nodes; },
    get rootGraphs() { return rootGraphs; },
    get canZoomOut() { return !!focus?.parent_node_id; },
    get loading() { return loading; },
    get error() { return error; },

    async generate(goal: string) {
      loading = true;
      error = null;
      try {
        const g = await api.generateStarterGraph(goal);
        focus = g;
        await loadWithPreviews(g);
        await refreshRoots();
      } catch (e) {
        error = String(e);
      } finally {
        loading = false;
      }
    },

    async ensureNodesLoaded(graphId: string) {
      if (nodes[graphId]) return;
      await loadNodes(graphId);
    },

    async ensureGraphLoaded(graphId: string): Promise<Graph | null> {
      if (graphs[graphId]) return graphs[graphId];
      const g = await api.getGraph(graphId);
      if (g) graphs[g.id] = g;
      return g;
    },

    async openGraph(g: Graph) {
      if (focus?.id === g.id) return;
      loading = true;
      try {
        focus = g;
        await loadWithPreviews(g);
      } catch (e) {
        error = String(e);
      } finally {
        loading = false;
      }
    },

    async zoom(nodeId: string) {
      loading = true;
      try {
        const child = await api.zoomIntoNode(nodeId);
        // Refresh current focus's cache — its zoomed node now has subgraph_id set.
        if (focus) await loadNodes(focus.id);
        focus = child;
        await loadWithPreviews(child);
      } catch (e) {
        error = String(e);
      } finally {
        loading = false;
      }
    },

    async zoomOut() {
      if (!focus?.parent_node_id) return;
      const parentGraphId = findGraphContainingNode(focus.parent_node_id);
      if (!parentGraphId) return;
      const g = graphs[parentGraphId] ?? (await api.getGraph(parentGraphId));
      if (!g) return;
      focus = g;
      await loadWithPreviews(g);
    },

    async refresh() {
      if (focus) await loadWithPreviews(focus);
    },

    async restoreMostRecent() {
      loading = true;
      try {
        await refreshRoots();
        if (rootGraphs.length === 0) return;
        focus = rootGraphs[0];
        await loadWithPreviews(rootGraphs[0]);
      } catch (e) {
        error = String(e);
      } finally {
        loading = false;
      }
    },
  };
}

export const store = makeStore();
