use std::time::{SystemTime, UNIX_EPOCH};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ContextMessage {
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub struct ActiveTask {
    pub task: String,
    pub started_at: String,
    pub status: String,
}

pub struct ContextManager {
    conversation_context: Vec<ContextMessage>,
    active_task: Option<ActiveTask>,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            conversation_context: Vec::new(),
            active_task: None,
        }
    }

    pub fn add_to_context(&mut self, role: &str, content: &str) {
        self.conversation_context.push(ContextMessage {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: iso_timestamp(),
        });
        if self.conversation_context.len() > 20 {
            self.conversation_context = self.conversation_context[self.conversation_context.len() - 20..].to_vec();
        }
    }

    pub fn get_context(&self, limit: usize) -> Vec<ContextMessage> {
        let len = self.conversation_context.len();
        if limit >= len {
            self.conversation_context.clone()
        } else {
            self.conversation_context[len - limit..].to_vec()
        }
    }

    pub fn clear_context(&mut self) {
        self.conversation_context.clear();
    }

    pub fn set_active_task(&mut self, task: &str) {
        self.active_task = Some(ActiveTask {
            task: task.to_string(),
            started_at: iso_timestamp(),
            status: "active".to_string(),
        });
    }

    pub fn get_active_task(&self) -> Option<ActiveTask> {
        self.active_task.clone()
    }

    pub fn clear_active_task(&mut self) {
        self.active_task = None;
    }

    pub fn update_task_progress(&mut self, progress: &str) {
        if let Some(ref mut task) = self.active_task {
            task.task = format!("{} [{}]", task.task, progress);
        }
    }

    pub fn complete_task(&mut self) {
        if let Some(ref mut task) = self.active_task {
            task.status = "completed".to_string();
        }
        self.active_task = None;
    }

    pub fn fail_task(&mut self, reason: &str) {
        if let Some(ref mut task) = self.active_task {
            task.status = format!("failed: {}", reason);
        }
        self.active_task = None;
    }

    pub fn extract_task_intent(&self, message: &str) -> Value {
        let lower = message.to_lowercase();

        let patterns: [(&str, &str); 6] = [
            ("i want to ", "creation"),
            ("can you ", "request"),
            ("help me ", "help"),
            ("i'm working on ", "progress"),
            ("finish ", "completion"),
            ("continue ", "continuation"),
        ];

        for (prefix, type_name) in &patterns {
            if lower.starts_with(prefix) {
                let detail = &message[prefix.len()..].trim();
                return serde_json::json!({
                    "type": type_name,
                    "detail": detail,
                });
            }
        }

        if lower.contains("stop") || lower.contains("pause") || lower.contains("hold") {
            return serde_json::json!({
                "type": "pause",
                "detail": message,
            });
        }

        serde_json::json!({
            "type": "general",
            "detail": message,
        })
    }

    pub fn build_context_summary(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        if let Some(ref task) = self.active_task {
            parts.push(format!("Active Task: {}", task.task));
        }

        if !self.conversation_context.is_empty() {
            parts.push(format!("Recent messages: {}", self.conversation_context.len()));
        }

        parts.join("\n")
    }
}

fn iso_timestamp() -> String {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let total_secs = dur.as_secs();
    let millis = dur.subsec_millis();

    let days = total_secs / 86400;
    let time_secs = total_secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    let (year, month, day) = days_to_date(days as i64);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        year, month, day, hours, minutes, seconds, millis
    )
}

fn days_to_date(mut days: i64) -> (i64, u32, u32) {
    let mut year: i64 = 1970;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }

    let month_days: [i64; 12] = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month: u32 = 0;
    for (i, md) in month_days.iter().enumerate() {
        if days < *md {
            month = (i + 1) as u32;
            break;
        }
        days -= *md;
    }

    let day = (days + 1) as u32;
    (year, month, day)
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}
