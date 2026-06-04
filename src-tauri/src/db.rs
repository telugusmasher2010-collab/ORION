// ORION — SQLite Database Module (Rust)
// Mirrors memory-engine.js functionality using rusqlite

use rusqlite::{Connection, params, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

// ========================================
// DATA STRUCTURES
// ========================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    pub id: i64,
    pub title: String,
    pub project_id: i64,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConversationRow {
    pub role: String,
    pub content: String,
    pub mode: String,
    pub brain: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub is_default: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Goal {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub deadline: Option<String>,
    pub status: String,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Folder {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub is_active: i64,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Client {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub notes: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Lead {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub company: String,
    pub stage: String,
    pub value: f64,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GoalStats {
    pub total: i64,
    pub active: i64,
    pub completed: i64,
    pub failed: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppStats {
    pub total_messages: i64,
    pub total_facts: i64,
    pub today_messages: i64,
    pub total_sessions: i64,
    pub total_goals_active: i64,
    pub total_goals_completed: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FollowUp {
    pub id: i64,
    pub context: String,
    pub remind_at: String,
    pub dismissed: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserProfileEntry {
    pub key: String,
    pub value: String,
    pub category: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryEntry {
    pub key: String,
    pub value: String,
    pub category: String,
    pub timestamp: String,
}

// ========================================
// DATABASE
// ========================================

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(db_path: &PathBuf) -> SqlResult<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(db_path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.create_tables()?;
        Ok(db)
    }

    fn create_tables(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT DEFAULT 'New Chat',
                project_id INTEGER DEFAULT 1,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS conversations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER DEFAULT 1,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                mode TEXT DEFAULT 'default',
                brain TEXT DEFAULT 'local',
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (session_id) REFERENCES sessions(id)
            );
            CREATE INDEX IF NOT EXISTS idx_conversations_session_id ON conversations(session_id);
            CREATE INDEX IF NOT EXISTS idx_conversations_timestamp ON conversations(timestamp);
            CREATE TABLE IF NOT EXISTS memories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                category TEXT DEFAULT 'general',
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS daily_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                date TEXT NOT NULL UNIQUE,
                energy INTEGER,
                focus INTEGER,
                tasks_completed TEXT,
                notes TEXT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS user_profile (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT NOT NULL UNIQUE,
                value TEXT NOT NULL,
                category TEXT DEFAULT 'general',
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS context (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER,
                key TEXT NOT NULL,
                value TEXT,
                expires_at DATETIME,
                FOREIGN KEY (session_id) REFERENCES sessions(id)
            );
            CREATE TABLE IF NOT EXISTS goals (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                description TEXT,
                deadline DATETIME,
                status TEXT DEFAULT 'active',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                completed_at DATETIME
            );
            CREATE TABLE IF NOT EXISTS follow_ups (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                context TEXT NOT NULL,
                remind_at DATETIME,
                dismissed INTEGER DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                description TEXT,
                is_default INTEGER DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS folders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL,
                name TEXT NOT NULL,
                is_active INTEGER DEFAULT 0,
                status TEXT DEFAULT 'active',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS clients (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                email TEXT,
                phone TEXT,
                notes TEXT DEFAULT '',
                status TEXT DEFAULT 'active',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS leads (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                email TEXT,
                phone TEXT,
                company TEXT,
                stage TEXT DEFAULT 'cold',
                value REAL DEFAULT 0,
                notes TEXT DEFAULT '',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );"
        )?;

        // Create default project if none exists
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM projects", [], |r| r.get(0))?;
        if count == 0 {
            conn.execute(
                "INSERT INTO projects (name, description, is_default) VALUES ('General', 'General conversations', 1)",
                [],
            )?;
        }

        // Create default session if none exists
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))?;
        if count == 0 {
            conn.execute(
                "INSERT INTO sessions (title, project_id) VALUES ('Default Session', 1)",
                [],
            )?;
        }

        Ok(())
    }

    // ========================================
    // SESSIONS
    // ========================================

    pub fn get_sessions(&self) -> SqlResult<Vec<Session>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, project_id, updated_at FROM sessions ORDER BY updated_at DESC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Session {
                id: row.get(0)?,
                title: row.get(1)?,
                project_id: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })?;
        rows.collect()
    }

    pub fn get_current_session_id(&self) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        // Try to get from profile first
        let result: SqlResult<String> = conn.query_row(
            "SELECT value FROM user_profile WHERE key = 'current_session_id'",
            [],
            |r| r.get(0),
        );
        if let Ok(val) = result {
            if let Ok(id) = val.parse::<i64>() {
                return Ok(id);
            }
        }
        // Fallback to most recent session
        let id: i64 = conn.query_row(
            "SELECT id FROM sessions ORDER BY updated_at DESC LIMIT 1",
            [],
            |r| r.get(0),
        )?;
        Ok(id)
    }

    pub fn create_session(&self, title: &str, project_id: i64) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO sessions (title, project_id) VALUES (?1, ?2)",
            params![title, project_id],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn switch_session(&self, session_id: i64) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        // Store current session in profile
        conn.execute(
            "INSERT OR REPLACE INTO user_profile (key, value, category, updated_at) VALUES ('current_session_id', ?1, 'system', CURRENT_TIMESTAMP)",
            params![session_id.to_string()],
        )?;
        // Update session timestamp
        conn.execute(
            "UPDATE sessions SET updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            params![session_id],
        )?;
        Ok(())
    }

    pub fn rename_session(&self, session_id: i64, new_title: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE sessions SET title = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            params![new_title, session_id],
        )?;
        Ok(())
    }

    pub fn delete_session(&self, session_id: i64) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM conversations WHERE session_id = ?1", params![session_id])?;
        conn.execute("DELETE FROM sessions WHERE id = ?1", params![session_id])?;
        Ok(())
    }

    // ========================================
    // CONVERSATIONS
    // ========================================

    pub fn get_history(&self, session_id: i64, limit: i64) -> SqlResult<Vec<ConversationRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT role, content, mode, brain, timestamp FROM (
                SELECT role, content, mode, brain, timestamp, id
                FROM conversations WHERE session_id = ?1
                ORDER BY id DESC LIMIT ?2
            ) ORDER BY id ASC"
        )?;
        let rows = stmt.query_map(params![session_id, limit], |row| {
            Ok(ConversationRow {
                role: row.get(0)?,
                content: row.get(1)?,
                mode: row.get(2)?,
                brain: row.get(3)?,
                timestamp: row.get(4)?,
            })
        })?;
        rows.collect()
    }

    pub fn save_message(&self, session_id: i64, role: &str, content: &str, mode: &str, brain: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO conversations (session_id, role, content, mode, brain) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![session_id, role, content, mode, brain],
        )?;
        conn.execute(
            "UPDATE sessions SET updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            params![session_id],
        )?;
        Ok(())
    }

    pub fn clear_history(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM conversations", [])?;
        Ok(())
    }

    // ========================================
    // STATS
    // ========================================

    pub fn get_stats(&self) -> SqlResult<AppStats> {
        let conn = self.conn.lock().unwrap();
        let total_messages: i64 = conn.query_row("SELECT COUNT(*) FROM conversations", [], |r| r.get(0))?;
        let total_facts: i64 = conn.query_row("SELECT COUNT(*) FROM memories", [], |r| r.get(0))?;
        let today_messages: i64 = conn.query_row(
            "SELECT COUNT(*) FROM conversations WHERE date(timestamp) = date('now')",
            [],
            |r| r.get(0),
        )?;
        let total_sessions: i64 = conn.query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))?;
        let total_goals_active: i64 = conn.query_row(
            "SELECT COUNT(*) FROM goals WHERE status = 'active'",
            [],
            |r| r.get(0),
        )?;
        let total_goals_completed: i64 = conn.query_row(
            "SELECT COUNT(*) FROM goals WHERE status = 'completed'",
            [],
            |r| r.get(0),
        )?;

        Ok(AppStats {
            total_messages,
            total_facts,
            today_messages,
            total_sessions,
            total_goals_active,
            total_goals_completed,
        })
    }

    // ========================================
    // GOALS
    // ========================================

    pub fn get_goals(&self) -> SqlResult<Vec<Goal>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, description, deadline, status, created_at, completed_at FROM goals WHERE status = 'active' ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Goal {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                deadline: row.get(3)?,
                status: row.get(4)?,
                created_at: row.get(5)?,
                completed_at: row.get(6)?,
            })
        })?;
        rows.collect()
    }

    pub fn get_goal_stats(&self) -> SqlResult<GoalStats> {
        let conn = self.conn.lock().unwrap();
        let total: i64 = conn.query_row("SELECT COUNT(*) FROM goals", [], |r| r.get(0))?;
        let active: i64 = conn.query_row("SELECT COUNT(*) FROM goals WHERE status = 'active'", [], |r| r.get(0))?;
        let completed: i64 = conn.query_row("SELECT COUNT(*) FROM goals WHERE status = 'completed'", [], |r| r.get(0))?;
        let failed: i64 = conn.query_row("SELECT COUNT(*) FROM goals WHERE status = 'failed'", [], |r| r.get(0))?;
        Ok(GoalStats { total, active, completed, failed })
    }

    pub fn create_goal(&self, title: &str, description: &str, priority: &str, category: &str) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        // Store priority and category in description for now
        let full_desc = format!("[{}] [{}] {}", priority, category, description);
        conn.execute(
            "INSERT INTO goals (title, description, status) VALUES (?1, ?2, 'active')",
            params![title, full_desc],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn complete_goal(&self, goal_id: i64) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE goals SET status = 'completed', completed_at = CURRENT_TIMESTAMP WHERE id = ?1",
            params![goal_id],
        )?;
        Ok(())
    }

    pub fn delete_goal(&self, goal_id: i64) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM goals WHERE id = ?1", params![goal_id])?;
        Ok(())
    }

    // ========================================
    // PROJECTS
    // ========================================

    pub fn get_projects(&self) -> SqlResult<Vec<Project>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, is_default, datetime(created_at) FROM projects ORDER BY is_default DESC, updated_at DESC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                is_default: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;
        rows.collect()
    }

    pub fn create_project(&self, name: &str, description: &str) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO projects (name, description) VALUES (?1, ?2)",
            params![name, description],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_project(&self, project_id: i64, name: &str, description: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE projects SET name = ?1, description = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?3",
            params![name, description, project_id],
        )?;
        Ok(())
    }

    pub fn delete_project(&self, project_id: i64) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE sessions SET project_id = 1 WHERE project_id = ?1", params![project_id])?;
        conn.execute("DELETE FROM projects WHERE id = ?1 AND is_default = 0", params![project_id])?;
        Ok(())
    }

    // ========================================
    // FOLDERS
    // ========================================

    pub fn get_folders(&self) -> SqlResult<Vec<Folder>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, path, name, is_active, status, datetime(created_at) FROM folders WHERE status = 'active' ORDER BY is_active DESC, created_at DESC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Folder {
                id: row.get(0)?,
                path: row.get(1)?,
                name: row.get(2)?,
                is_active: row.get(3)?,
                status: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    pub fn add_folder(&self, path: &str, name: &str) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO folders (path, name, is_active) VALUES (?1, ?2, 0)",
            params![path, name],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn set_active_folder(&self, folder_id: i64) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE folders SET is_active = 0", [])?;
        conn.execute("UPDATE folders SET is_active = 1 WHERE id = ?1", params![folder_id])?;
        Ok(())
    }

    pub fn get_active_folder(&self) -> SqlResult<Option<Folder>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, path, name, is_active, status, datetime(created_at) FROM folders WHERE is_active = 1"
        )?;
        let mut rows = stmt.query_map([], |row| {
            Ok(Folder {
                id: row.get(0)?,
                path: row.get(1)?,
                name: row.get(2)?,
                is_active: row.get(3)?,
                status: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn send_folder_to_scrap(&self, folder_id: i64) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE folders SET status = 'forgotten' WHERE id = ?1", params![folder_id])?;
        Ok(())
    }

    pub fn get_forgotten_folders(&self) -> SqlResult<Vec<Folder>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, path, name, is_active, status, datetime(created_at) FROM folders WHERE status = 'forgotten'"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Folder {
                id: row.get(0)?,
                path: row.get(1)?,
                name: row.get(2)?,
                is_active: row.get(3)?,
                status: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    // ========================================
    // CLIENTS
    // ========================================

    pub fn list_clients(&self) -> SqlResult<Vec<Client>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, email, phone, notes, status, created_at, updated_at FROM clients ORDER BY updated_at DESC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Client {
                id: row.get(0)?,
                name: row.get(1)?,
                email: row.get(2)?,
                phone: row.get(3)?,
                notes: row.get(4)?,
                status: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;
        rows.collect()
    }

    pub fn get_client(&self, client_id: i64) -> SqlResult<Option<Client>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, email, phone, notes, status, created_at, updated_at FROM clients WHERE id = ?1"
        )?;
        let mut rows = stmt.query_map(params![client_id], |row| {
            Ok(Client {
                id: row.get(0)?,
                name: row.get(1)?,
                email: row.get(2)?,
                phone: row.get(3)?,
                notes: row.get(4)?,
                status: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn create_client(&self, name: &str, email: &str, phone: &str, notes: &str) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO clients (name, email, phone, notes) VALUES (?1, ?2, ?3, ?4)",
            params![name, email, phone, notes],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_client(&self, client_id: i64, data: &serde_json::Value) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        let mut sets = Vec::new();
        let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(v) = data.get("name").and_then(|v| v.as_str()) {
            sets.push("name = ?");
            values.push(Box::new(v.to_string()));
        }
        if let Some(v) = data.get("email").and_then(|v| v.as_str()) {
            sets.push("email = ?");
            values.push(Box::new(v.to_string()));
        }
        if let Some(v) = data.get("phone").and_then(|v| v.as_str()) {
            sets.push("phone = ?");
            values.push(Box::new(v.to_string()));
        }
        if let Some(v) = data.get("notes").and_then(|v| v.as_str()) {
            sets.push("notes = ?");
            values.push(Box::new(v.to_string()));
        }
        if let Some(v) = data.get("status").and_then(|v| v.as_str()) {
            sets.push("status = ?");
            values.push(Box::new(v.to_string()));
        }

        if sets.is_empty() {
            return Ok(());
        }

        let sql = format!("UPDATE clients SET {}, updated_at = CURRENT_TIMESTAMP WHERE id = ?", sets.join(", "));
        values.push(Box::new(client_id));

        let params_ref: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|v| v.as_ref()).collect();
        conn.execute(&sql, params_ref.as_slice())?;
        Ok(())
    }

    pub fn delete_client(&self, client_id: i64) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM clients WHERE id = ?1", params![client_id])?;
        Ok(())
    }

    // ========================================
    // LEADS
    // ========================================

    pub fn list_leads(&self) -> SqlResult<Vec<Lead>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, email, phone, company, stage, value, notes, created_at, updated_at FROM leads ORDER BY updated_at DESC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Lead {
                id: row.get(0)?,
                name: row.get(1)?,
                email: row.get(2)?,
                phone: row.get(3)?,
                company: row.get(4)?,
                stage: row.get(5)?,
                value: row.get(6)?,
                notes: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?;
        rows.collect()
    }

    pub fn get_lead(&self, lead_id: i64) -> SqlResult<Option<Lead>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, email, phone, company, stage, value, notes, created_at, updated_at FROM leads WHERE id = ?1"
        )?;
        let mut rows = stmt.query_map(params![lead_id], |row| {
            Ok(Lead {
                id: row.get(0)?,
                name: row.get(1)?,
                email: row.get(2)?,
                phone: row.get(3)?,
                company: row.get(4)?,
                stage: row.get(5)?,
                value: row.get(6)?,
                notes: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn create_lead(&self, data: &serde_json::Value) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        let name = data.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let email = data.get("email").and_then(|v| v.as_str()).unwrap_or("");
        let phone = data.get("phone").and_then(|v| v.as_str()).unwrap_or("");
        let company = data.get("company").and_then(|v| v.as_str()).unwrap_or("");
        let stage = data.get("stage").and_then(|v| v.as_str()).unwrap_or("cold");
        let value = data.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let notes = data.get("notes").and_then(|v| v.as_str()).unwrap_or("");

        conn.execute(
            "INSERT INTO leads (name, email, phone, company, stage, value, notes) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![name, email, phone, company, stage, value, notes],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_lead(&self, lead_id: i64, data: &serde_json::Value) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        let mut sets = Vec::new();
        let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(v) = data.get("name").and_then(|v| v.as_str()) {
            sets.push("name = ?");
            values.push(Box::new(v.to_string()));
        }
        if let Some(v) = data.get("email").and_then(|v| v.as_str()) {
            sets.push("email = ?");
            values.push(Box::new(v.to_string()));
        }
        if let Some(v) = data.get("phone").and_then(|v| v.as_str()) {
            sets.push("phone = ?");
            values.push(Box::new(v.to_string()));
        }
        if let Some(v) = data.get("company").and_then(|v| v.as_str()) {
            sets.push("company = ?");
            values.push(Box::new(v.to_string()));
        }
        if let Some(v) = data.get("stage").and_then(|v| v.as_str()) {
            sets.push("stage = ?");
            values.push(Box::new(v.to_string()));
        }
        if let Some(v) = data.get("value").and_then(|v| v.as_f64()) {
            sets.push("value = ?");
            values.push(Box::new(v));
        }
        if let Some(v) = data.get("notes").and_then(|v| v.as_str()) {
            sets.push("notes = ?");
            values.push(Box::new(v.to_string()));
        }

        if sets.is_empty() {
            return Ok(());
        }

        let sql = format!("UPDATE leads SET {}, updated_at = CURRENT_TIMESTAMP WHERE id = ?", sets.join(", "));
        values.push(Box::new(lead_id));

        let params_ref: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|v| v.as_ref()).collect();
        conn.execute(&sql, params_ref.as_slice())?;
        Ok(())
    }

    pub fn delete_lead(&self, lead_id: i64) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM leads WHERE id = ?1", params![lead_id])?;
        Ok(())
    }

    // ========================================
    // USER PROFILE
    // ========================================

    pub fn get_user_profile(&self) -> SqlResult<Vec<UserProfileEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT key, value, category FROM user_profile")?;
        let rows = stmt.query_map([], |row| {
            Ok(UserProfileEntry {
                key: row.get(0)?,
                value: row.get(1)?,
                category: row.get(2)?,
            })
        })?;
        rows.collect()
    }

    pub fn get_profile(&self, key: &str) -> SqlResult<Option<UserProfileEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT key, value, category FROM user_profile WHERE key = ?1")?;
        let mut rows = stmt.query_map(params![key], |row| {
            Ok(UserProfileEntry {
                key: row.get(0)?,
                value: row.get(1)?,
                category: row.get(2)?,
            })
        })?;
        match rows.next() {
            Some(Ok(entry)) => Ok(Some(entry)),
            _ => Ok(None),
        }
    }

    pub fn set_profile(&self, key: &str, value: &str, category: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO user_profile (key, value, category, updated_at) VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)",
            params![key, value, category],
        )?;
        Ok(())
    }

    // ========================================
    // CONTEXT
    // ========================================

    pub fn set_context(&self, session_id: i64, key: &str, value: &str, expires_minutes: i64) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO context (session_id, key, value, expires_at) VALUES (?1, ?2, ?3, datetime('now', '+' || ?4 || ' minutes'))",
            params![session_id, key, value, expires_minutes],
        )?;
        Ok(())
    }

    pub fn get_context(&self, session_id: i64, key: &str) -> SqlResult<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT value FROM context WHERE session_id = ?1 AND key = ?2 AND (expires_at IS NULL OR expires_at > datetime('now'))"
        )?;
        let mut rows = stmt.query_map(params![session_id, key], |row| {
            row.get::<_, String>(0)
        })?;
        match rows.next() {
            Some(Ok(val)) => Ok(Some(val)),
            _ => Ok(None),
        }
    }

    pub fn clear_expired_context(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM context WHERE expires_at IS NOT NULL AND expires_at <= datetime('now')", [])?;
        Ok(())
    }

    // ========================================
    // MEMORIES / FACTS
    // ========================================

    pub fn save_fact(&self, key: &str, value: &str, category: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO memories (key, value, category, timestamp) VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP)",
            params![key, value, category],
        )?;
        Ok(())
    }

    pub fn get_facts(&self, category: Option<&str>) -> SqlResult<Vec<MemoryEntry>> {
        let conn = self.conn.lock().unwrap();
        let (sql, params_vec): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = match category {
            Some(cat) => (
                "SELECT key, value, category, timestamp FROM memories WHERE category = ?1 ORDER BY timestamp DESC".into(),
                vec![Box::new(cat.to_string())],
            ),
            None => (
                "SELECT key, value, category, timestamp FROM memories ORDER BY timestamp DESC".into(),
                vec![],
            ),
        };
        let mut stmt = conn.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            Ok(MemoryEntry {
                key: row.get(0)?,
                value: row.get(1)?,
                category: row.get(2)?,
                timestamp: row.get(3)?,
            })
        })?;
        rows.collect()
    }

    pub fn search_history(&self, query: &str, limit: i64) -> SqlResult<Vec<ConversationRow>> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT role, content, mode, brain, timestamp FROM conversations WHERE content LIKE ?1 ORDER BY id DESC LIMIT ?2"
        )?;
        let rows = stmt.query_map(params![pattern, limit], |row| {
            Ok(ConversationRow {
                role: row.get(0)?,
                content: row.get(1)?,
                mode: row.get(2)?,
                brain: row.get(3)?,
                timestamp: row.get(4)?,
            })
        })?;
        let mut results: Vec<ConversationRow> = rows.collect::<SqlResult<Vec<_>>>()?;
        results.reverse();
        Ok(results)
    }

    // ========================================
    // FOLLOW-UPS
    // ========================================

    pub fn add_follow_up(&self, context: &str, remind_at: &str) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO follow_ups (context, remind_at) VALUES (?1, ?2)",
            params![context, remind_at],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_pending_follow_ups(&self) -> SqlResult<Vec<FollowUp>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, context, remind_at, dismissed, created_at FROM follow_ups WHERE dismissed = 0 AND remind_at > datetime('now') ORDER BY remind_at ASC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(FollowUp {
                id: row.get(0)?,
                context: row.get(1)?,
                remind_at: row.get(2)?,
                dismissed: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;
        rows.collect()
    }

    // ========================================
    // FOLLOW-UPS / SUGGESTIONS
    // ========================================

    pub fn get_follow_ups(&self) -> SqlResult<Vec<FollowUp>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, context, remind_at, dismissed, created_at FROM follow_ups WHERE dismissed = 0 AND remind_at <= datetime('now') ORDER BY remind_at ASC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(FollowUp {
                id: row.get(0)?,
                context: row.get(1)?,
                remind_at: row.get(2)?,
                dismissed: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;
        rows.collect()
    }

    pub fn dismiss_follow_up(&self, follow_up_id: i64) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE follow_ups SET dismissed = 1 WHERE id = ?1", params![follow_up_id])?;
        Ok(())
    }
}
