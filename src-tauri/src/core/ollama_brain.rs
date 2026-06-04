use crate::core::constants;
use serde_json::Value;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

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
        let (mut child, reader) = spawn_bridge("GET", &url, Value::Null, false)?;

        let lines: Vec<String> = reader
            .lines()
            .filter_map(|l| l.ok())
            .filter(|l| !l.is_empty())
            .collect();

        let _ = child.wait();

        if lines.is_empty() {
            self.available = false;
            self.available_models.clear();
            return Ok(false);
        }

        let raw = lines.join("");
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

        let (mut child, reader) = spawn_bridge("POST", &url, body, false)?;

        let lines: Vec<String> = reader
            .lines()
            .filter_map(|l| l.ok())
            .filter(|l| !l.is_empty())
            .collect();

        let _ = child.wait();

        if lines.is_empty() {
            return Err("Empty response from Ollama".into());
        }

        let raw = lines.join("");
        let json: Value = serde_json::from_str(&raw).map_err(|e| format!("JSON parse: {}", e))?;

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

        let (mut child, reader) = spawn_bridge("POST", &url, body, true)?;

        let mut full_text = String::new();
        for line in reader.lines() {
            let line = line.map_err(|e| format!("Read: {}", e))?;
            if line.is_empty() {
                continue;
            }
            if let Ok(json) = serde_json::from_str::<Value>(&line) {
                if let Some(err) = json.get("error").and_then(|v| v.as_str()) {
                    let _ = child.wait();
                    return Err(err.to_string());
                }
                if json.get("done").and_then(|v| v.as_bool()) == Some(true) {
                    break;
                }
                if let Some(chunk) = json.get("chunk").and_then(|v| v.as_str()) {
                    full_text.push_str(chunk);
                    on_chunk(chunk);
                }
            }
        }

        let _ = child.wait();
        Ok(full_text)
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

fn spawn_bridge(
    method: &str,
    url: &str,
    body: Value,
    stream: bool,
) -> Result<(std::process::Child, BufReader<std::process::ChildStdout>), String> {
    let req = serde_json::json!({
        "method": method,
        "url": url,
        "headers": {},
        "body": body,
        "stream": stream,
    });

    let input = serde_json::to_string(&req).map_err(|e| format!("JSON: {}", e))?;

    let mut child = Command::new(constants::NODE_PATH)
        .arg(constants::bridge_path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Spawn node: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .map_err(|e| format!("stdin: {}", e))?;
        let _ = stdin.flush();
    }

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "No stdout".to_string())?;

    Ok((child, BufReader::new(stdout)))
}
