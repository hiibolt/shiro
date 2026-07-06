use crate::llm::{
    CapabilityHint, GeneratedGraph, GeneratedNode, LlmProvider, VerificationQuestion,
    VerificationResult,
};
use anyhow::Result;
use async_trait::async_trait;

pub struct MockLlm;

#[async_trait]
impl LlmProvider for MockLlm {
    fn capability_hint(&self) -> CapabilityHint {
        CapabilityHint::Low
    }

    async fn generate_graph(&self, goal: &str) -> Result<GeneratedGraph> {
        Ok(GeneratedGraph {
            title: format!("Learn: {goal}"),
            nodes: vec![
                GeneratedNode {
                    title: "Foundations".into(),
                    description: format!("Core vocabulary and orientation for {goal}."),
                    prerequisite_titles: vec![],
                },
                GeneratedNode {
                    title: "Core concepts".into(),
                    description: format!("Central ideas that make {goal} tractable."),
                    prerequisite_titles: vec!["Foundations".into()],
                },
                GeneratedNode {
                    title: "Applied practice".into(),
                    description: format!("Small hands-on exercises applying {goal}."),
                    prerequisite_titles: vec!["Core concepts".into()],
                },
                GeneratedNode {
                    title: "Synthesis".into(),
                    description: format!("Combine sub-skills to solve real {goal} problems."),
                    prerequisite_titles: vec!["Applied practice".into()],
                },
            ],
        })
    }

    async fn zoom_node(
        &self,
        node_title: &str,
        _node_description: &str,
    ) -> Result<GeneratedGraph> {
        Ok(GeneratedGraph {
            title: format!("Deep dive: {node_title}"),
            nodes: vec![
                GeneratedNode {
                    title: format!("{node_title} — background"),
                    description: "Prerequisite background for this subtopic.".into(),
                    prerequisite_titles: vec![],
                },
                GeneratedNode {
                    title: format!("{node_title} — mechanics"),
                    description: "How it actually works, step by step.".into(),
                    prerequisite_titles: vec![format!("{node_title} — background")],
                },
                GeneratedNode {
                    title: format!("{node_title} — exercise"),
                    description: "A concrete exercise for mastery check.".into(),
                    prerequisite_titles: vec![format!("{node_title} — mechanics")],
                },
            ],
        })
    }

    async fn generate_verification_question(
        &self,
        node_title: &str,
        _node_description: &str,
    ) -> Result<VerificationQuestion> {
        Ok(VerificationQuestion {
            prompt: format!(
                "Explain the core idea of \"{node_title}\" in your own words, and give one example."
            ),
            expects: "requires derivation".into(),
        })
    }

    async fn grade_answer(
        &self,
        _question: &VerificationQuestion,
        user_answer: &str,
    ) -> Result<VerificationResult> {
        let passed = user_answer.split_whitespace().count() >= 15;
        Ok(VerificationResult {
            passed,
            feedback: if passed {
                "Answer covered the core idea with a concrete example.".into()
            } else {
                "Answer was too brief — try restating the core idea and giving an example.".into()
            },
            suggested_new_prereqs: vec![],
        })
    }

    async fn create_learning_script(&self, context: &str) -> Result<String> {
        Ok(format!(
            "# Mock coaching script\n\nYou are a tutor. Teach ONLY the target node below using Socratic questioning + Feynman rephrase check. Do not drift into sibling nodes. Keep iterating until the learner can explain it in their own words and solve a novel example.\n\n## Context\n\n{context}\n"
        ))
    }
}
