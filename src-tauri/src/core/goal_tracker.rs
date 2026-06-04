use chrono::{DateTime, Duration, Utc};

pub struct GoalTracker;

impl GoalTracker {
    pub fn get_active_goals() -> Vec<serde_json::Value> {
        Vec::new()
    }

    pub fn get_all_goals() -> Vec<serde_json::Value> {
        Vec::new()
    }

    pub fn get_overdue_goals(goals: &[serde_json::Value]) -> Vec<serde_json::Value> {
        let now = Utc::now();
        goals
            .iter()
            .filter(|goal| {
                if goal.get("status").and_then(|s| s.as_str()) != Some("active") {
                    return false;
                }
                let deadline_str = match goal.get("deadline").and_then(|d| d.as_str()) {
                    Some(s) => s,
                    None => return false,
                };
                match DateTime::parse_from_rfc3339(deadline_str) {
                    Ok(dt) => dt.with_timezone(&Utc) < now,
                    Err(_) => false,
                }
            })
            .cloned()
            .collect()
    }

    pub fn get_upcoming_deadlines(
        goals: &[serde_json::Value],
        days: i64,
    ) -> Vec<serde_json::Value> {
        let now = Utc::now();
        let future = now + Duration::days(days);
        let mut upcoming: Vec<serde_json::Value> = goals
            .iter()
            .filter(|goal| {
                if goal.get("status").and_then(|s| s.as_str()) != Some("active") {
                    return false;
                }
                let deadline_str = match goal.get("deadline").and_then(|d| d.as_str()) {
                    Some(s) => s,
                    None => return false,
                };
                match DateTime::parse_from_rfc3339(deadline_str) {
                    Ok(dt) => {
                        let dt_utc = dt.with_timezone(&Utc);
                        dt_utc >= now && dt_utc <= future
                    }
                    Err(_) => false,
                }
            })
            .cloned()
            .collect();
        upcoming.sort_by(|a, b| {
            let a_dl = a.get("deadline").and_then(|d| d.as_str()).unwrap_or("");
            let b_dl = b.get("deadline").and_then(|d| d.as_str()).unwrap_or("");
            a_dl.cmp(&b_dl)
        });
        upcoming
    }

    pub fn get_stats(goals: &[serde_json::Value]) -> serde_json::Value {
        let total = goals.len();
        let active = goals
            .iter()
            .filter(|g| g.get("status").and_then(|s| s.as_str()) == Some("active"))
            .count();
        let completed = goals
            .iter()
            .filter(|g| g.get("status").and_then(|s| s.as_str()) == Some("completed"))
            .count();
        let failed = goals
            .iter()
            .filter(|g| g.get("status").and_then(|s| s.as_str()) == Some("failed"))
            .count();
        let overdue = Self::get_overdue_goals(goals).len();
        let upcoming = Self::get_upcoming_deadlines(goals, 7).len();
        let completion_rate = if total > 0 {
            format!("{:.1}%", (completed as f64 / total as f64) * 100.0)
        } else {
            "0%".to_string()
        };
        serde_json::json!({
            "total": total,
            "active": active,
            "completed": completed,
            "failed": failed,
            "overdue": overdue,
            "upcoming": upcoming,
            "completionRate": completion_rate
        })
    }

    pub fn check_deadlines(goals: &[serde_json::Value]) -> Vec<serde_json::Value> {
        let mut reminders = Vec::new();
        let now = Utc::now();

        let overdue = Self::get_overdue_goals(goals);
        for goal in &overdue {
            reminders.push(serde_json::json!({
                "type": "overdue",
                "priority": "high",
                "message": format!(
                    "⚠️ Goal \"{}\" is overdue!",
                    goal.get("title").and_then(|t| t.as_str()).unwrap_or("Unknown")
                ),
                "goalId": goal.get("id"),
                "deadline": goal.get("deadline")
            }));
        }

        let upcoming = Self::get_upcoming_deadlines(goals, 1);
        for goal in &upcoming {
            let hours_left = goal
                .get("deadline")
                .and_then(|d| d.as_str())
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| {
                    let diff = dt.with_timezone(&Utc) - now;
                    let hours = diff.num_hours();
                    if hours < 0 {
                        0
                    } else {
                        hours
                    }
                })
                .unwrap_or(0);
            let priority = if hours_left < 6 { "high" } else { "medium" };
            reminders.push(serde_json::json!({
                "type": "upcoming",
                "priority": priority,
                "message": format!(
                    "📅 \"{}\" due in {} hours",
                    goal.get("title").and_then(|t| t.as_str()).unwrap_or("Unknown"),
                    hours_left
                ),
                "goalId": goal.get("id"),
                "deadline": goal.get("deadline")
            }));
        }

        reminders
    }

    pub fn get_todays_agenda(goals: &[serde_json::Value]) -> serde_json::Value {
        let active: Vec<serde_json::Value> = goals
            .iter()
            .filter(|g| g.get("status").and_then(|s| s.as_str()) == Some("active"))
            .cloned()
            .collect();
        let overdue = Self::get_overdue_goals(goals);
        let today = Self::get_upcoming_deadlines(goals, 1);
        serde_json::json!({
            "overdue": overdue,
            "today": today,
            "allActive": active,
            "hasUrgent": !overdue.is_empty() || !today.is_empty()
        })
    }
}
