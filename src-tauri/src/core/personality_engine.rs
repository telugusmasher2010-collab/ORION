use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeConfig {
    pub name: String,
    pub display: String,
    pub description: String,
    #[serde(rename = "addressUser")]
    pub address_user: String,
    pub voice: String,
    #[serde(rename = "argumentStrength")]
    pub argument_strength: String,
    pub proactive: bool,
    pub color: String,
    pub keywords: Vec<String>,
    #[serde(rename = "systemPrompt")]
    pub system_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModesFile {
    modes: HashMap<String, ModeConfig>,
    #[serde(default, rename = "defaultMode")]
    default_mode: String,
    #[serde(default = "default_arg_strength", rename = "argumentStrength")]
    argument_strength: String,
    #[serde(default = "default_proactive_level", rename = "proactiveLevel")]
    proactive_level: String,
}

fn default_arg_strength() -> String { "direct".into() }
fn default_proactive_level() -> String { "on-request".into() }

pub struct PersonalityEngine {
    system_rules: String,
    master_prompt: String,
    constraints: String,
    long_term_memory: String,
    startup_prompt: String,
    current_mode: String,
    modes: HashMap<String, ModeConfig>,
    argument_strength: String,
    proactive_level: String,
}

impl PersonalityEngine {
    pub fn new(orion_root: &std::path::Path) -> Self {
        let system_rules = Self::read_file(&orion_root.join("CORE/systems_rules.md.txt"));
        let master_prompt = Self::read_file(&orion_root.join("PROMPTS/master_prompt.md.txt"));
        let constraints = Self::read_file(&orion_root.join("CONFIG/constraints.md.txt"));
        let long_term_memory = Self::read_file(&orion_root.join("MEMORY/long_term_memory.md.txt"));
        let startup_prompt = Self::read_file(&orion_root.join("PROMPTS/startup_prompt.md.txt"));

        let (modes, default_mode, arg_str, pro_level) = Self::load_modes(&orion_root.join("CONFIG/modes.json"));

        PersonalityEngine {
            system_rules,
            master_prompt,
            constraints,
            long_term_memory,
            startup_prompt,
            current_mode: default_mode,
            modes,
            argument_strength: arg_str,
            proactive_level: pro_level,
        }
    }

    fn read_file(path: &PathBuf) -> String {
        std::fs::read_to_string(path).unwrap_or_default()
    }

    fn load_modes(path: &PathBuf) -> (HashMap<String, ModeConfig>, String, String, String) {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        if let Ok(modes_file) = serde_json::from_str::<ModesFile>(&content) {
            (modes_file.modes, modes_file.default_mode, modes_file.argument_strength, modes_file.proactive_level)
        } else {
            let mut defaults = HashMap::new();
            defaults.insert("orion".into(), ModeConfig {
                name: "ORION".into(),
                display: "⚙️ ORION".into(),
                description: "Default partner mode".into(),
                address_user: "Abhi".into(),
                voice: "default".into(),
                argument_strength: "direct".into(),
                proactive: true,
                color: "#00d4ff".into(),
                keywords: vec!["default".into(), "chat".into(), "help".into()],
                system_prompt: "You are ORION, Abhi's partner and friend.".into(),
            });
            (defaults, "orion".into(), "direct".into(), "on-request".into())
        }
    }

    pub fn detect_mode(&self, message: &str) -> &str {
        if message.trim().is_empty() {
            return &self.current_mode;
        }
        let msg_lower = message.to_lowercase();
        for (mode_name, mode_config) in &self.modes {
            for keyword in &mode_config.keywords {
                if msg_lower.contains(&keyword.to_lowercase()) {
                    return mode_name;
                }
            }
        }
        &self.current_mode
    }

    pub fn get_mode_config(&self, mode: &str) -> &ModeConfig {
        self.modes.get(mode).unwrap_or_else(|| {
            self.modes.get("orion").expect("orion mode must exist")
        })
    }

    pub fn get_current_mode(&self) -> &str {
        &self.current_mode
    }

    pub fn set_mode(&mut self, mode_name: &str) -> bool {
        if self.modes.contains_key(mode_name) {
            self.current_mode = mode_name.to_string();
            true
        } else {
            false
        }
    }

    pub fn get_mode_display(&self, mode: &str) -> String {
        self.get_mode_config(mode).display.clone()
    }

    pub fn get_mode_color(&self, mode: &str) -> String {
        self.get_mode_config(mode).color.clone()
    }

    pub fn get_mode_description(&self, mode: &str) -> String {
        self.get_mode_config(mode).description.clone()
    }

    pub fn get_user_address(&self) -> String {
        self.get_mode_config(&self.current_mode).address_user.clone()
    }

    pub fn get_argument_strength(&self) -> String {
        self.get_mode_config(&self.current_mode).argument_strength.clone()
    }

    pub fn is_proactive(&self) -> bool {
        self.get_mode_config(&self.current_mode).proactive
    }

    pub fn get_voice_profile(&self) -> String {
        self.get_mode_config(&self.current_mode).voice.clone()
    }

    pub fn build_system_prompt(&self, conversation_context: &str) -> String {
        let mode_config = self.get_mode_config(&self.current_mode);
        let user_address = &mode_config.address_user;
        let mut parts: Vec<String> = Vec::new();

        if !self.master_prompt.is_empty() {
            parts.push(self.master_prompt.replace("[dynamically inserted]", &mode_config.display));
        }

        if !mode_config.system_prompt.is_empty() {
            parts.push(mode_config.system_prompt.clone());
        }

        if !self.system_rules.is_empty() {
            parts.push(self.system_rules.clone());
        }

        parts.push(format!("Address the user as \"{}\".", user_address));

        if !conversation_context.is_empty() {
            parts.push(format!("CONTEXT:\n{}", conversation_context));
        }

        parts.join("\n\n").trim().to_string()
    }

    pub fn get_all_modes(&self) -> Vec<serde_json::Value> {
        let mut list = Vec::new();
        for (key, config) in &self.modes {
            list.push(serde_json::json!({
                "id": key,
                "name": config.name,
                "display": config.display,
                "description": config.description,
                "color": config.color,
                "active": key == &self.current_mode,
            }));
        }
        list
    }

    pub fn get_mode_context(&self) -> serde_json::Value {
        let config = self.get_mode_config(&self.current_mode);
        serde_json::json!({
            "mode": self.current_mode,
            "display": config.display,
            "argumentStrength": config.argument_strength,
            "isProactive": config.proactive,
            "addressUser": config.address_user,
            "voice": config.voice,
        })
    }
}
