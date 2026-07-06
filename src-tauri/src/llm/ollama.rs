use crate::llm::{
    CapabilityHint, GeneratedGraph, LlmProvider, VerificationQuestion, VerificationResult,
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;

/// Ollama provider stub. Config plumbing is wired end to end so users can pick
/// this provider in the UI; the actual HTTP integration is not implemented yet.
pub struct OllamaProvider {
    pub host: String,
    pub model: String,
}

impl OllamaProvider {
    pub fn new(host: String, model: String) -> Self {
        Self { host, model }
    }
}

fn todo() -> anyhow::Error {
    anyhow!("Ollama provider is not implemented yet — pick Anthropic or Mock in Settings.")
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn capability_hint(&self) -> CapabilityHint {
        CapabilityHint::Medium
    }
    async fn generate_graph(&self, _goal: &str) -> Result<GeneratedGraph> {
        Err(todo())
    }
    async fn zoom_node(&self, _title: &str, _description: &str) -> Result<GeneratedGraph> {
        Err(todo())
    }
    async fn generate_verification_question(
        &self,
        _title: &str,
        _description: &str,
    ) -> Result<VerificationQuestion> {
        Err(todo())
    }
    async fn grade_answer(
        &self,
        _question: &VerificationQuestion,
        _answer: &str,
    ) -> Result<VerificationResult> {
        Err(todo())
    }
    async fn create_learning_script(&self, _context: &str) -> Result<String> {
        Err(todo())
    }
}
