use crate::core::brain_router::{BrainConfig, BrainRouter};
use crate::core::constants;
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use tauri::Emitter;

#[derive(serde::Serialize, Clone)]
pub struct ChatChunkPayload {
    pub chunk: String,
}

#[derive(serde::Serialize, Clone)]
pub struct ChatDonePayload {
    pub full_text: String,
    pub brain: String,
    pub model: String,
    pub label: String,
}

#[derive(serde::Serialize, Clone)]
pub struct ChatErrorPayload {
    pub error: String,
}

pub fn chat(
    app: &tauri::AppHandle,
    message: &str,
    _session_id: i64,
    settings: Value,
    history: &[Value],
    system_prompt: &str,
) -> Result<String, String> {
    let router = BrainRouter::new(settings.clone());
    let brain = router.route(message);

    if brain.brain == "error" {
        let err = "No AI brain available. Check your API keys in settings.json.".to_string();
        let _ = app.emit("chat_error", ChatErrorPayload { error: err.clone() });
        return Err(err);
    }

    let mut messages: Vec<Value> = Vec::new();
    messages.push(serde_json::json!({ "role": "system", "content": system_prompt }));
    for h in history {
        if let Some(role) = h.get("role").and_then(|v| v.as_str()) {
            if let Some(content) = h.get("content").and_then(|v| v.as_str()) {
                messages.push(serde_json::json!({ "role": role, "content": content }));
            }
        }
    }
    messages.push(serde_json::json!({ "role": "user", "content": message }));

    let result = match brain.brain.as_str() {
        "groq" => groq_request(&settings, &brain, &messages, app),
        "openrouter-claude" | "openrouter-grok" => {
            openrouter_request(&settings, &brain, &messages, app)
        }
        "gemini" => gemini_request(&settings, &brain, &messages, app),
        "ollama" => ollama_request(&settings, &brain, &messages, app),
        _ => groq_request(&settings, &brain, &messages, app),
    };

    match result {
        Ok(text) => {
            let _ = app.emit(
                "chat_done",
                ChatDonePayload {
                    full_text: text.clone(),
                    brain: brain.brain.clone(),
                    model: brain.model.clone(),
                    label: brain.label.clone(),
                },
            );
            Ok(text)
        }
        Err(e) => {
            let fb = try_fallback(app, &router, &brain, &e, &settings, &messages);
            if let Ok(text) = fb {
                return Ok(text);
            }
            let _ = app.emit("chat_error", ChatErrorPayload { error: e.clone() });
            Err(e)
        }
    }
}

fn try_fallback(
    app: &tauri::AppHandle,
    router: &BrainRouter,
    brain: &BrainConfig,
    primary_err: &str,
    settings: &Value,
    messages: &[Value],
) -> Result<String, String> {
    let fallback = router.get_fallback_brain();
    if fallback.brain == "error" || fallback.brain == brain.brain {
        return Err("No fallback".into());
    }

    let notify = format!(
        "Primary ({}) failed: {}. Trying fallback: {}",
        brain.label, primary_err, fallback.label
    );
    let _ = app.emit("chat_chunk", ChatChunkPayload { chunk: notify });

    let fb_result = match fallback.brain.as_str() {
        "groq" => groq_request(settings, &fallback, messages, app),
        "openrouter-claude" | "openrouter-grok" => {
            openrouter_request(settings, &fallback, messages, app)
        }
        "gemini" => gemini_request(settings, &fallback, messages, app),
        "ollama" => ollama_request(settings, &fallback, messages, app),
        _ => Err(format!("Fallback '{}' not impl", fallback.brain)),
    };

    match fb_result {
        Ok(text) => {
            let _ = app.emit(
                "chat_done",
                ChatDonePayload {
                    full_text: text.clone(),
                    brain: fallback.brain.clone(),
                    model: fallback.model.clone(),
                    label: format!("{} (fallback)", fallback.label),
                },
            );
            Ok(text)
        }
        Err(fb_err) => {
            let err = format!("Primary: {}. Fallback: {}", primary_err, fb_err);
            let _ = app.emit("chat_error", ChatErrorPayload { error: err.clone() });
            Err(err)
        }
    }
}
        }
    }

    let _ = child.wait();
    Ok(full_text)
}

fn ollama_request(
    settings: &Value,
    brain: &BrainConfig,
    messages: &[Value],
    app: &tauri::AppHandle,
) -> Result<String, String> {
    let body = serde_json::json!({
        "model": brain.model,
        "messages": messages,
        "stream": true,
    });

    let host = settings
        .get("ollama")
        .and_then(|o| o.get("host"))
        .and_then(|v| v.as_str())
        .unwrap_or("http://localhost:11434");

    let url = format!("{}/api/chat", host);
    let req = BridgeRequest {
        method: "POST".into(),
        url,
        headers: std::collections::HashMap::new(),
        body,
        stream: true,
    };
    let input = serde_json::to_string(&req).map_err(|e| format!("JSON: {}", e))?;

    let mut child = Command::new(constants::NODE_PATH)
        .arg(constants::BRIDGE_PATH)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Spawn node: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes()).map_err(|e| format!("stdin: {}", e))?;
        let _ = stdin.flush();
    }

    let stdout = child.stdout.take().ok_or_else(|| "No stdout".to_string())?;
    let reader = BufReader::new(stdout);
    let mut full_text = String::new();

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Read: {}", e))?;
        if line.is_empty() { continue; }
        if let Ok(json) = serde_json::from_str::<Value>(&line) {
            if let Some(err) = json.get("error").and_then(|v| v.as_str()) {
                return Err(err.to_string());
            }
            if json.get("done").and_then(|v| v.as_bool()) == Some(true) {
                break;
            }
            if let Some(chunk) = json.get("chunk").and_then(|v| v.as_str()) {
                full_text.push_str(chunk);
                let _ = app.emit("chat_chunk", ChatChunkPayload { chunk: chunk.to_string() });
            }
        }
    }

    let _ = child.wait();
    Ok(full_text)
}

// ========================================
// Node.js HTTP bridge
// ========================================
// Node.js HTTP bridge
// ========================================

#[derive(serde::Serialize)]
struct BridgeRequest {
    method: String,
    url: String,
    headers: std::collections::HashMap<String, String>,
    body: Value,
    stream: bool,
}

// ========================================
// Request builders (emit chunks during processing)
// ========================================

fn groq_request(
    settings: &Value,
    brain: &BrainConfig,
    messages: &[Value],
    app: &tauri::AppHandle,
) -> Result<String, String> {
    let api_key = settings
        .get("groq")
        .and_then(|g| g.get("apiKey"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Groq API key not found".to_string())?;

    if api_key.contains("PASTE") {
        return Err("Groq API key not configured".to_string());
    }

    let body = serde_json::json!({
        "model": brain.model,
        "messages": messages,
        "stream": true,
        "temperature": 0.7,
        "max_tokens": 8192,
    });

    let mut headers = std::collections::HashMap::new();
    headers.insert("Authorization".into(), format!("Bearer {}", api_key));

    let url = "https://api.groq.com/openai/v1/chat/completions".to_string();
    let req = BridgeRequest {
        method: "POST".into(),
        url,
        headers,
        body,
        stream: true,
    };
    let input = serde_json::to_string(&req).map_err(|e| format!("JSON: {}", e))?;

    let mut child = Command::new(constants::NODE_PATH)
        .arg(constants::BRIDGE_PATH)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Spawn node: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes()).map_err(|e| format!("stdin: {}", e))?;
        let _ = stdin.flush();
    }

    let stdout = child.stdout.take().ok_or_else(|| "No stdout".to_string())?;
    let reader = BufReader::new(stdout);
    let mut full_text = String::new();

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Read: {}", e))?;
        if line.is_empty() {
            continue;
        }
        if let Ok(json) = serde_json::from_str::<Value>(&line) {
            if let Some(err) = json.get("error").and_then(|v| v.as_str()) {
                return Err(err.to_string());
            }
            if json.get("done").and_then(|v| v.as_bool()) == Some(true) {
                break;
            }
            if let Some(chunk) = json.get("chunk").and_then(|v| v.as_str()) {
                full_text.push_str(chunk);
                let _ = app.emit("chat_chunk", ChatChunkPayload { chunk: chunk.to_string() });
            }
        }
    }

    let _ = child.wait();
    Ok(full_text)
}

fn openrouter_request(
    settings: &Value,
    brain: &BrainConfig,
    messages: &[Value],
    app: &tauri::AppHandle,
) -> Result<String, String> {
    let key_field = if brain.brain == "openrouter-claude" {
        "claudeKey"
    } else {
        "grokKey"
    };

    let api_key = settings
        .get("openrouter")
        .and_then(|o| o.get(key_field))
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("OpenRouter {} not found", key_field))?;

    if api_key.contains("PASTE") {
        return Err(format!("OpenRouter {} not configured", key_field));
    }

    let body = serde_json::json!({
        "model": brain.model,
        "messages": messages,
        "stream": true,
        "temperature": 0.7,
        "max_tokens": 8192,
    });

    let mut headers = std::collections::HashMap::new();
    headers.insert("Authorization".into(), format!("Bearer {}", api_key));
    headers.insert("HTTP-Referer".into(), "https://orion.local".into());
    headers.insert("X-Title".into(), "ORION".into());

    let url = "https://openrouter.ai/api/v1/chat/completions".to_string();
    let req = BridgeRequest {
        method: "POST".into(),
        url,
        headers,
        body,
        stream: true,
    };
    let input = serde_json::to_string(&req).map_err(|e| format!("JSON: {}", e))?;

    let mut child = Command::new(constants::NODE_PATH)
        .arg(constants::BRIDGE_PATH)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Spawn node: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes()).map_err(|e| format!("stdin: {}", e))?;
        let _ = stdin.flush();
    }

    let stdout = child.stdout.take().ok_or_else(|| "No stdout".to_string())?;
    let reader = BufReader::new(stdout);
    let mut full_text = String::new();

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Read: {}", e))?;
        if line.is_empty() {
            continue;
        }
        if let Ok(json) = serde_json::from_str::<Value>(&line) {
            if let Some(err) = json.get("error").and_then(|v| v.as_str()) {
                return Err(err.to_string());
            }
            if json.get("done").and_then(|v| v.as_bool()) == Some(true) {
                break;
            }
            if let Some(chunk) = json.get("chunk").and_then(|v| v.as_str()) {
                full_text.push_str(chunk);
                let _ = app.emit("chat_chunk", ChatChunkPayload { chunk: chunk.to_string() });
            }
        }
    }

    let _ = child.wait();
    Ok(full_text)
}

fn gemini_request(
    settings: &Value,
    brain: &BrainConfig,
    messages: &[Value],
    app: &tauri::AppHandle,
) -> Result<String, String> {
    let api_key = settings
        .get("gemini")
        .and_then(|g| g.get("apiKey"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Gemini API key not found".to_string())?;

    if api_key.contains("PASTE") {
        return Err("Gemini API key not configured".to_string());
    }

    let system_content = messages
        .iter()
        .find(|m| m.get("role").and_then(|v| v.as_str()) == Some("system"))
        .and_then(|m| m.get("content").and_then(|v| v.as_str()))
        .unwrap_or("");

    // Gemini expects alternating user/model turns, not all in one block
    let contents: Vec<Value> = messages
        .iter()
        .filter(|m| {
            let role = m.get("role").and_then(|v| v.as_str());
            role == Some("user") || role == Some("assistant")
        })
        .filter_map(|m| {
            let role = m.get("role").and_then(|v| v.as_str())?;
            let content = m.get("content").and_then(|v| v.as_str())?;
            let gemini_role = if role == "assistant" { "model" } else { "user" };
            Some(serde_json::json!({
                "role": gemini_role,
                "parts": [{ "text": content }]
            }))
        })
        .collect();

    let body = serde_json::json!({
        "systemInstruction": { "parts": [{ "text": system_content }] },
        "contents": contents
    });

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
        brain.model, api_key
    );

    let req = BridgeRequest {
        method: "POST".into(),
        url,
        headers: std::collections::HashMap::new(),
        body,
        stream: true,
    };
    let input = serde_json::to_string(&req).map_err(|e| format!("JSON: {}", e))?;

    let mut child = Command::new(constants::NODE_PATH)
        .arg(constants::BRIDGE_PATH)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Spawn node: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes()).map_err(|e| format!("stdin: {}", e))?;
        let _ = stdin.flush();
    }

    let stdout = child.stdout.take().ok_or_else(|| "No stdout".to_string())?;
    let reader = BufReader::new(stdout);
    let mut full_text = String::new();

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Read: {}", e))?;
        if let Ok(json) = serde_json::from_str::<Value>(&line) {
            if let Some(err) = json.get("error").and_then(|v| v.as_str()) {
                return Err(err.to_string());
            }
            if json.get("done").and_then(|v| v.as_bool()) == Some(true) {
                break;
            }
            if let Some(chunk) = json.get("chunk").and_then(|v| v.as_str()) {
                full_text.push_str(chunk);
                let _ = app.emit("chat_chunk", ChatChunkPayload { chunk: chunk.to_string() });
            }
        }
    }

    let _ = child.wait();
    Ok(full_text)
}
