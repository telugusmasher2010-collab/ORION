use chrono::Utc;
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub enum ArgumentStrength {
    Gentle,
    Direct,
    Stubborn,
}

impl ArgumentStrength {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "gentle" => ArgumentStrength::Gentle,
            "stubborn" => ArgumentStrength::Stubborn,
            "direct" | _ => ArgumentStrength::Direct,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ArgumentStrength::Gentle => "gentle",
            ArgumentStrength::Direct => "direct",
            ArgumentStrength::Stubborn => "stubborn",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArgumentLog {
    pub reason: String,
    pub response: String,
    pub arg_type: String,
    pub timestamp: String,
    pub strength: String,
    pub resolved: bool,
}

pub struct ArgumentEngine {
    strength: ArgumentStrength,
    argument_history: Vec<ArgumentLog>,
    persuasion_attempts: u32,
}

struct RiskyPattern {
    substrings: Vec<&'static str>,
    reason: &'static str,
    risk: &'static str,
}

const RISKY_PATTERNS: &[RiskyPattern] = &[
    RiskyPattern {
        substrings: vec!["just do it", "just build it", "just make it"],
        reason: "skipping planning",
        risk: "low",
    },
    RiskyPattern {
        substrings: vec!["i'll do it later", "i'll finish tomorrow"],
        reason: "procrastination",
        risk: "low",
    },
    RiskyPattern {
        substrings: vec!["it should be easy", "how hard can it be"],
        reason: "underestimating complexity",
        risk: "low",
    },
    RiskyPattern {
        substrings: vec!["buy now", "invest all"],
        reason: "financial risk",
        risk: "high",
    },
    RiskyPattern {
        substrings: vec!["delete everything", "remove all", "wipe"],
        reason: "data destruction",
        risk: "high",
    },
    RiskyPattern {
        substrings: vec!["give access", "grant permission"],
        reason: "security concern",
        risk: "medium",
    },
    RiskyPattern {
        substrings: vec!["no need to test", "skip testing"],
        reason: "quality risk",
        risk: "medium",
    },
    RiskyPattern {
        substrings: vec!["i know what i'm doing", "i'm sure"],
        reason: "overconfidence",
        risk: "low",
    },
    RiskyPattern {
        substrings: vec!["quick fix", "hack", "workaround"],
        reason: "technical debt",
        risk: "low",
    },
    RiskyPattern {
        substrings: vec!["copy paste", "from stackoverflow", "from github"],
        reason: "code quality",
        risk: "low",
    },
];

fn pseudo_random_index(max: usize) -> usize {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    (dur.as_nanos() as usize) % max
}

fn contains_any(input: &str, substrings: &[&str]) -> bool {
    let lower = input.to_lowercase();
    substrings.iter().any(|s| lower.contains(s))
}

impl ArgumentEngine {
    pub fn new(strength: &str) -> Self {
        ArgumentEngine {
            strength: ArgumentStrength::from_str(strength),
            argument_history: Vec::new(),
            persuasion_attempts: 0,
        }
    }

    pub fn set_strength(&mut self, strength: &str) {
        self.strength = ArgumentStrength::from_str(strength);
    }

    pub fn get_strength(&self) -> String {
        self.strength.as_str().to_string()
    }

    pub fn should_argue(&self, user_input: &str) -> Option<Value> {
        let lower = user_input.to_lowercase();

        for item in RISKY_PATTERNS {
            if contains_any(&lower, &item.substrings) {
                return Some(json!({
                    "shouldArgue": true,
                    "reason": item.reason,
                    "risk": item.risk,
                    "pattern": item.substrings[0]
                }));
            }
        }

        Some(json!({ "shouldArgue": false }))
    }

    pub fn generate_pushback(&mut self, issue: &Value) -> String {
        let address = "Abhi";
        let pushback = match self.strength {
            ArgumentStrength::Gentle => self.gentle_pushback(address, issue),
            ArgumentStrength::Stubborn => self.stubborn_pushback(address, issue),
            ArgumentStrength::Direct => self.direct_pushback(address, issue),
        };

        self.log_argument(issue, &pushback, "pushback");
        pushback
    }

    fn gentle_pushback(&self, address: &str, issue: &Value) -> String {
        let reason = issue["reason"].as_str().unwrap_or("unknown");
        let templates = vec![
            format!("{}, have you considered that {}?", address, reason),
            format!("{}, maybe think about why {} might be a problem.", address, reason),
            format!("{}, quick thought — {}. Worth checking before we proceed.", address, reason),
            format!("{}, not saying no, but {}. What do you think?", address, reason),
        ];
        let idx = pseudo_random_index(templates.len());
        templates[idx].clone()
    }

    fn direct_pushback(&self, address: &str, issue: &Value) -> String {
        let reason = issue["reason"].as_str().unwrap_or("unknown");
        let risk = issue["risk"].as_str().unwrap_or("low");
        let risk_warning = match risk {
            "high" => "This is HIGH RISK. ",
            "medium" => "This has some risk. ",
            _ => "",
        };

        let templates = vec![
            format!("{}, that's a bad idea because {}. {}Here's why: ", address, reason, risk_warning),
            format!("{}, I'm not letting you do that because {}. ", address, reason),
            format!("{}, stop. {} — that's a problem. Here's a better approach: ", address, reason),
            format!("{}, we need to talk about this. {}. Here's what we should do instead: ", address, reason),
        ];
        let idx = pseudo_random_index(templates.len());
        templates[idx].clone()
    }

    fn stubborn_pushback(&self, address: &str, issue: &Value) -> String {
        let reason = issue["reason"].as_str().unwrap_or("unknown");
        let templates = vec![
            format!("{}, I'm not proceeding until you explain why {} is worth the risk.", address, reason),
            format!("{}, I'm blocking this. {}. Convince me otherwise or we're doing it my way.", address, reason),
            format!("{}, no. {}. Tell me exactly how you'll handle that, or we're stopping here.", address, reason),
            format!("{}, I'm refusing this. {}. You need to explain the plan before I help.", address, reason),
        ];
        let idx = pseudo_random_index(templates.len());
        templates[idx].clone()
    }

    fn log_argument(&mut self, issue: &Value, response: &str, arg_type: &str) {
        let log = ArgumentLog {
            reason: issue["reason"].as_str().unwrap_or("unknown").to_string(),
            response: response.to_string(),
            arg_type: arg_type.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            strength: self.strength.as_str().to_string(),
            resolved: false,
        };
        self.argument_history.push(log);
        if self.argument_history.len() > 50 {
            self.argument_history = self.argument_history[self.argument_history.len() - 50..].to_vec();
        }
    }

    pub fn log_user_response(&mut self, user_response: &str) {
        if let Some(last) = self.argument_history.last_mut() {
            last.resolved = {
                let lower = user_response.to_lowercase();
                lower.contains("ok")
                    || lower.contains("fine")
                    || lower.contains("good point")
                    || lower.contains("yes")
            };
        }

        let resolved = self.argument_history.last().map(|a| a.resolved).unwrap_or(false);
        if !resolved && self.argument_history.len() > 0 {
            self.persuasion_attempts += 1;
        }
    }

    pub fn get_stats(&self) -> Value {
        let total = self.argument_history.len();
        let resolved = self.argument_history.iter().filter(|a| a.resolved).count();
        let rate = if total > 0 {
            format!("{:.1}%", (resolved as f64 / total as f64) * 100.0)
        } else {
            "0%".to_string()
        };

        json!({
            "totalArguments": total,
            "resolvedArguments": resolved,
            "resolutionRate": rate,
            "persuasionAttempts": self.persuasion_attempts,
            "currentStrength": self.strength.as_str()
        })
    }

    pub fn get_recent_arguments(&self, limit: usize) -> Vec<Value> {
        let start = if limit >= self.argument_history.len() {
            0
        } else {
            self.argument_history.len() - limit
        };

        self.argument_history[start..]
            .iter()
            .map(|log| {
                json!({
                    "reason": log.reason,
                    "response": log.response,
                    "type": log.arg_type,
                    "timestamp": log.timestamp,
                    "strength": log.strength,
                    "resolved": log.resolved
                })
            })
            .collect()
    }

    pub fn analyze_plan(&self, plan: &str) -> Vec<String> {
        let mut issues = Vec::new();
        let lower = plan.to_lowercase();

        if lower.contains("just") && lower.contains("do") {
            issues.push("Vague execution without planning".to_string());
        }

        if lower.contains("all at once") || lower.contains("everything at the same time") {
            issues.push("Multitasking detected - should be sequential".to_string());
        }

        if lower.contains("no testing") || lower.contains("skip test") || lower.contains("won't test") {
            issues.push("Skipping testing is risky".to_string());
        }

        let has_spend = contains_any(&lower, &["buy", "invest"]);
        let has_safety = contains_any(&lower, &["budget", "limit", "small"]);
        if has_spend && !has_safety {
            issues.push("Financial commitment without safety net".to_string());
        }

        issues
    }

    pub fn critique_plan(&self, plan: &str) -> Option<Value> {
        let issues = self.analyze_plan(plan);

        if issues.is_empty() {
            return None;
        }

        let address = "Abhi";
        let message = if issues.len() == 1 {
            format!(
                "{}, found 1 potential issue:\n- {}\n\nLet's fix this before proceeding.",
                address, issues[0]
            )
        } else {
            format!(
                "{}, found {} potential issues:\n- {}\n\nLet's fix these before proceeding.",
                address,
                issues.len(),
                issues.join("\n- ")
            )
        };

        Some(json!({
            "shouldBlock": issues.len() > 1,
            "issues": issues,
            "message": message
        }))
    }
}
