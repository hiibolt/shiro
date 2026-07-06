use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

use crate::llm::anthropic::AnthropicProvider;
use crate::llm::mock::MockLlm;
use crate::llm::ollama::OllamaProvider;
use crate::llm::LlmProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LlmConfig {
    Mock,
    Anthropic {
        api_key: String,
        #[serde(default = "default_anthropic_model")]
        model: String,
    },
    Ollama {
        #[serde(default = "default_ollama_host")]
        host: String,
        #[serde(default = "default_ollama_model")]
        model: String,
    },
}

fn default_anthropic_model() -> String {
    "claude-sonnet-4-6".into()
}
fn default_ollama_host() -> String {
    "http://localhost:11434".into()
}
fn default_ollama_model() -> String {
    "llama3.2".into()
}

impl Default for LlmConfig {
    fn default() -> Self {
        LlmConfig::Mock
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub llm: LlmConfig,
}

impl AppConfig {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading {}", path.display()))?;
        Ok(serde_json::from_str(&text).unwrap_or_default())
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let text = serde_json::to_string_pretty(self)?;
        std::fs::write(path, text).with_context(|| format!("writing {}", path.display()))?;
        Ok(())
    }
}

/// Sanitize a config for sending to the frontend — never expose secrets.
pub fn redact(cfg: &LlmConfig) -> LlmConfig {
    match cfg {
        LlmConfig::Anthropic { api_key, model } => LlmConfig::Anthropic {
            api_key: if api_key.is_empty() {
                String::new()
            } else {
                "***".into()
            },
            model: model.clone(),
        },
        other => other.clone(),
    }
}

pub fn build_llm(cfg: &LlmConfig) -> Arc<dyn LlmProvider> {
    match cfg {
        LlmConfig::Mock => Arc::new(MockLlm),
        LlmConfig::Anthropic { api_key, model } => {
            Arc::new(AnthropicProvider::new(api_key.clone(), model.clone()))
        }
        LlmConfig::Ollama { host, model } => {
            Arc::new(OllamaProvider::new(host.clone(), model.clone()))
        }
    }
}
