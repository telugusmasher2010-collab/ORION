use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
struct SuggestionLog {
    message: String,
    suggestion_type: String,
    accepted: bool,
    timestamp: String,
}

pub struct SuggestionEngine {
    suggestion_history: Vec<SuggestionLog>,
}

impl SuggestionEngine {
    pub fn new() -> Self {
        Self {
            suggestion_history: Vec::new(),
        }
    }

    pub fn get_suggestions(
        &self,
        goals: &[serde_json::Value],
        follow_ups: &[serde_json::Value],
        active_task: Option<&str>,
        task_started_at: Option<&str>,
    ) -> Vec<serde_json::Value> {
        let mut suggestions = Vec::new();
        let now_secs = current_epoch_secs();

        for goal in goals {
            if let Some(deadline_str) = goal.get("deadline").and_then(|v| v.as_str()) {
                if let Some(deadline_secs) = parse_iso_to_secs(deadline_str) {
                    if deadline_secs > now_secs {
                        let hours_until = (deadline_secs - now_secs) as f64 / 3600.0;
                        if hours_until < 24.0 {
                            let title = goal
                                .get("title")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Goal");
                            suggestions.push(serde_json::json!({
                                "type": "deadline",
                                "priority": "high",
                                "message": format!("\u{23f0} \"{title}\" is due in {} hours!", hours_until.round() as i64),
                                "action": "complete_goal",
                                "goalId": goal.get("id")
                            }));
                        }
                    }
                }
            }
        }

        for f in follow_ups {
            if let Some(context) = f.get("context").and_then(|v| v.as_str()) {
                suggestions.push(serde_json::json!({
                    "type": "follow_up",
                    "priority": "medium",
                    "message": context,
                    "action": "handle_follow_up",
                    "followUpId": f.get("id")
                }));
            }
        }

        if let (Some(task), Some(started_at)) = (active_task, task_started_at) {
            if let Some(start_secs) = parse_iso_to_secs(started_at) {
                let minutes_active = (now_secs as f64 - start_secs as f64) / 60.0;
                if minutes_active > 120.0 {
                    suggestions.push(serde_json::json!({
                        "type": "break",
                        "priority": "low",
                        "message": format!(
                            "\u{1f634} You've been on \"{task}\" for {} minutes. Take a break?",
                            minutes_active.round() as i64
                        ),
                        "action": "suggest_break"
                    }));
                }
            }
        }

        suggestions
    }

    pub fn create_follow_up_from_message(&self, message: &str) -> Option<serde_json::Value> {
        let lower = message.to_lowercase();

        if lower.contains("remind me") {
            let words: Vec<&str> = lower.split_whitespace().collect();
            let mut minutes = 60u64;
            let mut found = false;

            for (i, word) in words.iter().enumerate() {
                if let Ok(amount) = word.parse::<u64>() {
                    if let Some(next) = words.get(i + 1) {
                        let unit = next.to_lowercase();
                        if unit.starts_with("minute") {
                            minutes = amount;
                            found = true;
                        } else if unit.starts_with("hour") {
                            minutes = amount * 60;
                            found = true;
                        } else if unit.starts_with("day") {
                            minutes = amount * 60 * 24;
                            found = true;
                        }
                        if found {
                            break;
                        }
                    }
                }
            }

            if found {
                return Some(serde_json::json!({
                    "followUpId": format!("fu_{}", current_epoch_secs()),
                    "message": format!("I'll remind you about: \"{}\" in {} minutes.", "check on this", minutes)
                }));
            }
        }

        let action_keywords = ["don't forget", "remember to", "remind me to"];
        for kw in &action_keywords {
            if let Some(idx) = lower.find(kw) {
                let action = lower[idx + kw.len()..].trim();
                if !action.is_empty() {
                    return Some(serde_json::json!({
                        "followUpId": format!("fu_{}", current_epoch_secs()),
                        "message": format!("I'll remind you about: \"{action}\" in 60 minutes.")
                    }));
                }
            }
        }

        None
    }

    pub fn suggest_break(&self, _reason: &str) -> serde_json::Value {
        let break_options = [
            "\u{2615} Quick coffee break?",
            "\u{1f6b6} 5-minute stretch break?",
            "\u{1f4a7} Get some water?",
            "\u{1f440} Rest your eyes for a minute?",
        ];

        let idx = (current_epoch_secs() as usize) % break_options.len();

        serde_json::json!({
            "type": "break",
            "priority": "low",
            "message": break_options[idx],
            "action": "take_break"
        })
    }

    pub fn log_suggestion(&mut self, message: &str, suggestion_type: &str, accepted: bool) {
        self.suggestion_history.push(SuggestionLog {
            message: message.to_string(),
            suggestion_type: suggestion_type.to_string(),
            accepted,
            timestamp: epoch_to_iso(current_epoch_secs()),
        });

        if self.suggestion_history.len() > 50 {
            let excess = self.suggestion_history.len() - 50;
            self.suggestion_history.drain(0..excess);
        }
    }

    pub fn get_stats(&self) -> serde_json::Value {
        let total = self.suggestion_history.len();
        let accepted = self.suggestion_history.iter().filter(|s| s.accepted).count();
        let rate = if total > 0 {
            (accepted as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        serde_json::json!({
            "totalSuggestions": total,
            "acceptedSuggestions": accepted,
            "acceptanceRate": format!("{:.1}%", rate)
        })
    }
}

fn current_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn epoch_to_iso(secs: u64) -> String {
    let secs_per_day = 86400u64;
    let days = secs / secs_per_day;
    let time_secs = secs % secs_per_day;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    let mut y = 1970i64;
    let mut remaining_days = days as i64;

    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        y += 1;
    }

    let months = [
        31,
        if is_leap(y) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m = 0usize;
    let mut d = remaining_days;
    for (i, &days_in_month) in months.iter().enumerate() {
        if d < days_in_month {
            m = i + 1;
            break;
        }
        d -= days_in_month;
    }
    if m == 0 {
        m = 12;
        d = months[11] - 1;
    }

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.000Z",
        y,
        m,
        d + 1,
        hours,
        minutes,
        seconds
    )
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn parse_iso_to_secs(iso: &str) -> Option<u64> {
    let s = iso.trim_end_matches('Z');
    let without_tz = s.split('+').next().unwrap_or(s);
    let trimmed = without_tz.trim_end_matches('Z');

    let no_fraction = if let Some(dot) = trimmed.find('.') {
        &trimmed[..dot]
    } else {
        trimmed
    };

    let parts: Vec<&str> = no_fraction.split('T').collect();
    if parts.len() != 2 {
        return None;
    }

    let date_parts: Vec<&str> = parts[0].split('-').collect();
    let time_parts: Vec<&str> = parts[1].split(':').collect();

    if date_parts.len() != 3 || time_parts.len() < 2 {
        return None;
    }

    let year: i64 = date_parts[0].parse().ok()?;
    let month: u64 = date_parts[1].parse().ok()?;
    let day: u64 = date_parts[2].parse().ok()?;
    let hour: u64 = time_parts[0].parse().ok()?;
    let minute: u64 = time_parts[1].parse().ok()?;
    let second: u64 = time_parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);

    let days = days_from_epoch(year, month, day)?;
    Some(days * 86400 + hour * 3600 + minute * 60 + second)
}

fn days_from_epoch(year: i64, month: u64, day: u64) -> Option<u64> {
    if month < 1 || month > 12 || day < 1 || day > 31 {
        return None;
    }

    let mut total_days: i64 = 0;
    for y in 1970..year {
        total_days += if is_leap(y) { 366 } else { 365 };
    }

    let months = [
        31,
        if is_leap(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    for m in 0..(month as usize - 1) {
        total_days += months[m] as i64;
    }

    total_days += day as i64 - 1;

    if total_days < 0 {
        return None;
    }

    Some(total_days as u64)
}
