pub mod anthropic;
pub mod mock;
pub mod ollama;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GeneratedNode {
    pub title: String,
    pub description: String,
    pub prerequisite_titles: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GeneratedGraph {
    pub title: String,
    pub nodes: Vec<GeneratedNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationQuestion {
    pub prompt: String,
    /// e.g. "requires computed numeric answer", "requires code", "requires derivation"
    pub expects: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub passed: bool,
    pub feedback: String,
    pub suggested_new_prereqs: Vec<GeneratedNode>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CapabilityHint {
    Low,
    Medium,
    High,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn capability_hint(&self) -> CapabilityHint;

    async fn generate_graph(&self, goal: &str) -> Result<GeneratedGraph>;

    async fn zoom_node(
        &self,
        node_title: &str,
        node_description: &str,
    ) -> Result<GeneratedGraph>;

    async fn generate_verification_question(
        &self,
        node_title: &str,
        node_description: &str,
    ) -> Result<VerificationQuestion>;

    async fn grade_answer(
        &self,
        question: &VerificationQuestion,
        user_answer: &str,
    ) -> Result<VerificationResult>;
}
