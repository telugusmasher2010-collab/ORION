use crate::core::agent::{Agent, AgentInfo, AgentResponse};
use serde_json::Value;
use std::collections::HashMap;

pub struct AgentRegistry {
    agents: HashMap<String, Agent>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        AgentRegistry {
            agents: HashMap::new(),
        }
    }

    pub fn register(&mut self, agent: Agent) {
        println!("[AgentRegistry] Registered: {} ({})", agent.name, agent.id);
        self.agents.insert(agent.id.clone(), agent);
    }

    pub fn get(&self, agent_id: &str) -> Option<&Agent> {
        self.agents.get(agent_id)
    }

    pub fn get_mut(&mut self, agent_id: &str) -> Option<&mut Agent> {
        self.agents.get_mut(agent_id)
    }

    pub fn get_all_info(&self) -> Vec<AgentInfo> {
        let mut result = Vec::new();
        for agent in self.agents.values() {
            result.push(agent.get_info());
        }
        result
    }

    pub fn route_task(&self, task: &str) -> Option<&str> {
        let task_lower = task.to_lowercase();
        let mut best_id: Option<&str> = None;
        let mut best_score: usize = 0;

        let keyword_sets: HashMap<&str, Vec<&str>> = [
            ("coder", vec![
                "code", "function", "script", "bug", "debug", "build website", "html", "css",
                "javascript", "python", "api endpoint", "database query", "edit file", "create file",
                "project setup", "program", "deploy", "git commit", "github", "npm install",
                "framework", "library", "component", "react", "vue", "node.js", "express",
                "endpoint", "write code", "fix bug", "refactor",
            ]),
            ("business", vec![
                "youtube", "shopify", "store", "ecommerce", "client", "freelance", "revenue",
                "marketing", "seo", "audience", "subscriber", "monetize", "brand", "launch",
                "sales", "product listing", "pricing strategy", "content strategy", "video idea",
                "thumbnail idea",
            ]),
            ("scheduler", vec![
                "schedule", "remind", "deadline", "meeting", "calendar", "task list", "daily plan",
                "weekly plan", "alert", "organize", "when is", "set reminder", "create task",
            ]),
        ].into_iter().collect();

        for (&agent_id, keywords) in &keyword_sets {
            let mut score: usize = 0;
            for keyword in keywords {
                if task_lower.contains(keyword) {
                    score += keyword.split_whitespace().count() * 2;
                }
            }
            if score > best_score {
                best_score = score;
                best_id = Some(agent_id);
            }
        }

        if best_score > 0 { best_id } else { None }
    }

    pub fn execute_task(
        &mut self,
        task: &str,
        settings: &Value,
    ) -> Result<Option<(AgentInfo, AgentResponse)>, String> {
        let agent_id = match self.route_task(task) {
            Some(id) => id.to_string(),
            None => return Ok(None),
        };

        let agent = self.agents.get_mut(&agent_id)
            .ok_or_else(|| format!("Agent '{}' not found", agent_id))?;

        println!("[AgentRegistry] Routing to {}: {}...", agent.name, &task[..task.len().min(80)]);

        let response = agent.execute(task, settings)?;
        let info = agent.get_info();

        Ok(Some((info, response)))
    }

    pub fn status_summary(&self) -> Value {
        let total = self.agents.len();
        let active = self.agents.values().filter(|a| a.status == "working").count();
        serde_json::json!({
            "totalAgents": total,
            "activeAgents": active,
            "agents": self.get_all_info(),
        })
    }
}

pub fn create_default_agents() -> Vec<super::agent::Agent> {
    vec![
        Agent::new(
            "coder",
            "Coder",
            "Writes, edits, and manages code files. Full-stack development.",
            "You are ORION's Coder Agent. You write clean, production-ready code.\n\n\
             RULES:\n\
             1. When asked to create a file, output the FULL file content in a code block with the path as a comment at the top.\n\
             2. When asked to edit a file, output the FULL updated file content.\n\
             3. Always explain what you changed and why.\n\
             4. Use modern best practices. No deprecated patterns.\n\
             5. If the task involves multiple files, output each one separately with clear path markers.\n\
             6. If you need to run a command, output it as: [RUN]: command here\n\
             7. Default to the user's tech stack unless specified otherwise.\n\
             8. Keep code minimal but complete — no placeholder comments.\n\n\
             If asked to save a file, wrap the content like:\n\
             [FILE: relative/path/to/file.ext]\n\
             ```language\n\
             ...code...\n\
             ```",
        ),
        Agent::new(
            "business",
            "Business",
            "Business operations — revenue, clients, marketing, YouTube, Shopify.",
            "You are ORION's Business Agent. Revenue over perfection.\n\n\
             THINK:\n\
             - What makes money TODAY, not 'someday'.\n\
             - 'Good enough to ship' beats 'perfect but late'.\n\
             - Block overthinking. If the user is planning for hours, interrupt them.\n\
             - Flag unpaid work, vague promises, unclear deliverables.\n\
             - Think about the 80/20 — what 20% of effort gives 80% of results?\n\n\
             'Don't overthink this. Ship it by Friday.'",
        ),
        Agent::new(
            "scheduler",
            "Scheduler",
            "Calendar & scheduling — reminders, deadlines, task lists, daily plans.",
            "You are ORION's Scheduler Agent. You manage time and tasks.\n\n\
             RULES:\n\
             - When asked to set a reminder, confirm the time and task clearly.\n\
             - For deadlines, break into milestones with dates.\n\
             - Keep task lists actionable — one task = one concrete action.\n\
             - Prioritize by urgency AND impact.\n\
             - If the user has too many tasks, suggest deferring or dropping the lowest-impact ones.",
        ),
    ]
}
