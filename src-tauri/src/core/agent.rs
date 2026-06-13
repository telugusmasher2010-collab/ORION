use crate::core::native_http;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: String,
    pub current_task: Option<String>,
    pub tasks_completed: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskEntry {
    pub task: String,
    pub status: String,
    pub result: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentResponse {
    pub response: String,
    pub files_created: Vec<String>,
    pub commands: Vec<String>,
}

pub struct Agent {
    pub id: String,
    pub name: String,
    pub description: String,
    system_prompt: String,
    pub status: String,
    current_task: Option<String>,
    task_history: Vec<TaskEntry>,
}

fn now_iso() -> String {
    let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let total_secs = dur.as_secs();
    let millis = dur.subsec_millis();
    let days = total_secs / 86400;
    let time_secs = total_secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    let mut y = 1970i64;
    let mut remaining_days = days as i64;
    loop {
        let days_in_year = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 366 } else { 365 };
        if remaining_days < days_in_year { break; }
        remaining_days -= days_in_year;
        y += 1;
    }
    let month_days = [31, if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut m = 1u32;
    for &md in &month_days {
        if remaining_days < md as i64 { break; }
        remaining_days -= md as i64;
        m += 1;
    }
    let d = (remaining_days + 1) as u32;
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z", y, m, d, hours, minutes, seconds, millis)
}

impl Agent {
    pub fn new(id: &str, name: &str, description: &str, system_prompt: &str) -> Self {
        Agent {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            system_prompt: system_prompt.to_string(),
            status: "idle".into(),
            current_task: None,
            task_history: Vec::new(),
        }
    }

    pub fn execute(&mut self, task: &str, settings: &Value) -> Result<AgentResponse, String> {
        self.status = "working".into();
        self.current_task = Some(task.to_string());
        self.task_history.push(TaskEntry {
            task: task.to_string(),
            status: "started".into(),
            result: String::new(),
            timestamp: now_iso(),
        });

        let result = self.call_llm(task, settings);

        match result {
            Ok(response) => {
                self.status = "idle".into();
                self.current_task = None;
                self.log_task(task, "completed", &response);

                let files = if self.id == "coder" {
                    extract_files_from_response(&response)
                } else {
                    Vec::new()
                };
                let commands = extract_commands_from_response(&response);

                Ok(AgentResponse {
                    response,
                    files_created: files.into_iter().map(|f| f.path).collect(),
                    commands,
                })
            }
            Err(e) => {
                self.status = "error".into();
                self.current_task = None;
                self.log_task(task, "error", &e);
                Err(e)
            }
        }
    }

    fn call_llm(&self, task: &str, settings: &Value) -> Result<String, String> {
        let messages = vec![
            serde_json::json!({ "role": "system", "content": self.system_prompt }),
            serde_json::json!({ "role": "user", "content": task }),
        ];

        // Try local Ollama first (works offline, free, already configured)
        if let Ok(text) = try_ollama_non_streaming(&messages, settings) {
            return Ok(text);
        }

        // Cloud fallback chain: Groq → OpenRouter Claude → OpenRouter Grok → Gemini
        if let Ok(text) = try_groq_non_streaming(&messages, settings) {
            return Ok(text);
        }
        if let Ok(text) = try_openrouter_non_streaming(&messages, settings, "claude") {
            return Ok(text);
        }
        if let Ok(text) = try_openrouter_non_streaming(&messages, settings, "grok") {
            return Ok(text);
        }
        if let Ok(text) = try_gemini_non_streaming(&messages, settings) {
            return Ok(text);
        }
        Err("All cloud brains unavailable for agent execution".into())
    }

    fn log_task(&mut self, task: &str, status: &str, result: &str) {
        let truncated = |s: &str, max: usize| -> String {
            if s.len() > max { format!("{}...", &s[..max]) } else { s.to_string() }
        };
        self.task_history.push(TaskEntry {
            task: truncated(task, 200),
            status: status.to_string(),
            result: truncated(result, 500),
            timestamp: now_iso(),
        });
    }

    pub fn get_info(&self) -> AgentInfo {
        let completed = self.task_history.iter().filter(|t| t.status == "completed").count();
        AgentInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            status: self.status.clone(),
            current_task: self.current_task.clone(),
            tasks_completed: completed,
        }
    }
}

// ========================================
// LLM request helpers (non-streaming, native HTTP)
// ========================================

fn get_api_key(settings: &Value, path: &[&str]) -> Option<String> {
    let mut current = settings;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str().map(|s| s.to_string())
}

fn is_key_valid(key: &str) -> bool {
    !key.is_empty() && !key.contains("PASTE")
}

fn try_ollama_non_streaming(messages: &[Value], settings: &Value) -> Result<String, String> {
    let host = settings
        .get("ollama")
        .and_then(|o| o.get("host"))
        .and_then(|v| v.as_str())
        .unwrap_or("http://localhost:11434");
    let model = settings
        .get("ollama")
        .and_then(|o| o.get("models"))
        .and_then(|m| m.get("coder"))
        .and_then(|v| v.as_str())
        .unwrap_or("qwen2.5-coder:3b");

    let body = serde_json::json!({
        "model": model,
        "messages": messages,
        "stream": false,
    });
    let url = format!("{}/api/chat", host.trim_end_matches('/'));

    let raw = native_http::simple_post(&url, HashMap::new(), body)?;

    // Parse Ollama's JSON response to extract message.content
    if let Ok(json) = serde_json::from_str::<Value>(&raw) {
        if let Some(content) = json
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|v| v.as_str())
        {
            return Ok(content.to_string());
        }
    }
    // If JSON didn't parse or had no content, return raw text
    if !raw.is_empty() {
        return Ok(raw);
    }
    Err("Empty response from Ollama".into())
}

fn try_groq_non_streaming(messages: &[Value], settings: &Value) -> Result<String, String> {
    let api_key = get_api_key(settings, &["groq", "apiKey"]).ok_or_else(|| "no groq key".to_string())?;
    if !is_key_valid(&api_key) { return Err("groq key invalid".into()); }

    let model = settings.get("groq")
        .and_then(|g| g.get("model"))
        .and_then(|v| v.as_str())
        .unwrap_or("llama-3.3-70b-versatile");

    let body = serde_json::json!({ "model": model, "messages": messages, "stream": false });

    let mut headers = HashMap::new();
    headers.insert("Authorization".into(), format!("Bearer {}", api_key));

    native_http::simple_post("https://api.groq.com/openai/v1/chat/completions", headers, body)
}

fn try_openrouter_non_streaming(messages: &[Value], settings: &Value, variant: &str) -> Result<String, String> {
    let key_field = if variant == "grok" { "grokKey" } else { "claudeKey" };
    let model_field = if variant == "grok" { "grokModel" } else { "claudeModel" };
    let default_model = if variant == "grok" { "xai/grok-2" } else { "anthropic/claude-3.5-sonnet" };

    let api_key = get_api_key(settings, &["openrouter", key_field])
        .ok_or_else(|| format!("no openrouter {}", key_field))?;
    if !is_key_valid(&api_key) { return Err(format!("openrouter {} invalid", key_field)); }

    let model = settings.get("openrouter")
        .and_then(|o| o.get(model_field))
        .and_then(|v| v.as_str())
        .unwrap_or(default_model);

    let body = serde_json::json!({ "model": model, "messages": messages, "stream": false });

    let mut headers = HashMap::new();
    headers.insert("Authorization".into(), format!("Bearer {}", api_key));
    headers.insert("HTTP-Referer".into(), "https://orion.local".into());
    headers.insert("X-Title".into(), "ORION".into());

    native_http::simple_post("https://openrouter.ai/api/v1/chat/completions", headers, body)
}

fn try_gemini_non_streaming(messages: &[Value], settings: &Value) -> Result<String, String> {
    let api_key = get_api_key(settings, &["gemini", "apiKey"])
        .ok_or_else(|| "no gemini key".to_string())?;
    if !is_key_valid(&api_key) { return Err("gemini key invalid".into()); }

    let model = settings.get("gemini")
        .and_then(|g| g.get("model"))
        .and_then(|v| v.as_str())
        .unwrap_or("gemini-2.0-flash");

    let system_content = messages.iter()
        .find(|m| m.get("role").and_then(|v| v.as_str()) == Some("system"))
        .and_then(|m| m.get("content").and_then(|v| v.as_str()))
        .unwrap_or("");

    let user_text: String = messages.iter()
        .filter(|m| m.get("role").and_then(|v| v.as_str()) == Some("user"))
        .filter_map(|m| m.get("content").and_then(|v| v.as_str()))
        .collect::<Vec<_>>()
        .join("\n");

    let body = serde_json::json!({
        "systemInstruction": { "parts": [{ "text": system_content }] },
        "contents": [{ "parts": [{ "text": user_text }] }]
    });

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    native_http::simple_post(&url, HashMap::new(), body)
}

// ========================================
// Response parsing helpers
// ========================================

pub struct ExtractedFile {
    pub path: String,
    pub language: String,
    pub content: String,
}

pub fn extract_files_from_response(response: &str) -> Vec<ExtractedFile> {
    let mut files = Vec::new();
    let mut search_start = 0;

    while let Some(header_start) = response[search_start..].find("[FILE: ") {
        let abs_start = search_start + header_start;
        let after_header = &response[abs_start + 7..]; // after "[FILE: "

        // Find the closing ]
        let close_bracket = match after_header.find(']') {
            Some(pos) => pos,
            None => break,
        };

        let file_path = after_header[..close_bracket].trim().to_string();
        let content_start_abs = abs_start + 7 + close_bracket + 1;

        // Find code block after header
        let remaining = &response[content_start_abs..];
        if let Some(code_start) = remaining.find("```") {
            let after_opening = &remaining[code_start + 3..];
            // Skip optional language specifier (until newline)
            let after_lang = if let Some(nl) = after_opening.find('\n') {
                &after_opening[nl + 1..]
            } else {
                break;
            };

            // Find closing ```
            let closing = match after_lang.find("\n```") {
                Some(pos) => pos,
                None => break,
            };

            let language_end = after_opening.find('\n').unwrap_or(0);
            let language = after_opening[..language_end].trim().to_string();
            let content = after_lang[..closing].trim().to_string();

            files.push(ExtractedFile { path: file_path, language, content });

            // Advance past: ] + ``` + language_line + content + closing ```
            let block_end = code_start + 3 + language_end + 1 + closing + 4;
            search_start = content_start_abs + block_end;
        } else {
            break;
        }
    }

    files
}

pub fn extract_commands_from_response(response: &str) -> Vec<String> {
    let mut commands = Vec::new();
    let prefix = "[RUN]: ";
    let mut search_start = 0;

    while let Some(pos) = response[search_start..].find(prefix) {
        let abs_pos = search_start + pos;
        let after_prefix = &response[abs_pos + prefix.len()..];
        let line_end = after_prefix.find('\n').unwrap_or(after_prefix.len());
        commands.push(after_prefix[..line_end].trim().to_string());
        search_start = abs_pos + prefix.len() + line_end;
    }

    commands
}

pub fn save_file(workspace: &str, relative_path: &str, content: &str) -> Result<String, String> {
    let full_path = std::path::PathBuf::from(workspace).join(relative_path);
    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Create dir: {}", e))?;
    }
    std::fs::write(&full_path, content).map_err(|e| format!("Write: {}", e))?;
    Ok(full_path.to_string_lossy().to_string())
}
