//! Native Rust HTTP client — replaces Node.js http-bridge.js
//!
//! Provides streaming (SSE) and simple request helpers for all AI brain providers.
//! No Node.js or external process needed.

use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::time::Duration;

fn build_client() -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(90))
        .build()
        .map_err(|e| format!("HTTP client build: {}", e))
}

fn build_headers(custom: HashMap<String, String>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    for (k, v) in custom {
        if let (Ok(name), Ok(val)) = (HeaderName::from_str(&k), HeaderValue::from_str(&v)) {
            headers.insert(name, val);
        }
    }
    headers
}

/// Make a streaming POST request to an AI API endpoint.
///
/// Parses response as SSE JSON-lines (OpenAI, Gemini, Ollama formats),
/// calls `on_chunk` for each text fragment, and returns the full response text.
pub fn stream_request(
    url: &str,
    headers: HashMap<String, String>,
    body: Value,
    mut on_chunk: impl FnMut(&str),
) -> Result<String, String> {
    let client = build_client()?;
    let req_headers = build_headers(headers);

    let response = client
        .post(url)
        .headers(req_headers)
        .json(&body)
        .send()
        .map_err(|e| format!("HTTP request: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let text = response.text().unwrap_or_default();
        return Err(format!("HTTP {}: {}", status, text));
    }

    let reader = BufReader::new(response);
    let mut full_text = String::new();
    let mut line_count = 0usize;

    for line in reader.lines() {
        let line = line.map_err(|e| format!("Read line: {}", e))?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed == "data: [DONE]" {
            continue;
        }

        line_count += 1;

        // Parse SSE data: {...} or plain JSON {...}
        let data = if let Some(sse) = trimmed.strip_prefix("data: ") {
            sse.trim()
        } else {
            trimmed
        };

        if data.is_empty() {
            continue;
        }

        let json: Value = match serde_json::from_str(data) {
            Ok(j) => j,
            Err(_) => continue, // skip malformed lines
        };

        // Error from API
        if let Some(err) = json.get("error") {
            let msg = err.as_str().unwrap_or("API returned an error");
            // Some APIs wrap error in { "error": { "message": "..." } }
            let detailed = err.get("message").and_then(|m| m.as_str()).unwrap_or(msg);
            return Err(detailed.to_string());
        }

        // Extract text chunk from known AI response formats
        let chunk = extract_chunk(&json);
        if let Some(text) = chunk {
            if !text.is_empty() {
                full_text.push_str(text);
                on_chunk(text);
            }
        }
    }

    if line_count == 0 {
        return Err("Empty response (0 lines)".to_string());
    }

    if full_text.is_empty() {
        return Err("Empty response (no text chunks)".to_string());
    }

    Ok(full_text)
}

/// Make a non-streaming POST request and return the full response as raw JSON text.
pub fn simple_post(
    url: &str,
    headers: HashMap<String, String>,
    body: Value,
) -> Result<String, String> {
    let client = build_client()?;
    let req_headers = build_headers(headers);

    let response = client
        .post(url)
        .headers(req_headers)
        .json(&body)
        .send()
        .map_err(|e| format!("HTTP request: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().unwrap_or_default();
        return Err(format!("HTTP {}: {}", status, text));
    }

    response.text().map_err(|e| format!("Read response: {}", e))
}

/// Make a non-streaming GET request and return the full response as raw JSON text.
pub fn simple_get(url: &str) -> Result<String, String> {
    let client = build_client()?;

    let response = client
        .get(url)
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .send()
        .map_err(|e| format!("HTTP GET: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().unwrap_or_default();
        return Err(format!("HTTP GET {}: {}", status, text));
    }

    response.text().map_err(|e| format!("Read response: {}", e))
}

/// Extract text chunk from various AI response JSON formats.
fn extract_chunk(json: &Value) -> Option<&str> {
    // OpenAI-compatible (Groq, OpenRouter, Nvidia): choices[0].delta.content
    if let Some(choices) = json.get("choices").and_then(|a| a.as_array()) {
        if let Some(choice) = choices.first() {
            if let Some(text) = choice
                .get("delta")
                .and_then(|d| d.get("content"))
                .and_then(|v| v.as_str())
            {
                return Some(text);
            }
            if let Some(text) = choice.get("text").and_then(|v| v.as_str()) {
                return Some(text);
            }
        }
    }

    // Gemini: candidates[0].content.parts[0].text
    if let Some(candidates) = json.get("candidates").and_then(|a| a.as_array()) {
        if let Some(candidate) = candidates.first() {
            if let Some(text) = candidate
                .get("content")
                .and_then(|c| c.get("parts"))
                .and_then(|p| p.as_array())
                .and_then(|parts| parts.first())
                .and_then(|part| part.get("text"))
                .and_then(|v| v.as_str())
            {
                return Some(text);
            }
        }
    }

    // Ollama: message.content
    if let Some(text) = json
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|v| v.as_str())
    {
        return Some(text);
    }

    // Direct "chunk" field (bridge compatibility)
    if let Some(text) = json.get("chunk").and_then(|v| v.as_str()) {
        return Some(text);
    }

    None
}
