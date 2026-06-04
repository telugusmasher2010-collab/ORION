#![allow(dead_code)]
// ORION 3.0 — Tauri Backend
// Full IPC commands backed by SQLite database

mod core;
mod db;

use std::sync::{Mutex, OnceLock};
use tauri::Manager;

// Global database instance
static DB: OnceLock<db::Database> = OnceLock::new();
// Personality engine (loaded from CONFIG/modes.json + prompt files)
static PERSONALITY: OnceLock<Mutex<core::personality_engine::PersonalityEngine>> = OnceLock::new();
// Agent registry (loaded with default agents)
static REGISTRY: OnceLock<Mutex<core::agent_registry::AgentRegistry>> = OnceLock::new();
// Argument engine (stores history)
static ARGUMENT_ENGINE: OnceLock<Mutex<core::argument_engine::ArgumentEngine>> = OnceLock::new();
// Context manager (in-memory context + active task)
static CONTEXT_MANAGER: OnceLock<Mutex<core::context_manager::ContextManager>> = OnceLock::new();
// Suggestion engine (tracks suggestion history)
static SUGGESTION_ENGINE: OnceLock<Mutex<core::suggestion_engine::SuggestionEngine>> = OnceLock::new();
// Ollama brain (health check + model list)
static OLLAMA_BRAIN: OnceLock<Mutex<core::ollama_brain::OllamaBrain>> = OnceLock::new();

fn get_db() -> &'static db::Database {
    DB.get().expect("Database not initialized")
}

fn get_personality() -> &'static Mutex<core::personality_engine::PersonalityEngine> {
    PERSONALITY.get().expect("Personality engine not initialized")
}

fn get_registry() -> &'static Mutex<core::agent_registry::AgentRegistry> {
    REGISTRY.get().expect("Agent registry not initialized")
}

fn get_argument_engine() -> &'static Mutex<core::argument_engine::ArgumentEngine> {
    ARGUMENT_ENGINE.get().expect("Argument engine not initialized")
}

fn get_context_manager() -> &'static Mutex<core::context_manager::ContextManager> {
    CONTEXT_MANAGER.get().expect("Context manager not initialized")
}

fn get_suggestion_engine() -> &'static Mutex<core::suggestion_engine::SuggestionEngine> {
    SUGGESTION_ENGINE.get().expect("Suggestion engine not initialized")
}

fn get_ollama_brain() -> &'static Mutex<core::ollama_brain::OllamaBrain> {
    OLLAMA_BRAIN.get().expect("Ollama brain not initialized")
}

// ========================================
// WINDOW CONTROL COMMANDS
// ========================================

#[tauri::command]
fn minimize_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.minimize();
    }
}

#[tauri::command]
fn maximize_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_maximized().unwrap_or(false) {
            let _ = window.unmaximize();
        } else {
            let _ = window.maximize();
        }
    }
}

#[tauri::command]
fn close_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.close();
    }
}

// ========================================
// CHAT — AI Brain powered (Groq / OpenRouter / Gemini streaming)
// ========================================

static RESOURCE_DIR: OnceLock<std::path::PathBuf> = OnceLock::new();

fn read_settings_file() -> (serde_json::Value, bool) {
    let mut paths = vec![
        std::path::PathBuf::from("../CONFIG/settings.json"),
        std::path::PathBuf::from("../../CONFIG/settings.json"),
        std::path::PathBuf::from("../../../CONFIG/settings.json"),
    ];
    if let Some(rd) = RESOURCE_DIR.get() {
        paths.push(rd.join("CONFIG/settings.json"));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            paths.push(parent.join("CONFIG/settings.json"));
        }
    }
    for path in &paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(val) = serde_json::from_str(&content) {
                return (val, true);
            }
        }
    }
    (serde_json::json!({}), false)
}

#[tauri::command]
fn chat(app: tauri::AppHandle, message: String, session_id: i64) -> Result<String, String> {
    println!("[ORION] Chat: {} (session: {})", message, session_id);

    // Load settings for API keys
    let (settings, _) = read_settings_file();

    // Load conversation history (exclude current message — save it after history fetch)
    let history_rows = get_db().get_history(session_id, 50).unwrap_or_default();
    let mut history: Vec<serde_json::Value> = Vec::new();
    for row in &history_rows {
        if row.role == "user" || row.role == "assistant" {
            history.push(serde_json::json!({ "role": row.role, "content": row.content }));
        }
    }

    // Detect mode from message content
    let mode = get_personality().lock().unwrap().detect_mode(&message).to_string();

    // Save user message to DB (after history fetch, so it's not duplicated in prompt)
    let _ = get_db().save_message(session_id, "user", &message, &mode, "user");

    // Build system prompt from personality engine
    let system_prompt = get_personality().lock().unwrap().build_system_prompt("");

    // Check if a sub-agent should handle this task first
    let response = match get_registry().lock().unwrap().execute_task(&message, &settings) {
        Ok(Some((_info, agent_resp))) => {
            // Agent handled it — save files if any were created
            let workspace = RESOURCE_DIR.get()
                .cloned()
                .unwrap_or_else(|| std::path::PathBuf::from(".."))
                .join("PROJECTS");
            for file_path in &agent_resp.files_created {
                // Find the file content in the response for saving
                let files = core::agent::extract_files_from_response(&agent_resp.response);
                for f in files {
                    if &f.path == file_path {
                        let _ = core::agent::save_file(
                            &workspace.to_string_lossy(),
                            &f.path,
                            &f.content,
                        );
                    }
                }
            }
            agent_resp.response
        }
        Ok(None) => {
            // No agent routed — use AI brain
            core::ai_brain::chat(&app, &message, session_id, settings, &history, &system_prompt)?
        }
        Err(e) => {
            // Agent error — fall back to AI brain
            println!("[ORION] Agent error, falling back to brain: {}", e);
            core::ai_brain::chat(&app, &message, session_id, settings, &history, &system_prompt)?
        }
    };

    // Save assistant response to DB
    let _ = get_db().save_message(session_id, "assistant", &response, &mode, "groq");

    println!("[ORION] Chat complete ({} chars)", response.len());
    Ok(response)
}

// ========================================
// SESSION COMMANDS
// ========================================

#[tauri::command]
fn get_sessions() -> Vec<db::Session> {
    get_db().get_sessions().unwrap_or_default()
}

#[tauri::command]
fn get_current_session_id() -> i64 {
    get_db().get_current_session_id().unwrap_or(1)
}

#[tauri::command]
fn create_session(name: Option<String>, project_id: Option<i64>) -> i64 {
    let title = name.unwrap_or_else(|| "New Chat".to_string());
    let pid = project_id.unwrap_or(1);
    get_db().create_session(&title, pid).unwrap_or(1)
}

#[tauri::command]
fn switch_session(session_id: i64) -> bool {
    get_db().switch_session(session_id).is_ok()
}

#[tauri::command]
fn rename_session(session_id: i64, new_title: String) -> bool {
    get_db().rename_session(session_id, &new_title).is_ok()
}

#[tauri::command]
fn delete_session(session_id: i64) -> bool {
    get_db().delete_session(session_id).is_ok()
}

#[tauri::command]
fn get_history(session_id: i64) -> Vec<db::ConversationRow> {
    get_db().get_history(session_id, 100).unwrap_or_default()
}

#[tauri::command]
fn clear_history() -> bool {
    get_db().clear_history().is_ok()
}

// ========================================
// SETTINGS & STATS
// ========================================

#[tauri::command]
fn get_settings() -> serde_json::Value {
    let (val, found) = read_settings_file();
    if found { val } else { serde_json::json!({ "error": "settings.json not found" }) }
}

#[tauri::command]
fn get_stats() -> serde_json::Value {
    match get_db().get_stats() {
        Ok(stats) => serde_json::to_value(stats).unwrap_or_default(),
        Err(_) => serde_json::json!({}),
    }
}

#[tauri::command]
fn get_mode() -> serde_json::Value {
    let p = get_personality().lock().unwrap();
    serde_json::json!({
        "mode": p.get_current_mode(),
        "display": p.get_mode_display(p.get_current_mode()),
        "color": p.get_mode_color(p.get_current_mode()),
        "description": p.get_mode_description(p.get_current_mode()),
        "allModes": p.get_all_modes(),
    })
}

#[tauri::command]
fn set_mode(mode_name: String) -> serde_json::Value {
    let mut p = get_personality().lock().unwrap();
    let success = p.set_mode(&mode_name);
    serde_json::json!({ "success": success, "mode": mode_name })
}

// ========================================
// GOALS
// ========================================

#[tauri::command]
fn get_goals() -> Vec<db::Goal> {
    get_db().get_goals().unwrap_or_default()
}

#[tauri::command]
fn get_goal_stats() -> serde_json::Value {
    match get_db().get_goal_stats() {
        Ok(stats) => serde_json::to_value(stats).unwrap_or_default(),
        Err(_) => serde_json::json!({}),
    }
}

#[tauri::command]
fn create_goal(title: String, description: Option<String>, priority: Option<String>, category: Option<String>) -> i64 {
    let desc = description.unwrap_or_default();
    let pri = priority.unwrap_or_else(|| "medium".to_string());
    let cat = category.unwrap_or_else(|| "general".to_string());
    get_db().create_goal(&title, &desc, &pri, &cat).unwrap_or(0)
}

#[tauri::command]
fn complete_goal(id: i64) -> bool {
    get_db().complete_goal(id).is_ok()
}

#[tauri::command]
fn delete_goal(id: i64) -> bool {
    get_db().delete_goal(id).is_ok()
}

// ========================================
// PROJECTS
// ========================================

#[tauri::command]
fn get_projects() -> Vec<db::Project> {
    get_db().get_projects().unwrap_or_default()
}

#[tauri::command]
fn create_project(name: String, description: Option<String>) -> i64 {
    let desc = description.unwrap_or_default();
    get_db().create_project(&name, &desc).unwrap_or(0)
}

#[tauri::command]
fn update_project(id: i64, name: String, description: Option<String>) -> bool {
    let desc = description.unwrap_or_default();
    get_db().update_project(id, &name, &desc).is_ok()
}

#[tauri::command]
fn delete_project(id: i64) -> bool {
    get_db().delete_project(id).is_ok()
}

// ========================================
// FOLDERS
// ========================================

#[tauri::command]
fn get_folders() -> Vec<db::Folder> {
    get_db().get_folders().unwrap_or_default()
}

#[tauri::command]
fn select_folder() -> Option<String> {
    // TODO: Implement native folder dialog via tauri-plugin-dialog
    // For now, returns None — frontend should handle via manual path input
    None
}

#[tauri::command]
fn add_folder(path: String, name: Option<String>) -> i64 {
    let folder_name = name.unwrap_or_else(|| {
        std::path::Path::new(&path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    });
    get_db().add_folder(&path, &folder_name).unwrap_or(0)
}

#[tauri::command]
fn set_active_folder(id: i64) -> bool {
    get_db().set_active_folder(id).is_ok()
}

#[tauri::command]
fn get_active_folder() -> Option<db::Folder> {
    get_db().get_active_folder().unwrap_or(None)
}

#[tauri::command]
fn send_folder_to_scrap(id: i64) -> bool {
    get_db().send_folder_to_scrap(id).is_ok()
}

#[tauri::command]
fn get_forgotten_folders() -> Vec<db::Folder> {
    get_db().get_forgotten_folders().unwrap_or_default()
}

// ========================================
// CLIENTS
// ========================================

#[tauri::command]
fn list_clients() -> Vec<db::Client> {
    get_db().list_clients().unwrap_or_default()
}

#[tauri::command]
fn get_client(id: i64) -> Option<db::Client> {
    get_db().get_client(id).unwrap_or(None)
}

#[tauri::command]
fn create_client(data: serde_json::Value) -> i64 {
    let name = data.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let email = data.get("email").and_then(|v| v.as_str()).unwrap_or("");
    let phone = data.get("phone").and_then(|v| v.as_str()).unwrap_or("");
    let notes = data.get("notes").and_then(|v| v.as_str()).unwrap_or("");
    get_db().create_client(name, email, phone, notes).unwrap_or(0)
}

#[tauri::command]
fn update_client(id: i64, data: serde_json::Value) -> bool {
    get_db().update_client(id, &data).is_ok()
}

#[tauri::command]
fn delete_client(id: i64) -> bool {
    get_db().delete_client(id).is_ok()
}

// ========================================
// LEADS
// ========================================

#[tauri::command]
fn list_leads() -> Vec<db::Lead> {
    get_db().list_leads().unwrap_or_default()
}

#[tauri::command]
fn get_lead(id: i64) -> Option<db::Lead> {
    get_db().get_lead(id).unwrap_or(None)
}

#[tauri::command]
fn create_lead(data: serde_json::Value) -> i64 {
    get_db().create_lead(&data).unwrap_or(0)
}

#[tauri::command]
fn update_lead(id: i64, data: serde_json::Value) -> bool {
    get_db().update_lead(id, &data).is_ok()
}

#[tauri::command]
fn delete_lead(id: i64) -> bool {
    get_db().delete_lead(id).is_ok()
}

// ========================================
// AI / AGENTS (placeholder — port later)
// ========================================

#[tauri::command]
fn get_agents() -> serde_json::Value {
    get_registry().lock().unwrap().status_summary()
}

#[tauri::command]
fn get_ollama() -> serde_json::Value {
    get_ollama_brain().lock().unwrap().get_info()
}

#[tauri::command]
fn get_suggestions() -> Vec<serde_json::Value> {
    let goals = get_db().get_goals().unwrap_or_default();
    let goals_json: Vec<serde_json::Value> = goals.iter().map(|g| serde_json::to_value(g).unwrap_or_default()).collect();

    let follow_ups = get_db().get_follow_ups().unwrap_or_default();
    let follow_ups_json: Vec<serde_json::Value> = follow_ups.iter().map(|f| serde_json::to_value(f).unwrap_or_default()).collect();

    let active_task = get_context_manager().lock().unwrap().get_active_task();

    get_suggestion_engine().lock().unwrap().get_suggestions(
        &goals_json,
        &follow_ups_json,
        active_task.as_ref().map(|t| t.task.as_str()),
        active_task.as_ref().map(|t| t.started_at.as_str()),
    )
}

#[tauri::command]
fn get_argument_stats() -> serde_json::Value {
    get_argument_engine().lock().unwrap().get_stats()
}

#[tauri::command]
fn get_memory_context() -> serde_json::Value {
    let profile = get_db().get_user_profile().unwrap_or_default();
    let facts = get_db().get_facts(None).unwrap_or_default();
    serde_json::json!({
        "profile": serde_json::to_value(profile).unwrap_or_default(),
        "facts": serde_json::to_value(facts).unwrap_or_default(),
    })
}

// ========================================
// NEW: USER PROFILE COMMANDS
// ========================================

#[tauri::command]
fn get_user_info() -> serde_json::Value {
    serde_json::json!({
        "name": get_db().get_profile("name").ok().flatten().map(|p| p.value).unwrap_or_else(|| "Abhi".into()),
        "timezone": get_db().get_profile("timezone").ok().flatten().map(|p| p.value).unwrap_or_else(|| "Asia/Kolkata".into()),
        "communication_style": get_db().get_profile("communication_style").ok().flatten().map(|p| p.value).unwrap_or_else(|| "direct".into()),
    })
}

#[tauri::command]
fn set_preference(key: String, value: String) -> bool {
    get_db().set_profile(&key, &value, "preference").is_ok()
}

#[tauri::command]
fn get_facts(category: Option<String>) -> Vec<db::MemoryEntry> {
    get_db().get_facts(category.as_deref()).unwrap_or_default()
}

#[tauri::command]
fn save_fact(key: String, value: String, category: Option<String>) -> bool {
    get_db().save_fact(&key, &value, &category.unwrap_or_else(|| "general".into())).is_ok()
}

#[tauri::command]
fn search_history(query: String, limit: Option<i64>) -> Vec<db::ConversationRow> {
    get_db().search_history(&query, limit.unwrap_or(20)).unwrap_or_default()
}

#[tauri::command]
fn get_follow_ups() -> Vec<db::FollowUp> {
    get_db().get_follow_ups().unwrap_or_default()
}

#[tauri::command]
fn add_follow_up(context: String, remind_minutes: i64) -> i64 {
    // remind_at is minutes from now, stored as ISO datetime via SQL
    get_db().add_follow_up(&context, &format_remind_at(remind_minutes)).unwrap_or(0)
}

#[tauri::command]
fn dismiss_follow_up(id: i64) -> bool {
    get_db().dismiss_follow_up(id).is_ok()
}

// ========================================
// NEW: CONTEXT COMMANDS
// ========================================

#[tauri::command]
fn set_active_task(task: String) -> bool {
    get_context_manager().lock().unwrap().set_active_task(&task);
    true
}

#[tauri::command]
fn get_active_task() -> Option<serde_json::Value> {
    get_context_manager().lock().unwrap().get_active_task().map(|t| serde_json::json!({
        "task": t.task,
        "startedAt": t.started_at,
        "status": t.status,
    }))
}

#[tauri::command]
fn clear_active_task() -> bool {
    get_context_manager().lock().unwrap().clear_active_task();
    true
}

#[tauri::command]
fn check_ollama_health() -> serde_json::Value {
    let available = get_ollama_brain().lock().unwrap().check_health().unwrap_or(false);
    serde_json::json!({ "available": available })
}

fn format_remind_at(minutes_from_now: i64) -> String {
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let total_secs = dur.as_secs() + (minutes_from_now.max(0) as u64 * 60);
    let days = total_secs / 86400;
    let time_secs = total_secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;
    // Approximate date (good enough for reminders)
    let year = 1970 + (days / 365) as u32;
    let day_of_year = (days % 365) as u32;
    let month_days = [31, if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut remaining = day_of_year;
    let mut month = 1u32;
    let mut day = 1u32;
    for (i, &md) in month_days.iter().enumerate() {
        if remaining < md { month = i as u32 + 1; day = remaining + 1; break; }
        remaining -= md;
    }
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", year, month, day, hours, minutes, seconds)
}

// ========================================
// VOICE (placeholder)
// ========================================

#[tauri::command]
fn toggle_voice_input() -> serde_json::Value {
    serde_json::json!({ "enabled": false, "message": "Voice not yet ported to Rust" })
}

#[tauri::command]
fn toggle_voice_output() -> serde_json::Value {
    serde_json::json!({ "enabled": false, "message": "Voice not yet ported to Rust" })
}

// ========================================
// APP ENTRY
// ========================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Initialize database in DATA/orion.db
            let data_dir = app.path().app_data_dir().unwrap_or_else(|_| {
                std::path::PathBuf::from("../DATA")
            });
            let db_path = data_dir.join("orion.db");
            println!("[ORION] Database path: {:?}", db_path);

            let database = db::Database::new(&db_path)
                .expect("Failed to initialize database");
            if DB.set(database).is_err() {
                eprintln!("[ORION] Database already initialized");
            }

            let resource_dir = app.path().resource_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));

            // Store resource dir for settings file resolution
            let _ = RESOURCE_DIR.set(resource_dir.clone());

            // Initialize bridge path so Node.js child processes can find http-bridge.js
            core::constants::set_bridge_path(
                resource_dir.join("CORE/http-bridge.js").to_string_lossy().to_string()
            );

            // Initialize personality engine from resource files
            let personality = core::personality_engine::PersonalityEngine::new(&resource_dir);
            if PERSONALITY.set(Mutex::new(personality)).is_err() {
                eprintln!("[ORION] Personality engine already initialized");
            }

            // Initialize agent registry with default agents
            let mut registry = core::agent_registry::AgentRegistry::new();
            for agent in core::agent_registry::create_default_agents() {
                registry.register(agent);
            }
            if REGISTRY.set(Mutex::new(registry)).is_err() {
                eprintln!("[ORION] Agent registry already initialized");
            }

            // Initialize argument engine
            if ARGUMENT_ENGINE.set(Mutex::new(core::argument_engine::ArgumentEngine::new("direct"))).is_err() {
                eprintln!("[ORION] Argument engine already initialized");
            }

            // Initialize context manager
            if CONTEXT_MANAGER.set(Mutex::new(core::context_manager::ContextManager::new())).is_err() {
                eprintln!("[ORION] Context manager already initialized");
            }

            // Initialize suggestion engine
            if SUGGESTION_ENGINE.set(Mutex::new(core::suggestion_engine::SuggestionEngine::new())).is_err() {
                eprintln!("[ORION] Suggestion engine already initialized");
            }

            // Initialize Ollama brain
            if OLLAMA_BRAIN.set(Mutex::new(core::ollama_brain::OllamaBrain::new())).is_err() {
                eprintln!("[ORION] Ollama brain already initialized");
            }

            println!("[ORION] Database initialized successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Window
            minimize_window,
            maximize_window,
            close_window,
            // Chat
            chat,
            // Sessions
            get_sessions,
            get_current_session_id,
            create_session,
            switch_session,
            rename_session,
            delete_session,
            get_history,
            clear_history,
            // Settings & Stats
            get_settings,
            get_stats,
            get_mode,
            set_mode,
            // Goals
            get_goals,
            get_goal_stats,
            create_goal,
            complete_goal,
            delete_goal,
            // Projects
            get_projects,
            create_project,
            update_project,
            delete_project,
            // Folders
            get_folders,
            select_folder,
            add_folder,
            set_active_folder,
            get_active_folder,
            send_folder_to_scrap,
            get_forgotten_folders,
            // Clients
            list_clients,
            get_client,
            create_client,
            update_client,
            delete_client,
            // Leads
            list_leads,
            get_lead,
            create_lead,
            update_lead,
            delete_lead,
            // AI / Agents
            get_agents,
            get_ollama,
            check_ollama_health,
            get_suggestions,
            get_argument_stats,
            get_memory_context,
            get_user_info,
            set_preference,
            get_facts,
            save_fact,
            search_history,
            get_follow_ups,
            add_follow_up,
            dismiss_follow_up,
            set_active_task,
            get_active_task,
            clear_active_task,
            // Voice
            toggle_voice_input,
            toggle_voice_output,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ORION");
}
