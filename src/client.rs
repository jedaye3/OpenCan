use anyhow::{bail, Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;
use serde::Serialize;

use crate::config::Config;
use crate::model::Message;

#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [Message],
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: AssistantMessage,
}

#[derive(Debug, Deserialize)]
struct AssistantMessage {
    content: serde_json::Value,
}

pub fn chat_completion(config: &Config, messages: &[Message]) -> Result<String> {
    let key = config.api_key()?;
    let endpoint = format!(
        "{}/chat/completions",
        config.base_url.trim_end_matches('/')
    );

    let payload = ChatRequest {
        model: &config.model,
        messages,
        temperature: config.temperature,
    };

    let response = Client::new()
        .post(endpoint)
        .bearer_auth(key)
        .json(&payload)
        .send()
        .context("Failed to call model API")?
        .error_for_status()
        .context("Model API returned an error status")?;

    let body: ChatResponse = response
        .json()
        .context("Failed to parse model response JSON")?;

    let first = body
        .choices
        .into_iter()
        .next()
        .context("No completion choices returned")?;

    value_to_text(first.message.content)
}

fn value_to_text(value: serde_json::Value) -> Result<String> {
    match value {
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Array(items) => {
            let mut parts = Vec::new();
            for item in items {
                if let Some(text) = item.get("text").and_then(|x| x.as_str()) {
                    parts.push(text.to_string());
                }
            }
            if parts.is_empty() {
                bail!("Assistant response content array had no text fields");
            }
            Ok(parts.join("\n"))
        }
        _ => bail!("Unsupported assistant content format"),
    }
}
