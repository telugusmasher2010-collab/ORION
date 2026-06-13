use crate::core::native_http;
use serde_json::Value;
use std::collections::HashMap;

pub struct OllamaBrain {
    host: String,
    models: HashMap<String, String>,
    pub available: bool,
    pub available_models: Vec<String>,
}

impl Default for OllamaBrain {
    fn default() -> Self {
        let mut models = HashMap::new();
        models.insert("fast".into(), "qwen2.5-coder:1.5b".into());
        models.insert("coder".into(), "qwen2.5-coder:3b".into());
        models.insert("manager".into(), "qwen3:4b".into());
        models.insert("reasoning".into(), "deepseek-r1:7b".into());
        Self {
            host: "http://localhost:11434".into(),
            models,
            available: false,
            available_models: Vec::new(),
        }
    }
}

impl OllamaBrain {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn check_health(&mut self) -> Result<bool, String> {
        let url = format!("{}/api/tags", self.host);
        let raw = native_http::simple_get(&url)?;

        if raw.is_empty() {
            self.available = false;
            self.available_models.clear();
            return Ok(false);
        }

        if let Ok(json) = serde_json::from_str::<Value>(&raw) {
            if json.get("error").is_some() {
                self.available = false;
                self.available_models.clear();
                return Ok(false);
            }
            if let Some(models) = json.get("models").and_then(|v| v.as_array()) {
                self.available = true;
                self.available_models = models
                    .iter()
                    .filter_map(|m| m.get("name").and_then(|v| v.as_str()))
                    .map(|s| s.to_string())
                    .collect();
                return Ok(true);
            }
        }

        self.available = false;
        self.available_models.clear();
        Ok(false)
    }

    pub fn get_model_for_task(&self, task_type: &str) -> &str {
        match task_type {
            "fast" => self.models.get("fast").map_or("qwen2.5-coder:1.5b", |v| v),
            "coder" => self.models.get("coder").map_or("qwen2.5-coder:3b", |v| v),
            "reasoning" => self.models.get("reasoning").map_or("deepseek-r1:7b", |v| v),
            _ => self.models.get("manager").map_or("qwen3:4b", |v| v),
        }
    }

    pub fn chat(&self, messages: &[Value], model: &str) -> Result<String, String> {
        let url = format!("{}/api/chat", self.host);
        let body = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": false,
        });

        let raw = native_http::simple_post(&url, std::collections::HashMap::new(), body)?;

        if raw.is_empty() {
            return Err("Empty response from Ollama".into());
        }

        let json: Value =
            serde_json::from_str(&raw).map_err(|e| format!("JSON parse: {}", e))?;

        if let Some(err) = json.get("error").and_then(|v| v.as_str()) {
            return Err(err.to_string());
        }

        json.get("message")
            .and_then(|m| m.get("content"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "No content in response".into())
    }

    pub fn stream_chat(
        &self,
        messages: &[Value],
        model: &str,
        on_chunk: impl Fn(&str),
    ) -> Result<String, String> {
        let url = format!("{}/api/chat", self.host);
        let body = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": true,
        });

        native_http::stream_request(&url, std::collections::HashMap::new(), body, |chunk| {
            on_chunk(chunk);
        })
    }

    pub fn get_info(&self) -> Value {
        serde_json::json!({
            "available": self.available,
            "host": self.host,
            "models": self.models,
            "availableModels": self.available_models,
        })
    }
}
