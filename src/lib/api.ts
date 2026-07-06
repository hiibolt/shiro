import { invoke } from "@tauri-apps/api/core";

export type MasteryStatus =
  | { type: "Unknown" }
  | { type: "Learning" }
  | { type: "Mastered"; verified_at: string };

export interface Graph {
  id: string;
  title: string;
  parent_node_id: string | null;
  created_at: string;
}

export interface Node {
  id: string;
  graph_id: string;
  title: string;
  description: string;
  status: MasteryStatus;
  prerequisite_ids: string[];
  subgraph_id: string | null;
}

export interface VerificationQuestion {
  prompt: string;
  expects: string;
}

export type LlmConfig =
  | { type: "Mock" }
  | { type: "Anthropic"; api_key: string; model: string }
  | { type: "Ollama"; host: string; model: string };

export interface VerificationResult {
  passed: boolean;
  feedback: string;
  suggested_new_prereqs: { title: string; description: string; prerequisite_titles: string[] }[];
}

export const api = {
  generateStarterGraph: (goal: string) => invoke<Graph>("generate_starter_graph", { goal }),
  getGraph: (graphId: string) => invoke<Graph | null>("get_graph", { graphId }),
  listRootGraphs: () => invoke<Graph[]>("list_root_graphs"),
  listNodes: (graphId: string) => invoke<Node[]>("list_nodes", { graphId }),
  zoomIntoNode: (nodeId: string) => invoke<Graph>("zoom_into_node", { nodeId }),
  updateNodeStatus: (nodeId: string, status: MasteryStatus) =>
    invoke<Node>("update_node_status", { nodeId, status }),
  createNode: (graphId: string, title: string, description: string, prerequisiteIds: string[]) =>
    invoke<Node>("create_node", { graphId, title, description, prerequisiteIds }),
  updateNodeMeta: (nodeId: string, title: string, description: string, prerequisiteIds: string[]) =>
    invoke<Node>("update_node_meta", { nodeId, title, description, prerequisiteIds }),
  deleteNode: (nodeId: string, orphanChildren: boolean) =>
    invoke<void>("delete_node", { nodeId, orphanChildren }),
  requestVerification: (nodeId: string) =>
    invoke<VerificationQuestion>("request_verification", { nodeId }),
  submitAnswer: (nodeId: string, question: VerificationQuestion, answer: string) =>
    invoke<VerificationResult>("submit_answer", { nodeId, question, answer }),
  createLearningScript: (nodeId: string) =>
    invoke<string>("create_learning_script", { nodeId }),
  getLlmConfig: () => invoke<LlmConfig>("get_llm_config"),
  setLlmConfig: (config: LlmConfig) => invoke<void>("set_llm_config", { config }),
};
