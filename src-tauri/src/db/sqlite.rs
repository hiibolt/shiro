use crate::db::GraphStore;
use crate::model::{Graph, MasteryStatus, Node};
use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use std::path::Path;
use std::str::FromStr;
use uuid::Uuid;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS graphs (
    id             TEXT PRIMARY KEY,
    title          TEXT NOT NULL,
    parent_node_id TEXT,
    created_at     TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS nodes (
    id               TEXT PRIMARY KEY,
    graph_id         TEXT NOT NULL,
    title            TEXT NOT NULL,
    description      TEXT NOT NULL,
    status           TEXT NOT NULL,
    prerequisite_ids TEXT NOT NULL DEFAULT '[]',
    subgraph_id      TEXT,
    FOREIGN KEY (graph_id) REFERENCES graphs(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_nodes_graph_id ON nodes(graph_id);
"#;

pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    pub async fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let opts = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
        let pool = SqlitePoolOptions::new()
            .max_connections(4)
            .connect_with(opts)
            .await
            .with_context(|| format!("opening sqlite at {}", path.display()))?;
        sqlx::query(SCHEMA).execute(&pool).await?;
        Ok(Self { pool })
    }
}

fn row_to_graph(row: &sqlx::sqlite::SqliteRow) -> Result<Graph> {
    let id: String = row.try_get("id")?;
    let title: String = row.try_get("title")?;
    let parent: Option<String> = row.try_get("parent_node_id")?;
    let created_at: String = row.try_get("created_at")?;
    Ok(Graph {
        id: Uuid::from_str(&id)?,
        title,
        parent_node_id: parent.map(|s| Uuid::from_str(&s)).transpose()?,
        created_at: chrono::DateTime::parse_from_rfc3339(&created_at)?.with_timezone(&chrono::Utc),
    })
}

fn row_to_node(row: &sqlx::sqlite::SqliteRow) -> Result<Node> {
    let id: String = row.try_get("id")?;
    let graph_id: String = row.try_get("graph_id")?;
    let title: String = row.try_get("title")?;
    let description: String = row.try_get("description")?;
    let status_json: String = row.try_get("status")?;
    let prereq_json: String = row.try_get("prerequisite_ids")?;
    let subgraph_id: Option<String> = row.try_get("subgraph_id")?;

    let status: MasteryStatus = serde_json::from_str(&status_json)?;
    let prereq_ids: Vec<Uuid> = serde_json::from_str(&prereq_json)?;

    Ok(Node {
        id: Uuid::from_str(&id)?,
        graph_id: Uuid::from_str(&graph_id)?,
        title,
        description,
        status,
        prerequisite_ids: prereq_ids,
        subgraph_id: subgraph_id.map(|s| Uuid::from_str(&s)).transpose()?,
    })
}

#[async_trait]
impl GraphStore for SqliteStore {
    async fn create_graph(&self, graph: &Graph) -> Result<()> {
        sqlx::query(
            "INSERT INTO graphs (id, title, parent_node_id, created_at) VALUES (?, ?, ?, ?)",
        )
        .bind(graph.id.to_string())
        .bind(&graph.title)
        .bind(graph.parent_node_id.map(|u| u.to_string()))
        .bind(graph.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_graph(&self, id: Uuid) -> Result<Option<Graph>> {
        let row = sqlx::query("SELECT * FROM graphs WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;
        row.as_ref().map(row_to_graph).transpose()
    }

    async fn delete_graph(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM graphs WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn list_root_graphs(&self) -> Result<Vec<Graph>> {
        let rows = sqlx::query(
            "SELECT * FROM graphs WHERE parent_node_id IS NULL ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(row_to_graph).collect()
    }

    async fn create_node(&self, node: &Node) -> Result<()> {
        sqlx::query(
            "INSERT INTO nodes (id, graph_id, title, description, status, prerequisite_ids, subgraph_id) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(node.id.to_string())
        .bind(node.graph_id.to_string())
        .bind(&node.title)
        .bind(&node.description)
        .bind(serde_json::to_string(&node.status)?)
        .bind(serde_json::to_string(&node.prerequisite_ids)?)
        .bind(node.subgraph_id.map(|u| u.to_string()))
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_node(&self, id: Uuid) -> Result<Option<Node>> {
        let row = sqlx::query("SELECT * FROM nodes WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;
        row.as_ref().map(row_to_node).transpose()
    }

    async fn list_nodes(&self, graph_id: Uuid) -> Result<Vec<Node>> {
        let rows = sqlx::query("SELECT * FROM nodes WHERE graph_id = ?")
            .bind(graph_id.to_string())
            .fetch_all(&self.pool)
            .await?;
        rows.iter().map(row_to_node).collect()
    }

    async fn update_node(&self, node: &Node) -> Result<()> {
        sqlx::query(
            "UPDATE nodes \
             SET title = ?, description = ?, status = ?, prerequisite_ids = ?, subgraph_id = ? \
             WHERE id = ?",
        )
        .bind(&node.title)
        .bind(&node.description)
        .bind(serde_json::to_string(&node.status)?)
        .bind(serde_json::to_string(&node.prerequisite_ids)?)
        .bind(node.subgraph_id.map(|u| u.to_string()))
        .bind(node.id.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete_node(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM nodes WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn reparent_children(&self, deleted_node: Uuid, new_prereqs: Vec<Uuid>) -> Result<()> {
        // Find every node that lists the deleted node as a prereq, then rewrite
        // its JSON prerequisite_ids array in a transaction.
        let mut tx = self.pool.begin().await?;
        let rows = sqlx::query("SELECT id, prerequisite_ids FROM nodes")
            .fetch_all(&mut *tx)
            .await?;
        let deleted_s = deleted_node.to_string();
        for row in rows {
            let prereq_json: String = row.try_get("prerequisite_ids")?;
            let mut prereqs: Vec<Uuid> = serde_json::from_str(&prereq_json)?;
            if !prereqs.iter().any(|p| p.to_string() == deleted_s) {
                continue;
            }
            prereqs.retain(|p| p.to_string() != deleted_s);
            for p in &new_prereqs {
                if !prereqs.contains(p) {
                    prereqs.push(*p);
                }
            }
            let id: String = row.try_get("id")?;
            sqlx::query("UPDATE nodes SET prerequisite_ids = ? WHERE id = ?")
                .bind(serde_json::to_string(&prereqs)?)
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }
}
