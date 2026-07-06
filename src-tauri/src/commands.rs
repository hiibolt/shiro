use crate::config::{build_llm, redact, AppConfig, LlmConfig};
use crate::llm::{VerificationQuestion, VerificationResult};
use crate::model::{Graph, MasteryStatus, Node};
use crate::service::Service;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct AppState {
    pub service: Arc<Service>,
    pub config: Mutex<AppConfig>,
    pub config_path: PathBuf,
}

fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

#[tauri::command]
pub async fn generate_starter_graph(
    state: State<'_, AppState>,
    goal: String,
) -> Result<Graph, String> {
    state.service.generate_starter_graph(&goal).await.map_err(err)
}

#[tauri::command]
pub async fn get_graph(
    state: State<'_, AppState>,
    graph_id: Uuid,
) -> Result<Option<Graph>, String> {
    state.service.get_graph(graph_id).await.map_err(err)
}

#[tauri::command]
pub async fn list_root_graphs(state: State<'_, AppState>) -> Result<Vec<Graph>, String> {
    state.service.list_root_graphs().await.map_err(err)
}

#[tauri::command]
pub async fn list_nodes(
    state: State<'_, AppState>,
    graph_id: Uuid,
) -> Result<Vec<Node>, String> {
    state.service.list_nodes(graph_id).await.map_err(err)
}

#[tauri::command]
pub async fn zoom_into_node(
    state: State<'_, AppState>,
    node_id: Uuid,
) -> Result<Graph, String> {
    state.service.zoom_into_node(node_id).await.map_err(err)
}

#[tauri::command]
pub async fn create_node(
    state: State<'_, AppState>,
    graph_id: Uuid,
    title: String,
    description: String,
    prerequisite_ids: Vec<Uuid>,
) -> Result<Node, String> {
    state
        .service
        .create_node(graph_id, title, description, prerequisite_ids)
        .await
        .map_err(err)
}

#[tauri::command]
pub async fn update_node_meta(
    state: State<'_, AppState>,
    node_id: Uuid,
    title: String,
    description: String,
    prerequisite_ids: Vec<Uuid>,
) -> Result<Node, String> {
    state
        .service
        .update_node_meta(node_id, title, description, prerequisite_ids)
        .await
        .map_err(err)
}

#[tauri::command]
pub async fn update_node_status(
    state: State<'_, AppState>,
    node_id: Uuid,
    status: MasteryStatus,
) -> Result<Node, String> {
    state.service.set_status(node_id, status).await.map_err(err)
}

#[tauri::command]
pub async fn delete_node(
    state: State<'_, AppState>,
    node_id: Uuid,
    orphan_children: bool,
) -> Result<(), String> {
    state
        .service
        .delete_node(node_id, orphan_children)
        .await
        .map_err(err)
}

#[tauri::command]
pub async fn request_verification(
    state: State<'_, AppState>,
    node_id: Uuid,
) -> Result<VerificationQuestion, String> {
    state.service.request_verification(node_id).await.map_err(err)
}

#[tauri::command]
pub async fn get_llm_config(state: State<'_, AppState>) -> Result<LlmConfig, String> {
    Ok(redact(&state.config.lock().await.llm))
}

#[tauri::command]
pub async fn set_llm_config(
    state: State<'_, AppState>,
    config: LlmConfig,
) -> Result<(), String> {
    // Preserve the existing api_key when the frontend echoes back "***".
    let merged = merge_secrets(&state.config.lock().await.llm, config);
    state.service.set_llm(build_llm(&merged)).await;
    let mut cfg = state.config.lock().await;
    cfg.llm = merged;
    cfg.save(&state.config_path).map_err(err)?;
    Ok(())
}

fn merge_secrets(current: &LlmConfig, incoming: LlmConfig) -> LlmConfig {
    match (current, incoming) {
        (
            LlmConfig::Anthropic { api_key: cur, .. },
            LlmConfig::Anthropic {
                api_key: incoming,
                model,
            },
        ) if incoming == "***" => LlmConfig::Anthropic {
            api_key: cur.clone(),
            model,
        },
        (_, incoming) => incoming,
    }
}

#[tauri::command]
pub async fn create_learning_script(
    state: State<'_, AppState>,
    node_id: Uuid,
) -> Result<String, String> {
    state
        .service
        .create_learning_script(node_id)
        .await
        .map_err(err)
}

#[tauri::command]
pub async fn submit_answer(
    state: State<'_, AppState>,
    node_id: Uuid,
    question: VerificationQuestion,
    answer: String,
) -> Result<VerificationResult, String> {
    state
        .service
        .submit_answer(node_id, question, answer)
        .await
        .map_err(err)
}
