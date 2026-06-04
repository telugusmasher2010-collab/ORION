use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainConfig {
    pub brain: String,
    pub model: String,
    pub label: String,
    pub source: String,
}

pub struct BrainRouter {
    pub settings: serde_json::Value,
}

impl BrainRouter {
    pub fn new(settings: serde_json::Value) -> Self {
        BrainRouter { settings }
    }

    pub fn is_internet_task(&self, message: &str) -> bool {
        let msg = message.to_lowercase();
        if self.is_code_task(message) {
            return false;
        }
        let keywords = [
            "search for", "browse the", "google", "look up", "find online",
            "check the web", "latest news", "current news", "trending now",
            "what is happening", "stock price", "weather today", "live score",
            "real-time", "recently released", "update on", "status of",
            "live data", "research on", "gather info about", "collect data from",
            "fetch from web", "scrape website", "current events",
            "what is the latest", "trending on", "check online",
            "niche", "famous", "trending", "popular", "market", "profit", "selling",
            "current", "latest", "recent", "news", "happening", "around", "india",
        ];
        keywords.iter().any(|k| msg.contains(k))
    }

    pub fn is_code_task(&self, message: &str) -> bool {
        let msg = message.to_lowercase();
        let keywords = [
            "code", "function", "script", "bug", "debug", "build", "website",
            "html", "css", "javascript", "python", "api", "database", "file",
            "edit", "create file", "project", "program", "deploy", "git",
            "github", "npm", "package", "install", "framework", "library",
            "component", "react", "vue", "node", "express", "endpoint",
            "write a", "make a", "create a", "fix the", "update the",
        ];
        keywords.iter().any(|k| msg.contains(k))
    }

    pub fn is_reasoning_task(&self, message: &str) -> bool {
        let msg = message.to_lowercase();
        let keywords = [
            "logic", "math", "calculate", "reason", "plan", "complex",
            "solve", "proof", "algorithm", "deduce", "analyze",
        ];
        keywords.iter().any(|k| msg.contains(k))
    }

    pub fn is_simple_task(&self, message: &str) -> bool {
        let msg = message.to_lowercase().trim().to_string();
        let word_count = msg.split_whitespace().count();
        if word_count <= 15 {
            return true;
        }
        let question_starters = [
            "what", "which", "who", "when", "where", "why", "how",
            "is", "are", "do", "does", "can", "will", "should",
        ];
        if let Some(first) = msg.split_whitespace().next() {
            if question_starters.contains(&first) {
                return true;
            }
        }
        message.len() < 25
    }

    pub fn route(&self, message: &str) -> BrainConfig {
        let routing = self.settings.get("routing");
        let local_first = routing
            .and_then(|r| r.get("localFirst"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let internet_cloud = routing
            .and_then(|r| r.get("internetTasksUseCloud"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if self.is_internet_task(message) && internet_cloud {
            if let Some(cfg) = self.try_openrouter_grok() {
                return cfg;
            }
            if let Some(cfg) = self.try_openrouter_claude() {
                return cfg;
            }
        }

        if local_first {
            if self.is_reasoning_task(message) {
                return self.ollama_brain("reasoning");
            }
            if self.is_simple_task(message) {
                return self.ollama_brain("fast");
            }
            if self.is_code_task(message) {
                return self.ollama_brain("coder");
            }
            return self.ollama_brain("manager");
        }

        self.get_fallback_brain()
    }

    fn ollama_brain(&self, task: &str) -> BrainConfig {
        let model = self
            .settings
            .get("ollama")
            .and_then(|o| o.get("models"))
            .and_then(|m| m.get(task))
            .and_then(|v| v.as_str())
            .unwrap_or("qwen3.5:latest");
        BrainConfig {
            brain: "ollama".into(),
            model: model.to_string(),
            label: format!("🧠 Ollama ({})", task),
            source: "local".into(),
        }
    }

    pub fn get_fallback_brain(&self) -> BrainConfig {
        if let Some(cfg) = self.try_groq() {
            return cfg;
        }
        if let Some(cfg) = self.try_openrouter_claude() {
            return cfg;
        }
        if let Some(cfg) = self.try_openrouter_grok() {
            return cfg;
        }
        if let Some(cfg) = self.try_gemini() {
            return cfg;
        }
        BrainConfig {
            brain: "error".into(),
            model: "none".into(),
            label: "❌ No Brain Available".into(),
            source: "none".into(),
        }
    }

    fn try_groq(&self) -> Option<BrainConfig> {
        let key = self
            .settings
            .get("groq")
            .and_then(|g| g.get("apiKey"))
            .and_then(|v| v.as_str())?;
        if key.contains("PASTE") {
            return None;
        }
        let model = self
            .settings
            .get("groq")
            .and_then(|g| g.get("model"))
            .and_then(|v| v.as_str())
            .unwrap_or("llama-3.3-70b-versatile");
        Some(BrainConfig {
            brain: "groq".into(),
            model: model.to_string(),
            label: "🚀 Groq".into(),
            source: "cloud".into(),
        })
    }

    fn try_openrouter_claude(&self) -> Option<BrainConfig> {
        let key = self
            .settings
            .get("openrouter")
            .and_then(|o| o.get("claudeKey"))
            .and_then(|v| v.as_str())?;
        if key.contains("PASTE") {
            return None;
        }
        let model = self
            .settings
            .get("openrouter")
            .and_then(|o| o.get("claudeModel"))
            .and_then(|v| v.as_str())
            .unwrap_or("anthropic/claude-3.5-sonnet");
        Some(BrainConfig {
            brain: "openrouter-claude".into(),
            model: model.to_string(),
            label: "🧠 Claude".into(),
            source: "cloud".into(),
        })
    }

    fn try_openrouter_grok(&self) -> Option<BrainConfig> {
        let key = self
            .settings
            .get("openrouter")
            .and_then(|o| o.get("grokKey"))
            .and_then(|v| v.as_str())?;
        if key.contains("PASTE") {
            return None;
        }
        let model = self
            .settings
            .get("openrouter")
            .and_then(|o| o.get("grokModel"))
            .and_then(|v| v.as_str())
            .unwrap_or("xai/grok-2");
        Some(BrainConfig {
            brain: "openrouter-grok".into(),
            model: model.to_string(),
            label: "🔍 Grok".into(),
            source: "cloud".into(),
        })
    }

    fn try_gemini(&self) -> Option<BrainConfig> {
        let key = self
            .settings
            .get("gemini")
            .and_then(|g| g.get("apiKey"))
            .and_then(|v| v.as_str())?;
        if key.contains("PASTE") {
            return None;
        }
        let model = self
            .settings
            .get("gemini")
            .and_then(|g| g.get("model"))
            .and_then(|v| v.as_str())
            .unwrap_or("gemini-2.0-flash");
        Some(BrainConfig {
            brain: "gemini".into(),
            model: model.to_string(),
            label: "️ Gemini".into(),
            source: "cloud".into(),
        })
    }
}
