//! AI client for Kaku's built-in chat overlay.
//!
//! Reads API config from `~/.config/kaku/assistant.toml` and provides
//! a synchronous streaming chat completion client (OpenAI-compatible API).
//!
//! Runs on a plain OS thread (inside overlay), so blocking I/O is fine.

use anyhow::{Context, Result};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

const DEFAULT_MODEL: &str = "gpt-5.4-mini";
const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";

/// Configuration loaded from `assistant.toml`.
#[derive(Clone)]
pub struct AssistantConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

impl AssistantConfig {
    pub fn load() -> Result<Self> {
        let path = assistant_toml_path()?;
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("Cannot read {}", path.display()))?;
        let parsed: toml::Value = raw.parse().context("Invalid assistant.toml")?;

        let api_key = parsed
            .get("api_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "api_key not set in {}. Run `kaku ai` to configure.",
                    path.display()
                )
            })?
            .to_string();

        let model = parsed
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or(DEFAULT_MODEL)
            .to_string();

        let base_url = parsed
            .get("base_url")
            .and_then(|v| v.as_str())
            .unwrap_or(DEFAULT_BASE_URL)
            .trim_end_matches('/')
            .to_string();

        Ok(Self {
            api_key,
            model,
            base_url,
        })
    }
}

fn assistant_toml_path() -> Result<PathBuf> {
    let user_config_path = config::user_config_path();
    let config_dir = user_config_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("invalid user config path"))?;
    Ok(config_dir.join("assistant.toml"))
}

/// A single message in the conversation.
#[derive(Clone)]
pub struct ApiMessage {
    pub role: &'static str,
    pub content: String,
}

impl ApiMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system",
            content: content.into(),
        }
    }
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user",
            content: content.into(),
        }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant",
            content: content.into(),
        }
    }
}

/// Synchronous AI client for use inside overlay threads.
/// Clone is cheap: reqwest::blocking::Client is Arc-backed internally.
#[derive(Clone)]
pub struct AiClient {
    config: AssistantConfig,
    client: reqwest::blocking::Client,
}

impl AiClient {
    pub fn new(config: AssistantConfig) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap_or_else(|e| {
                log::warn!("Failed to build HTTP client: {e}; falling back to default client");
                reqwest::blocking::Client::new()
            });
        Self { config, client }
    }

    /// Stream a chat completion. Calls `on_token` for each streamed token.
    /// Stops early if `cancelled` is set to `true`.
    /// Returns the complete response text on success.
    pub fn chat_stream(
        &self,
        messages: &[ApiMessage],
        cancelled: &AtomicBool,
        on_token: &mut dyn FnMut(&str),
    ) -> Result<String> {
        let url = format!("{}/chat/completions", self.config.base_url);

        let body = serde_json::json!({
            "model": self.config.model,
            "messages": messages.iter().map(|m| serde_json::json!({
                "role": m.role,
                "content": m.content,
            })).collect::<Vec<_>>(),
            "stream": true,
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .context("HTTP request failed")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            anyhow::bail!("API error {}: {}", status, body);
        }

        let reader = BufReader::new(response);
        let mut full = String::new();

        for line in reader.lines() {
            if cancelled.load(Ordering::Relaxed) {
                break;
            }
            let line = line.context("read SSE line")?;
            let Some(data) = line.strip_prefix("data: ") else {
                continue;
            };
            if data.trim() == "[DONE]" {
                break;
            }
            match serde_json::from_str::<serde_json::Value>(data) {
                Ok(chunk) => {
                    if let Some(token) = chunk
                        .get("choices")
                        .and_then(|c| c.get(0))
                        .and_then(|c| c.get("delta"))
                        .and_then(|d| d.get("content"))
                        .and_then(|v| v.as_str())
                    {
                        on_token(token);
                        full.push_str(token);
                    }
                }
                Err(e) => {
                    log::warn!("Failed to parse SSE chunk: {e} — data: {data}");
                }
            }
        }

        Ok(full)
    }
}
