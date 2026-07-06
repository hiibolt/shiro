use crate::db::GraphStore;
use crate::model::{Graph, Node};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Default)]
pub struct InMemoryStore {
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    graphs: HashMap<Uuid, Graph>,
    nodes: HashMap<Uuid, Node>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl GraphStore for InMemoryStore {
    async fn create_graph(&self, graph: &Graph) -> Result<()> {
        self.inner.lock().unwrap().graphs.insert(graph.id, graph.clone());
        Ok(())
    }
    async fn get_graph(&self, id: Uuid) -> Result<Option<Graph>> {
        Ok(self.inner.lock().unwrap().graphs.get(&id).cloned())
    }
    async fn delete_graph(&self, id: Uuid) -> Result<()> {
        let mut i = self.inner.lock().unwrap();
        i.graphs.remove(&id);
        i.nodes.retain(|_, n| n.graph_id != id);
        Ok(())
    }

    async fn list_root_graphs(&self) -> Result<Vec<Graph>> {
        let mut out: Vec<Graph> = self
            .inner
            .lock()
            .unwrap()
            .graphs
            .values()
            .filter(|g| g.parent_node_id.is_none())
            .cloned()
            .collect();
        out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(out)
    }

    async fn create_node(&self, node: &Node) -> Result<()> {
        self.inner.lock().unwrap().nodes.insert(node.id, node.clone());
        Ok(())
    }
    async fn get_node(&self, id: Uuid) -> Result<Option<Node>> {
        Ok(self.inner.lock().unwrap().nodes.get(&id).cloned())
    }
    async fn list_nodes(&self, graph_id: Uuid) -> Result<Vec<Node>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .nodes
            .values()
            .filter(|n| n.graph_id == graph_id)
            .cloned()
            .collect())
    }
    async fn update_node(&self, node: &Node) -> Result<()> {
        self.inner.lock().unwrap().nodes.insert(node.id, node.clone());
        Ok(())
    }
    async fn delete_node(&self, id: Uuid) -> Result<()> {
        self.inner.lock().unwrap().nodes.remove(&id);
        Ok(())
    }

    async fn reparent_children(
        &self,
        deleted_node: Uuid,
        new_prereqs: Vec<Uuid>,
    ) -> Result<()> {
        let mut i = self.inner.lock().unwrap();
        for node in i.nodes.values_mut() {
            if !node.prerequisite_ids.contains(&deleted_node) {
                continue;
            }
            node.prerequisite_ids.retain(|p| *p != deleted_node);
            for p in &new_prereqs {
                if !node.prerequisite_ids.contains(p) {
                    node.prerequisite_ids.push(*p);
                }
            }
        }
        Ok(())
    }
}
