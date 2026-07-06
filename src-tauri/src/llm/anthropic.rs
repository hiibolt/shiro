use crate::llm::{
    CapabilityHint, GeneratedGraph, GeneratedNode, LlmProvider, VerificationQuestion,
    VerificationResult,
};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

// One fixed rubric per PLAN: provider swaps change fluency, never grading criteria.
const GRADE_SYSTEM: &str = r#"You are a rigorous but fair learning coach grading a short-answer response.

Grading criteria (identical across all providers):
- The answer must demonstrate genuine understanding, not restate the question.
- It must include at least one concrete example, derivation, or worked case.
- Vagueness, hand-waving, or memorized definitions without explanation FAIL.
- If the failure suggests a missing prerequisite concept, propose 1–3 prereq nodes that would help.

Respond with ONLY valid JSON matching this exact schema, no preamble, no code fences:
{
  "passed": boolean,
  "feedback": "one to three sentences of specific, actionable feedback",
  "suggested_new_prereqs": [
    { "title": string, "description": string, "prerequisite_titles": [string] }
  ]
}"#;

const GRAPH_SYSTEM: &str = r#"You design learning DAGs. Given a goal, produce a directed acyclic graph of 4–8 concept nodes.
- Each node has a title (2–5 words), a one-sentence description, and prerequisite_titles referencing other nodes in this same graph.
- The DAG must be acyclic and roughly linear-with-branches: at least one node has no prereqs (root), at least one node has 2+ prereqs (synthesis).
- Titles must be unique within the graph.

Respond with ONLY valid JSON matching this exact schema, no preamble, no code fences:
{
  "title": "short title for the whole graph",
  "nodes": [
    { "title": string, "description": string, "prerequisite_titles": [string] }
  ]
}"#;

const ZOOM_SYSTEM: &str = r#"You expand a single concept node into its own sub-DAG of 3–6 finer-grained subtopics.
- Each subtopic has a title, one-sentence description, and prerequisite_titles referencing other subtopics in this same subgraph.
- Do NOT reference the parent concept as a prereq. Sub-DAG must be self-contained.
- The sub-DAG must be acyclic. Titles unique within the subgraph.

Respond with ONLY valid JSON matching this exact schema, no preamble, no code fences:
{
  "title": "short title for the subgraph",
  "nodes": [
    { "title": string, "description": string, "prerequisite_titles": [string] }
  ]
}"#;

const QUESTION_SYSTEM: &str = r#"You write ONE verification question that probes whether a learner has genuinely internalized a concept.
- The question must require reasoning, derivation, or a worked example — not recall of a definition.
- Be specific to the concept; avoid generic prompts like "explain X in your own words".

Respond with ONLY valid JSON matching this exact schema, no preamble, no code fences:
{
  "prompt": "the question",
  "expects": "one of: requires derivation | requires code | requires numeric answer | requires worked example"
}"#;

pub struct AnthropicProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            client: reqwest::Client::new(),
        }
    }

    async fn call<T: for<'de> Deserialize<'de>>(&self, system: &str, user: String) -> Result<T> {
        let body = json!({
            "model": self.model,
            "max_tokens": 2048,
            "system": [{
                "type": "text",
                "text": system,
                "cache_control": { "type": "ephemeral" }
            }],
            "messages": [{ "role": "user", "content": user }],
        });

        let resp = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .context("anthropic request")?;

        let status = resp.status();
        let value: Value = resp.json().await.context("anthropic response body")?;
        if !status.is_success() {
            return Err(anyhow!("anthropic {}: {}", status, value));
        }

        let text = value
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.iter().find(|b| b.get("type").and_then(|t| t.as_str()) == Some("text")))
            .and_then(|b| b.get("text"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow!("no text block in anthropic response: {}", value))?;

        let cleaned = strip_fences(text);
        serde_json::from_str::<T>(cleaned)
            .with_context(|| format!("parsing model JSON: {cleaned}"))
    }
}

// The model occasionally wraps JSON in ```json ... ``` despite the system prompt.
fn strip_fences(s: &str) -> &str {
    let s = s.trim();
    if let Some(rest) = s.strip_prefix("```json") {
        return rest.trim().trim_end_matches("```").trim();
    }
    if let Some(rest) = s.strip_prefix("```") {
        return rest.trim().trim_end_matches("```").trim();
    }
    s
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn capability_hint(&self) -> CapabilityHint {
        CapabilityHint::High
    }

    async fn generate_graph(&self, goal: &str) -> Result<GeneratedGraph> {
        self.call(GRAPH_SYSTEM, format!("Goal: {goal}")).await
    }

    async fn zoom_node(
        &self,
        node_title: &str,
        node_description: &str,
    ) -> Result<GeneratedGraph> {
        self.call(
            ZOOM_SYSTEM,
            format!("Concept: {node_title}\nDescription: {node_description}"),
        )
        .await
    }

    async fn generate_verification_question(
        &self,
        node_title: &str,
        node_description: &str,
    ) -> Result<VerificationQuestion> {
        self.call(
            QUESTION_SYSTEM,
            format!("Concept: {node_title}\nDescription: {node_description}"),
        )
        .await
    }

    async fn grade_answer(
        &self,
        question: &VerificationQuestion,
        user_answer: &str,
    ) -> Result<VerificationResult> {
        // Also include a nested GeneratedNode fallback: model may return prereqs w/o
        // description or prerequisite_titles. Post-process to fill blanks.
        #[derive(Deserialize)]
        struct Raw {
            passed: bool,
            feedback: String,
            #[serde(default)]
            suggested_new_prereqs: Vec<RawPrereq>,
        }
        #[derive(Deserialize)]
        struct RawPrereq {
            title: String,
            #[serde(default)]
            description: String,
            #[serde(default)]
            prerequisite_titles: Vec<String>,
        }

        let raw: Raw = self
            .call(
                GRADE_SYSTEM,
                format!(
                    "Question: {}\nExpects: {}\nAnswer: {}",
                    question.prompt, question.expects, user_answer
                ),
            )
            .await?;

        Ok(VerificationResult {
            passed: raw.passed,
            feedback: raw.feedback,
            suggested_new_prereqs: raw
                .suggested_new_prereqs
                .into_iter()
                .map(|p| GeneratedNode {
                    title: p.title,
                    description: p.description,
                    prerequisite_titles: p.prerequisite_titles,
                })
                .collect(),
        })
    }
}
