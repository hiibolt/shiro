use crate::db::GraphStore;
use crate::llm::{GeneratedGraph, LlmProvider, VerificationQuestion, VerificationResult};
use crate::model::{Graph, MasteryStatus, Node};
use anyhow::{anyhow, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct Service {
    pub store: Arc<dyn GraphStore>,
    pub llm: RwLock<Arc<dyn LlmProvider>>,
}

impl Service {
    async fn llm(&self) -> Arc<dyn LlmProvider> {
        self.llm.read().await.clone()
    }
    pub async fn set_llm(&self, llm: Arc<dyn LlmProvider>) {
        *self.llm.write().await = llm;
    }
}

impl Service {
    pub async fn generate_starter_graph(&self, goal: &str) -> Result<Graph> {
        let generated = self.llm().await.generate_graph(goal).await?;
        let graph = Graph {
            id: Uuid::new_v4(),
            title: generated.title.clone(),
            parent_node_id: None,
            created_at: Utc::now(),
        };
        self.store.create_graph(&graph).await?;
        self.insert_generated_nodes(&graph, &generated).await?;
        Ok(graph)
    }

    pub async fn zoom_into_node(&self, node_id: Uuid) -> Result<Graph> {
        let mut parent = self
            .store
            .get_node(node_id)
            .await?
            .ok_or_else(|| anyhow!("node not found"))?;
        if let Some(existing) = parent.subgraph_id {
            if let Some(g) = self.store.get_graph(existing).await? {
                return Ok(g);
            }
        }
        let generated = self.llm().await.zoom_node(&parent.title, &parent.description).await?;
        let subgraph = Graph {
            id: Uuid::new_v4(),
            title: generated.title.clone(),
            parent_node_id: Some(parent.id),
            created_at: Utc::now(),
        };
        self.store.create_graph(&subgraph).await?;
        self.insert_generated_nodes(&subgraph, &generated).await?;
        parent.subgraph_id = Some(subgraph.id);
        self.store.update_node(&parent).await?;
        Ok(subgraph)
    }

    pub async fn list_nodes(&self, graph_id: Uuid) -> Result<Vec<Node>> {
        self.store.list_nodes(graph_id).await
    }

    pub async fn get_graph(&self, graph_id: Uuid) -> Result<Option<Graph>> {
        self.store.get_graph(graph_id).await
    }

    pub async fn list_root_graphs(&self) -> Result<Vec<Graph>> {
        self.store.list_root_graphs().await
    }

    pub async fn create_node(
        &self,
        graph_id: Uuid,
        title: String,
        description: String,
        prerequisite_ids: Vec<Uuid>,
    ) -> Result<Node> {
        // Sanity: prereqs must belong to the same graph.
        let siblings = self.store.list_nodes(graph_id).await?;
        let sib_ids: std::collections::HashSet<Uuid> = siblings.iter().map(|n| n.id).collect();
        let clean_prereqs: Vec<Uuid> = prerequisite_ids
            .into_iter()
            .filter(|p| sib_ids.contains(p))
            .collect();
        let node = Node {
            id: Uuid::new_v4(),
            graph_id,
            title,
            description,
            status: MasteryStatus::Unknown,
            prerequisite_ids: clean_prereqs,
            subgraph_id: None,
        };
        self.store.create_node(&node).await?;
        Ok(node)
    }

    pub async fn update_node_meta(
        &self,
        node_id: Uuid,
        title: String,
        description: String,
        prerequisite_ids: Vec<Uuid>,
    ) -> Result<Node> {
        let mut node = self
            .store
            .get_node(node_id)
            .await?
            .ok_or_else(|| anyhow!("node not found"))?;
        // Filter prereqs to siblings, and prevent self-reference.
        let siblings = self.store.list_nodes(node.graph_id).await?;
        let sib_ids: std::collections::HashSet<Uuid> =
            siblings.iter().map(|n| n.id).filter(|id| *id != node_id).collect();
        let clean_prereqs: Vec<Uuid> = prerequisite_ids
            .into_iter()
            .filter(|p| sib_ids.contains(p))
            .collect();
        node.title = title;
        node.description = description;
        node.prerequisite_ids = clean_prereqs;
        self.store.update_node(&node).await?;
        Ok(node)
    }

    pub async fn set_status(&self, node_id: Uuid, status: MasteryStatus) -> Result<Node> {
        let mut node = self
            .store
            .get_node(node_id)
            .await?
            .ok_or_else(|| anyhow!("node not found"))?;
        node.status = status;
        self.store.update_node(&node).await?;
        Ok(node)
    }

    pub async fn delete_node(&self, node_id: Uuid, orphan_children: bool) -> Result<()> {
        let node = self
            .store
            .get_node(node_id)
            .await?
            .ok_or_else(|| anyhow!("node not found"))?;
        let new_prereqs = if orphan_children {
            vec![]
        } else {
            node.prerequisite_ids.clone()
        };
        self.store.reparent_children(node_id, new_prereqs).await?;
        self.store.delete_node(node_id).await?;
        Ok(())
    }

    pub async fn request_verification(&self, node_id: Uuid) -> Result<VerificationQuestion> {
        let node = self
            .store
            .get_node(node_id)
            .await?
            .ok_or_else(|| anyhow!("node not found"))?;
        self.llm()
            .await
            .generate_verification_question(&node.title, &node.description)
            .await
    }

    pub async fn submit_answer(
        &self,
        node_id: Uuid,
        question: VerificationQuestion,
        answer: String,
    ) -> Result<VerificationResult> {
        let result = self.llm().await.grade_answer(&question, &answer).await?;
        if result.passed {
            self.set_status(
                node_id,
                MasteryStatus::Mastered {
                    verified_at: Utc::now(),
                },
            )
            .await?;
        } else {
            self.set_status(node_id, MasteryStatus::Learning).await?;
        }
        Ok(result)
    }

    pub async fn create_learning_script(&self, node_id: Uuid) -> Result<String> {
        let target = self
            .store
            .get_node(node_id)
            .await?
            .ok_or_else(|| anyhow!("node not found"))?;

        // Sibling graph: all nodes in the target's graph.
        let siblings = self.store.list_nodes(target.graph_id).await?;
        let graph = self
            .store
            .get_graph(target.graph_id)
            .await?
            .ok_or_else(|| anyhow!("graph not found"))?;

        #[derive(serde::Serialize)]
        struct NodeLite<'a> {
            id: Uuid,
            title: &'a str,
            description: &'a str,
            prerequisite_ids: &'a [Uuid],
            is_target: bool,
        }
        let graph_json: Vec<NodeLite> = siblings
            .iter()
            .map(|n| NodeLite {
                id: n.id,
                title: &n.title,
                description: &n.description,
                prerequisite_ids: &n.prerequisite_ids,
                is_target: n.id == target.id,
            })
            .collect();
        let graph_json_str = serde_json::to_string_pretty(&graph_json)?;

        // Walk up parent chain: graph.parent_node_id -> node -> its graph -> ...
        #[derive(serde::Serialize)]
        struct ParentContext {
            depth: usize,
            graph_title: String,
            parent_node_title: String,
            parent_node_description: String,
            graph_nodes: Vec<ParentNodeLite>,
        }
        #[derive(serde::Serialize)]
        struct ParentNodeLite {
            title: String,
            description: String,
        }

        let mut chain: Vec<ParentContext> = Vec::new();
        let mut cur_graph = graph.clone();
        let mut depth = 1;
        while let Some(pnid) = cur_graph.parent_node_id {
            let pnode = match self.store.get_node(pnid).await? {
                Some(n) => n,
                None => break,
            };
            let pgraph = match self.store.get_graph(pnode.graph_id).await? {
                Some(g) => g,
                None => break,
            };
            let pnodes = self.store.list_nodes(pnode.graph_id).await?;
            chain.push(ParentContext {
                depth,
                graph_title: pgraph.title.clone(),
                parent_node_title: pnode.title.clone(),
                parent_node_description: pnode.description.clone(),
                graph_nodes: pnodes
                    .iter()
                    .map(|n| ParentNodeLite {
                        title: n.title.clone(),
                        description: n.description.clone(),
                    })
                    .collect(),
            });
            depth += 1;
            cur_graph = pgraph;
        }
        let parent_chain_str = if chain.is_empty() {
            "(this graph is a top-level root — no enclosing context)".to_string()
        } else {
            serde_json::to_string_pretty(&chain)?
        };

        let context = format!(
            "# TARGET NODE (the one and only concept to teach)\n\
             Title: {target_title}\n\
             Description: {target_desc}\n\
             ID: {target_id}\n\n\
             # CURRENT GRAPH — \"{graph_title}\"\n\
             The target lives inside this graph. All other nodes here are BACKGROUND context only.\n\
             ```json\n{graph_json}\n```\n\n\
             # PARENT CHAIN (immediate parent first, outermost last)\n\
             Each entry is a graph that contains the previous one via a parent node. Use ONLY for big-picture framing.\n\
             ```json\n{parent_chain}\n```\n",
            target_title = target.title,
            target_desc = target.description,
            target_id = target.id,
            graph_title = graph.title,
            graph_json = graph_json_str,
            parent_chain = parent_chain_str,
        );

        self.llm().await.create_learning_script(&context).await
    }

    async fn insert_generated_nodes(
        &self,
        graph: &Graph,
        generated: &GeneratedGraph,
    ) -> Result<Vec<Node>> {
        let mut title_to_id: HashMap<String, Uuid> = HashMap::new();
        for gn in &generated.nodes {
            title_to_id.insert(gn.title.clone(), Uuid::new_v4());
        }
        let mut inserted = Vec::new();
        for gn in &generated.nodes {
            let id = title_to_id[&gn.title];
            let prereq_ids = gn
                .prerequisite_titles
                .iter()
                .filter_map(|t| title_to_id.get(t).copied())
                .collect();
            let node = Node {
                id,
                graph_id: graph.id,
                title: gn.title.clone(),
                description: gn.description.clone(),
                status: MasteryStatus::Unknown,
                prerequisite_ids: prereq_ids,
                subgraph_id: None,
            };
            self.store.create_node(&node).await?;
            inserted.push(node);
        }
        Ok(inserted)
    }
}
