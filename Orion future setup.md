# ORION 3.0 — Future Setup & Development Plan

## Architecture Overview

ORION 3.0 is an Electron desktop app — a personal AI mission control center.

### Stack
- **Runtime**: Electron 35 + Node.js (Chromium engine)
- **Frontend**: Vanilla HTML/CSS/JS, 3-column Mission Control layout
- **Backend (main process)**: Node.js with IPC bridge
- **Database**: SQL.js (SQLite compiled to WebAssembly) — zero native dependencies
- **AI Models**: Ollama (local) + 4 cloud APIs (Groq, OpenRouter Claude, OpenRouter Grok, Gemini)

### Directory Map
```
C:\ORION\
├── main.js              # Electron main process — IPC handlers, module wiring
├── preload.js           # Context bridge — exposes safe API to renderer
├── CONFIG\
│   ├── settings.json    # API keys, model configs, routing rules
│   └── modes.json       # Personality mode definitions (6 modes)
├── CORE\
│   ├── ai-brain.js      # Multi-brain router & stream manager
│   ├── brain-router.js  # Task classification → brain selection logic
│   ├── ollama.js        # Ollama local model client (streaming + non-streaming)
│   ├── agent.js         # Base agent class
│   ├── coder-agent.js   # Code generation agent
│   ├── business-agent.js# Business/media ops agent
│   ├── scheduler-agent.js# Scheduling/reminder agent
│   ├── agent-registry.js# Agent routing & execution
│   ├── memory-engine.js # SQLite-based memory (sessions, conversations, goals, etc.)
│   ├── personality-engine.js# Mode management & system prompt builder
│   ├── context-manager.js# Session context & active task tracking
│   ├── user-profile.js  # User learning & preference storage
│   ├── suggestion-engine.js# Proactive suggestions & follow-ups
│   ├── goal-tracker.js  # Goal lifecycle management
│   └── argument-engine.js# Constructive pushback logic
├── UI\
│   ├── index2.html      # Main UI — 3-column Mission Control layout
│   ├── renderer.js      # UI logic — chat, sessions, goals, voice, folders
│   ├── orion-3.js       # View switching, dashboard, sidebar data loading
│   ├── styles\
│   │   ├── orion-theme.css # Base dark theme, flex layout, scrollbar
│   │   └── orion-3.css    # 3-column layout styles (pending review)
│   └── assets\
│       ├── icon.png     # Source PNG icon
│       └── icon.ico     # [BROKEN] Windows icon — 126 bytes, corrupted
├── MEMORY\              # SQLite database files (runtime-generated)
├── DATA\                # Debug logs
├── LAUNCH_ORION.bat     # Desktop launcher
└── create-shortcut.vbs  # Shortcut creator
```

---

## ✅ Current Status — IMPLEMENTED & WORKING

### Core AI (100% operational)
- Multi-model routing chain: Ollama (local-first, 4 models) → Groq → OpenRouter Claude → OpenRouter Grok → Gemini
- Task classification: code, reasoning, simple/quick, internet/research, general
- Streaming responses from all clouds (SSE/OpenAI-compatible)
- Auto-fallback on timeout or API error
- 30-60s timeouts per brain with AbortController

### Agent System (100% operational)
- 3 sub-agents: Coder, Business, Scheduler
- Keyword-based automatic routing via AgentRegistry
- Each agent can create files, use Ollama, and return structured results
- Base agent class with extensible interface

### Memory & Persistence (100% operational)
- SQL.js with full table schema: sessions, conversations, memories, daily_logs, user_profile, context, goals, follow_ups, projects, folders
- Session management (create, switch, rename, delete)
- Conversation history with chronological ordering
- Project-based organization
- Folder/workspace tracking with scrap/recovery

### Personality Engine (100% operational)
- 6 modes: ORION (default), FRIDAY, JARVIS, Analyst, Coder, Business
- Each mode: unique name, display, description, voice, address, argument strength, color, system prompt
- Auto-detect mode from user message keywords
- Dynamic system prompt builder with user profile injection

### Advanced Modules (100% operational)
- **ContextManager**: Per-session context, active task tracking with expiry
- **UserProfile**: Learns from interactions, preference persistence
- **GoalTracker**: Create, complete, fail, delete goals; today's agenda; overdue
- **SuggestionEngine**: Proactive suggestions, timed follow-ups
- **ArgumentEngine**: Constructive pushback with configurable strength (gentle/direct/stubborn)

### Frontend UI (100% operational)
- 3-column Mission Control layout: left nav, center (views + chat), right sidebar
- 9 views: Dashboard, Tasks, Clients, Leads, Systems, Calendar, Analytics, Logs, Settings
- Dashboard: Mission progress ring, stats cards, goals panel, suggestions, quick actions
- Chat: Streaming messages, voice input/output, auto-resize textarea, typing indicator
- Right sidebar: Session info, AI recommendations, system status, recent activity
- Custom titlebar with minimize/maximize/close
- Settings overlay with API status indicators

### API Keys Configured (4 providers)
- Groq: llama-3.3-70b-versatile
- Gemini: gemini-2.0-flash
- OpenRouter: claude-3.5-sonnet
- OpenRouter: grok-2

---

## 🔴 CRITICAL — BROKEN (Fix immediately)

### 1. Chat Scroll Flexbox Bug — FIXED NOW
**Root cause**: `#main-content` and `#chat-area` were missing `min-height: 0`. In a flex column layout, a flex child with `overflow-y: auto` needs `min-height: 0` to constrain itself. Without it, the child expands beyond the parent, scroll never activates, messages overflow invisibly.

**Fix applied**: Added `min-height: 0` to both selectors in `UI/styles/orion-theme.css`.

### 2. Desktop Icon — icon.ico is corrupted (126 bytes)
**Root cause**: Previously attempted PowerShell System.Drawing.Icon.Save() which doesn't produce valid multi-resolution ICO files. The file is only 126 bytes (should be 5KB-100KB+).
**Fix needed**: Use a proper ICO encoder. Options:
- Use Python PIL with `img.save('icon.ico', format='ICO', sizes=[(256,256)])`
- Use online converter (manual step, convert icon.png to icon.ico)
- Use a Node.js-based ICO conversion package
- **Workaround**: Electron can use PNG directly for the taskbar icon if `icon.png` is used

---

## 📋 SHORT-TERM ENHANCEMENTS (Next Session)

### 1. Fix Icons Properly
- Generate valid icon.ico from icon.png (must have embedded 256x256 PNG inside ICO header)
- Update create-shortcut.vbs to delete old .lnk before creating
- Update LAUNCH_ORION.bat to always recreate shortcut (already done)

### 2. Missing Feature: Clients View
- Preload file: add IPC handler for `orion:addClient` and `orion:getClients`
- Core: add `clients` table to memory-engine.js
- UI: connect "Add Client" button to real form in index2.html

### 3. Missing Feature: Leads View
- Add IPC handlers, database table, UI popup for lead tracking

### 4. Missing Feature: Calendar View
- Integrate GoalTracker deadlines with a calendar display
- Use a lightweight date picker or build a simple month view

### 5. Code Formatting / Markdown Rendering
- Current: messages use `textContent` (XSS-safe but no formatting)
- Enhancement: Use a sanitized Markdown renderer with syntax highlighting for code blocks

### 6. System Tray + Minimize to Tray
- Add tray icon that appears in system notification area
- Minimize to tray instead of closing
- Click tray to restore window

---

## 🚀 MEDIUM-TERM ROADMAP

### 1. Ollama Local-First Mode (Re-enable)
- `settings.json` currently has `localFirst: false` and `ollama.enabled: false`
- When user installs Ollama + pulls models, flip these to true
- Models needed: qwen2.5-coder:1.5b, qwen2.5-coder:7b, qwen3:8b, deepseek-r1:7b
- Create a "Download Models" button in Settings UI

### 2. Agentic Autonomy
- Implement task queue in AgentRegistry (currently placeholder only)
- Allow agents to auto-spawn sub-tasks
- Add "approval gate" for file creation / destructive actions

### 3. Voice-First Mode
- Trigger voice input from hotkey (Ctrl+Space)
- Continuous listening mode with wake word "ORION"
- Voice activity detection (VAD) integration

### 4. Code Workspace Integration
- Full file tree browser in sidebar
- Edit files directly from chat (agent creates → user confirms)
- Git integration: view diffs, commit, push automations
- Terminal panel output streaming

### 5. Multi-Modal Input
- Image upload → OCR / description
- PDF analysis
- Screenshot capture → UI analysis

### 6. Plugin System
- Plugin directory with loadable modules
- Each plugin: manifest.json + JS handler
- Community plugin store concept

### 7. End-to-End Encryption
- Encrypt API keys at rest
- Encrypt conversation database
- Secure credential storage via OS keychain API

---

## 🔧 DEVELOPMENT WORKFLOW

### How to Run After Fixes
```
npm install                           # Install dependencies (electron, sql.js, google-ai)
LAUNCH_ORION.bat                      # Creates shortcut, launches app
# OR
npx electron .                        # Direct launch
# OR
npx electron . --dev                  # Dev mode with DevTools open
```

### Debugging Tips
- Debug logs: `DATA/debug.log`
- Console: Press Ctrl+Shift+I in app (or use --dev flag)
- Database: `MEMORY/orion.db` — can be opened with DB Browser for SQLite
- For renderer issues: Set breakpoints in renderer.js via DevTools Sources tab

### Build for Distribution
```bash
npx electron-builder --win            # Windows installer
npx electron-builder --win --x64      # 64-bit only
```
Requires electron-builder in devDependencies.

### Code Conventions
- Main process: Node.js CJS require/module.exports
- Renderer: Vanilla JS, no framework
- IPC: `window.orion.*` preload bridge for all main→renderer communication
- XSS prevention: Always use `textContent`, never `innerHTML` with user data
- Database: sql.js raw SQL via prepared statements
- CSS: Custom properties for theming, BEM-like class naming

### Known Configurations
- Ollama models in settings.json map to qwen3:8b (manager), qwen2.5-coder:7b (coder), deepseek-r1:7b (reasoning), qwen2.5-coder:1.5b (fast)
- Fallback chain: groq → openrouter-claude → gemini
- 6 personality modes with distinct system prompts and tones

---

## 📝 NOTES

- The project currently loads `index2.html` (the new Mission Control UI). The older `index.html` is legacy.
- SQL.js database auto-creates on first launch at `MEMORY/orion.db`
- There are 4 API keys in settings.json — all are active and funded
- Ollama is disabled by default — set `ollama.enabled: true` and `routing.localFirst: true` to activate
- The app is 100% offline-capable if Ollama is running locally with pulled models
