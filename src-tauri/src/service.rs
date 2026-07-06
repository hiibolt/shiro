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
