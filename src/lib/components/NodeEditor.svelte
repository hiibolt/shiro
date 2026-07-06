<script lang="ts">
  import { api, type Node } from "$lib/api";
  import { store } from "$lib/store.svelte";
  import { fade, scale } from "svelte/transition";
  import { cubicOut } from "svelte/easing";

  interface Props {
    // Either editing an existing node, or creating within a graph.
    mode: "create" | "edit";
    graphId: string;
    existing?: Node | null;
    onClose: () => void;
    onSaved?: (n: Node) => void;
  }
  let { mode, graphId, existing, onClose, onSaved }: Props = $props();

  let title = $state("");
  let description = $state("");
  let prereqIds = $state<Set<string>>(new Set());
  $effect(() => {
    title = existing?.title ?? "";
    description = existing?.description ?? "";
    prereqIds = new Set(existing?.prerequisite_ids ?? []);
  });

  let titleInput: HTMLInputElement | null = $state(null);
  $effect(() => { titleInput?.focus(); });
  let saving = $state(false);
  let error = $state<string | null>(null);

  const siblings = $derived(
    (store.nodes[graphId] ?? []).filter((n) => n.id !== existing?.id)
  );

  function toggleP(id: string) {
    const next = new Set(prereqIds);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    prereqIds = next;
  }

  async function save() {
    if (!title.trim()) return;
    saving = true;
    error = null;
    try {
      const prereqs = Array.from(prereqIds);
      let saved: Node;
      if (mode === "create") {
        saved = await api.createNode(graphId, title.trim(), description.trim(), prereqs);
      } else if (existing) {
        saved = await api.updateNodeMeta(existing.id, title.trim(), description.trim(), prereqs);
      } else {
        return;
      }
      await store.refresh();
      onSaved?.(saved);
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
    aria-label={mode === "create" ? "Add node" : "Edit node"}
    tabindex="-1"
    transition:scale={{ start: 0.94, duration: 220, easing: cubicOut }}
  >
    <header>
      <h2>{mode === "create" ? "Add node" : "Edit node"}</h2>
      <button class="close" onclick={onClose} aria-label="Close">×</button>
    </header>

    <label class="field">
      <span>Title</span>
      <input type="text" bind:value={title} bind:this={titleInput} placeholder="e.g. Kernel bypass" />
    </label>

    <label class="field">
      <span>Description</span>
      <textarea rows="3" bind:value={description} placeholder="One-sentence summary of this concept."></textarea>
    </label>

    <div class="field">
      <span>Prerequisites</span>
      {#if siblings.length === 0}
        <div class="muted">No other nodes in this graph yet.</div>
      {:else}
        <div class="prereq-list">
          {#each siblings as s (s.id)}
            <label class="prereq" class:checked={prereqIds.has(s.id)}>
              <input
                type="checkbox"
                checked={prereqIds.has(s.id)}
                onchange={() => toggleP(s.id)}
              />
              <span>{s.title}</span>
            </label>
          {/each}
        </div>
      {/if}
    </div>

    {#if error}<p class="error">{error}</p>{/if}

    <footer>
      <button onclick={onClose} disabled={saving}>Cancel</button>
      <button class="primary" onclick={save} disabled={saving || !title.trim()}>
        {saving ? "Saving…" : mode === "create" ? "Add" : "Save"}
      </button>
    </footer>
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
    z-index: 200;
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
    gap: 14px;
  }
  header { display: flex; align-items: center; justify-content: space-between; }
  h2 { margin: 0; font-size: 1.05rem; }
  .close { background: transparent; border: none; color: #6b7180; font-size: 1.4rem; cursor: pointer; padding: 0 4px; line-height: 1; }
  .close:hover { color: #e7e9ee; }

  .field { display: flex; flex-direction: column; gap: 6px; }
  .field > span { font-size: 0.78rem; color: #c8ccd6; }
  input[type="text"], textarea {
    padding: 8px 10px;
    background: #0f1219;
    border: 1px solid #2a2f3c;
    border-radius: 6px;
    color: inherit;
    font: inherit;
    outline: none;
    resize: vertical;
  }
  input[type="text"]:focus, textarea:focus {
    border-color: #3a5da8;
    box-shadow: 0 0 0 3px rgba(138, 180, 255, 0.12);
  }
  .muted { color: #6b7180; font-size: 0.8rem; }

  .prereq-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-height: 180px;
    overflow-y: auto;
    padding: 4px;
    background: #0f1219;
    border: 1px solid #2a2f3c;
    border-radius: 6px;
  }
  .prereq {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.82rem;
    color: #c8ccd6;
    transition: background 140ms;
  }
  .prereq:hover { background: #1c2030; }
  .prereq.checked { color: #8ab4ff; }
  .prereq input { accent-color: #8ab4ff; }

  .error {
    margin: 0;
    padding: 8px 10px;
    background: rgba(229, 117, 117, 0.08);
    border: 1px solid rgba(229, 117, 117, 0.3);
    border-radius: 6px;
    font-size: 0.8rem;
    color: #e57575;
  }

  footer { display: flex; justify-content: flex-end; gap: 8px; }
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
