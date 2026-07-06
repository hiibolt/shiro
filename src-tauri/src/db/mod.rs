pub mod memory;
pub mod sqlite;

use crate::model::{Graph, Node};
use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait GraphStore: Send + Sync {
    async fn create_graph(&self, graph: &Graph) -> Result<()>;
    async fn get_graph(&self, id: Uuid) -> Result<Option<Graph>>;
    async fn delete_graph(&self, id: Uuid) -> Result<()>;
    /// Root graphs (no parent), newest first.
    async fn list_root_graphs(&self) -> Result<Vec<Graph>>;

    async fn create_node(&self, node: &Node) -> Result<()>;
    async fn get_node(&self, id: Uuid) -> Result<Option<Node>>;
    async fn list_nodes(&self, graph_id: Uuid) -> Result<Vec<Node>>;
    async fn update_node(&self, node: &Node) -> Result<()>;
    async fn delete_node(&self, id: Uuid) -> Result<()>;

    /// Re-link children when a node with children is deleted. `new_prereqs`
    /// is applied in place of `deleted_node` in every child's
    /// `prerequisite_ids`. Pass an empty vec to orphan the children.
    async fn reparent_children(
        &self,
        deleted_node: Uuid,
        new_prereqs: Vec<Uuid>,
    ) -> Result<()>;
}
