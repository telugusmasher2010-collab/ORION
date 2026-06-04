use serde_json::Value;
use std::collections::HashMap;

struct CacheEntry {
    value: String,
    category: String,
}

pub struct UserProfile {
    db: &'static crate::db::Database,
    preferences: HashMap<String, String>,
    cache: HashMap<String, CacheEntry>,
    goals: Vec<Value>,
    projects: Vec<Value>,
}

impl UserProfile {
    pub fn new(db: &'static crate::db::Database) -> Self {
        let mut preferences = HashMap::new();
        preferences.insert("communication_style".into(), "direct".into());
        preferences.insert("code_style".into(), "modern".into());
        preferences.insert("response_length".into(), "medium".into());
        preferences.insert("use_telugu".into(), "true".into());
        preferences.insert("argument_strength".into(), "direct".into());
        preferences.insert("voice_enabled".into(), "false".into());

        let mut profile = UserProfile {
            db,
            preferences,
            cache: HashMap::new(),
            goals: Vec::new(),
            projects: Vec::new(),
        };
        profile.load_profile();
        profile
    }

    pub fn load_profile(&mut self) {
        let all = self.db.get_user_profile().unwrap_or_default();
        if !all.is_empty() {
            for entry in &all {
                self.cache.insert(
                    entry.key.clone(),
                    CacheEntry {
                        value: entry.value.clone(),
                        category: entry.category.clone(),
                    },
                );
            }
            self._parse_stored_preferences();
        } else {
            self._initialize_defaults();
        }

        if let Some(json) = self.get("goals") {
            if let Ok(v) = serde_json::from_str::<Value>(&json) {
                if let Some(arr) = v.as_array() {
                    self.goals = arr.clone();
                }
            }
        }
        if let Some(json) = self.get("projects") {
            if let Ok(v) = serde_json::from_str::<Value>(&json) {
                if let Some(arr) = v.as_array() {
                    self.projects = arr.clone();
                }
            }
        }
    }

    fn _parse_stored_preferences(&mut self) {
        for key in [
            "communication_style",
            "code_style",
            "use_telugu",
            "argument_strength",
            "voice_enabled",
            "response_length",
        ] {
            if let Some(entry) = self.cache.get(key) {
                self.preferences.insert(key.to_string(), entry.value.clone());
            }
        }
    }

    fn _initialize_defaults(&mut self) {
        self.set("name", "Abhi", "general");
        self.set("preferred_language", "en", "general");
        self.set("communication_style", "direct", "preference");
        self.set("code_style", "modern", "preference");
        self.set("timezone", "Asia/Kolkata", "general");
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.cache.get(key).map(|e| e.value.clone())
    }

    pub fn set(&mut self, key: &str, value: &str, category: &str) {
        let _ = self.db.set_profile(key, value, category);
        self.cache.insert(
            key.to_string(),
            CacheEntry {
                value: value.to_string(),
                category: category.to_string(),
            },
        );
    }

    pub fn get_preferences(&self) -> HashMap<String, String> {
        self.preferences.clone()
    }

    pub fn update_preferences(&mut self, updates: HashMap<String, String>) {
        for (key, value) in &updates {
            self.preferences.insert(key.clone(), value.clone());
            self.set(key, value, "preference");
        }
    }

    pub fn learn_from_interaction(&mut self, user_message: &str, orion_response: &str) {
        self._learn_communication_pattern(user_message);
        self._learn_code_preferences(user_message);
        self._learn_response_preferences(orion_response);
    }

    fn _learn_communication_pattern(&mut self, message: &str) {
        let has_telugu_chars = message.chars().any(|c| ('\u{0C00}'..='\u{0C7F}').contains(&c));
        let telugu_words = ["ey Raysin", "vunte", "kadhu", "maa"];
        let has_telugu_words = telugu_words.iter().any(|w| message.contains(w));
        let uses_telugu = has_telugu_chars || has_telugu_words;

        let current = self.get("use_telugu");
        let expected = if uses_telugu { "true" } else { "false" };
        if current.as_deref() != Some(expected) {
            self.set("use_telugu", expected, "preference");
        }

        if message.len() < 50 {
            self.set("preferred_response_length", "short", "preference");
        } else if message.len() > 200 {
            self.set("preferred_response_length", "long", "preference");
        }
    }

    fn _learn_code_preferences(&mut self, message: &str) {
        let code_indicators = ["code", "function", "bug", "api", "react", "node", "python"];
        let msg_lower = message.to_lowercase();
        if code_indicators.iter().any(|k| msg_lower.contains(k)) {
            let techs = [
                ("react", "react"),
                ("vue", "vue"),
                ("angular", "angular"),
                ("node", "node"),
                ("python", "python"),
                ("javascript", "javascript"),
                ("typescript", "typescript"),
                ("go", "go"),
                ("rust", "rust"),
            ];
            for (pat, tech) in &techs {
                if msg_lower.contains(pat) {
                    self.set("preferred_tech", tech, "preference");
                    break;
                }
            }
        }
    }

    fn _learn_response_preferences(&mut self, response: &str) {
        if response.contains("```") {
            self.set("prefers_code_blocks", "true", "preference");
        }
        if response.contains("NEXT ACTION") || response.contains("Next step") {
            self.set("prefers_action_items", "true", "preference");
        }
    }

    pub fn get_profile_for_context(&self) -> Value {
        serde_json::json!({
            "name": self.get("name").unwrap_or_else(|| "Abhi".to_string()),
            "preferences": self.preferences,
            "goals": self.goals,
            "projects": self.projects,
        })
    }

    pub fn get_user_info(&self) -> Value {
        serde_json::json!({
            "name": self.get("name").unwrap_or_else(|| "Abhi".to_string()),
            "timezone": self.get("timezone").unwrap_or_else(|| "Asia/Kolkata".to_string()),
            "preferences": self.preferences,
        })
    }

    pub fn add_project(&mut self, name: &str, description: &str) {
        let now = chrono::Utc::now().to_rfc3339();
        self.projects.push(serde_json::json!({
            "name": name,
            "description": description,
            "createdAt": now,
            "status": "active",
        }));
        self.set(
            "projects",
            &serde_json::to_string(&self.projects).unwrap_or_default(),
            "project",
        );
    }

    pub fn update_project_status(&mut self, name: &str, status: &str) {
        let now = chrono::Utc::now().to_rfc3339();
        for project in &mut self.projects {
            if project.get("name").and_then(|n| n.as_str()) == Some(name) {
                project["status"] = Value::String(status.to_string());
                project["updatedAt"] = Value::String(now.clone());
                break;
            }
        }
        self.set(
            "projects",
            &serde_json::to_string(&self.projects).unwrap_or_default(),
            "project",
        );
    }

    pub fn get_active_projects(&self) -> Vec<&Value> {
        self.projects
            .iter()
            .filter(|p| p.get("status").and_then(|s| s.as_str()) == Some("active"))
            .collect()
    }

    pub fn add_goal(&mut self, title: &str, deadline: Option<&str>) {
        let now = chrono::Utc::now().to_rfc3339();
        self.goals.push(serde_json::json!({
            "title": title,
            "deadline": deadline,
            "createdAt": now,
            "status": "active",
        }));
        self.set(
            "goals",
            &serde_json::to_string(&self.goals).unwrap_or_default(),
            "goal",
        );
    }

    pub fn get_active_goals(&self) -> Vec<&Value> {
        self.goals
            .iter()
            .filter(|g| g.get("status").and_then(|s| s.as_str()) == Some("active"))
            .collect()
    }

    pub fn complete_goal(&mut self, title: &str) {
        let now = chrono::Utc::now().to_rfc3339();
        for goal in &mut self.goals {
            if goal.get("title").and_then(|t| t.as_str()) == Some(title) {
                goal["status"] = Value::String("completed".to_string());
                goal["completedAt"] = Value::String(now.clone());
                break;
            }
        }
        self.set(
            "goals",
            &serde_json::to_string(&self.goals).unwrap_or_default(),
            "goal",
        );
    }

    pub fn set_user_info(&mut self, key: &str, value: &str) {
        self.set(key, value, "user_info");
    }
}
