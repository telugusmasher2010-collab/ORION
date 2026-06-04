# ORION 3.0 — Personal AI Assistant & Multi-Agent Operator

> **Author:** Abhi  
> **Version:** 3.0.0  
> **Electron + Node.js Desktop App**

---

## Architecture Overview

### Stack
- **Runtime:** Electron 35 + Node.js
- **Frontend:** Vanilla JS, CSS3, HTML5
- **Backend:** Node.js (main process), sql.js (SQLite)
- **AI Backends:** Groq (primary), OpenRouter (Claude/Grok), Gemini (fallback), Ollama (local, disabled)

### Directory Structure
```
C:\ORION
├── CONFIG/settings.json         # AI backends config, routing rules
├── CORE/                        # Backend engine
│   ├── main.js                  # Electron main process, IPC handlers
│   ├── ai-brain.js              # Multi-brain manager (routes to AI)
│   ├── brain-router.js          # Smart routing logic (local/cloud)
│   ├── memory-engine.js         # SQLite-based chat/session memory
│   ├── personality-engine.js    # Mode detection (coder/business/etc)
│   ├── agent-registry.js        # Sub-agent system
│   ├── agent.js                 # Base agent class
│   ├── coder-agent.js           # Code-focused sub-agent
│   ├── business-agent.js        # Business-focused sub-agent
│   ├── scheduler-agent.js       # Scheduling sub-agent
│   ├── ollama.js                # Ollama local LLM client
│   ├── user-profile.js          # User learning & preferences
│   ├── context-manager.js       # Session context tracking
│   ├── argument-engine.js       # Pushback/critique engine
│   ├── suggestion-engine.js     # Proactive suggestions
│   ├── goal-tracker.js          # Goal management (CRUD)
│   └── systems_rules.md.txt     # System prompt rules
├── UI/                          # Frontend
│   ├── index.html               # Legacy UI (v2.0)
│   ├── index2.html              # ORION 3.0 UI (3-column Mission Control)
│   ├── renderer.js              # UI logic (chat, voice, goals, sessions)
│   ├── orion-3.js               # ORION 3.0 view system, sidebar data
│   ├── styles/
│   │   ├── orion-theme.css      # Legacy styles (v2.0)
│   │   └── orion-3.css          # ORION 3.0 styles (dark theme, layouts)
│   └── assets/                  # Icons, images
├── DATA/                        # SQLite databases
├── preload.js                   # Electron preload (secure bridge)
├── main.js                      # App entry point
└── package.json
```

---

## Phase 1: Original Architecture (ORION 2.0)

### What Existed
- Electron app with single-column chat UI
- 3 sub-agents: Coder, Business, Scheduler
- AI routing: Ollama (local) → Groq → Gemini
- SQLite memory engine with sessions/projects/folders
- Personality engine with modes: ORION, FRIDAY, JARVIS, Analyst, Coder, Business
- Voice input/output (Web Speech API)
- Goal tracker, suggestion engine, argument engine
- User profile with learning
- Frameless custom title bar

### Settings Configuration
- **Primary AI:** Groq (`llama-3.3-70b-versatile`)
- **Cloud AI:** OpenRouter (Claude 3.5 Sonnet, Grok-2), Gemini 2.0 Flash
- **Local AI:** Ollama (disabled — `enabled: false`)
- **Routing:** cloud-first (`localFirst: false`), Groq primary with Claude/Gemini fallback

---

## Phase 2: ORION 3.0 UI — Advanced Mission Control

### Objective
Redesign from single-column chat to 3-column **Mission Control** layout with persistent chat, contextual sidebar, and navigation views.

### Files Created/Modified

| File | Status | Purpose |
|------|--------|---------|
| `UI/index2.html` | **Created** | 3-column HTML layout |
| `UI/styles/orion-3.css` | **Created** | Full dark theme, animations, all panel styles |
| `UI/orion-3.js` | **Created** | View switching, right sidebar data binding, progress ring |
| `main.js` | **Modified** | Loads index2.html, added error logging |

### 3-Column Layout
```
┌─────────────────────────────────────────────────────┐
│  Title Bar: ORION 3.0 | ● ONLINE | Mode Badge    │
├────────┬────────────────────────────┬────────────────┤
│ LEFT   │      CENTER PANEL          │  RIGHT         │
│ SIDEBAR│  ┌─────────────────────┐   │  SIDEBAR       │
│        │  │ View Container      │   │                │
│ Home   │  │ (Dashboard / Tasks  │   │  Session Info  │
│ Tasks  │  │  / Systems / etc.)  │   │  AI Recs       │
│ Clients│  └─────────────────────┘   │  System Status │
│ Leads  │  ┌─────────────────────┐   │  Activity      │
│ Systems│  │ Command Center      │   │                │
│ Cal.   │  │ [Chat Messages]     │   │                │
│ Analyt.│  │ [Input Area]        │   │                │
│ Logs   │  └─────────────────────┘   │                │
│ Setti. │                           │                │
│        │  Quick Actions Bar        │                │
│ [User] │  (Generate/Write/etc)     │                │
│[+Task] │                           │                │
└────────┴────────────────────────────┴────────────────┘
```

---

## Phase 3: Bug Fixes — Chat Reply Not Working

### Root Cause
`preload.js` used variable `initialized` without declaring it (`let initialized = false;` was missing). This caused a `ReferenceError` every time `window.orion.getCurrentSessionId()` was called.

### Impact
- `sendMessage()` would crash on line 224
- User saw "No chat selected" error instead of AI reply
- Chat appeared to send but no response showed

### Fix Applied
- **`preload.js:9`** — Added `let initialized = false;`
- **`ai-brain.js:165-217`** — Added non-streaming fallback (`_groqNonStreaming`) for Electron environments where streaming `response.body.getReader()` isn't available

### Result
Groq API calls succeed. Chat replies work correctly.

---

## Phase 4: Sidebar Wiring

### Left Sidebar — 9 Navigation Views
| Nav Item | View ID | Content | Backend Data |
|----------|---------|---------|--------------|
| Dashboard | `view-dashboard` | Mission progress ring, stats cards, welcome card | `getStats()`, `getGoals()`, `getSessions()` |
| Tasks | `view-tasks` | Goal cards with complete/delete | `getGoals()`, `createGoal()` |
| Clients | `view-clients` | Placeholder (coming soon) | — |
| Leads | `view-leads` | Placeholder (coming soon) | — |
| Systems | `view-systems` | AI Engine, Ollama, Memory, Agents status | `getAgents()`, `getOllama()`, `getStats()` |
| Calendar | `view-calendar` | Placeholder (coming soon) | — |
| Analytics | `view-analytics` | Sessions, messages, goals counts | `getStats()`, `getGoalStats()` |
| Logs | `view-logs` | System log output | `logSystem()` |
| Settings | `view-settings` | API statuses, mode, restart button | `getMode()`, `getOllama()` |

### Right Sidebar — 4 Context Panels
| Panel | Content | Backend Data |
|-------|---------|--------------|
| Current Session | Session ID, mode, msg count, brain | `getCurrentSessionId()`, `getMode()`, `getHistory()` |
| AI Recommendations | Smart suggestions | `getSuggestions()` |
| System Status | AI engine, Groq, Memory, Ollama | `getOllama()`, `getMode()` |
| Recent Activity | Last 5 chat messages | `getHistory()` |

### Fixes for View Switching
- **CSS specificity conflict:** `orion-theme.css` had `.view.hidden { display: none; }` conflicting with `.view.active { display: flex; }`
- **Solution:** Use inline styles (`element.style.display`) instead of CSS classes for view visibility
- **Dual click handling:** Both `onclick` attributes and `addEventListener` for reliable nav response
- **renderer.js `showView`:** Modified to delegate to `switchView` when ORION 3.0 UI is active

---

## Current State

### Working
- ✅ Electron app boots, loads 3-column Mission Control UI
- ✅ Groq API integration — chat sends and receives AI replies
- ✅ Streaming responses with chunk updates
- ✅ Session management (create, switch, rename, delete)
- ✅ Multiple AI backends configured (Groq primary, OpenRouter/Gemini fallbacks)
- ✅ All 9 sidebar navigation views switch content
- ✅ Dashboard stats load from backend
- ✅ Tasks/Goals view with create/complete/delete
- ✅ Systems view with agent/engine status
- ✅ Analytics view with counts
- ✅ Logs view with system log output
- ✅ Settings view with API/mode status
- ✅ Right sidebar live data (session info, recommendations, status, activity)
- ✅ Custom frameless title bar with window controls
- ✅ Voice input (Web Speech API)
- ✅ Mode switching (ORION, FRIDAY, JARVIS, Analyst, Coder, Business)
- ✅ Sub-agents (Coder, Business, Scheduler)
- ✅ SQLite memory with sessions, projects, folders
- ✅ Smart AI routing (code → coder agent, research → cloud, etc.)

### Not Working / Placeholder
- ❌ **Ollama local models not connected** (intentionally disabled — `settings.json: ollama.enabled: false`)
- ❌ **Clients view** — placeholder only
- ❌ **Leads view** — placeholder only
- ❌ **Calendar view** — placeholder only
- ❌ **New Task button** on sidebar footer (`createNewTask`) — creates goals, but button not wired in HTML

### Known Issues
- Gemini API quota exceeded (429 errors — free tier limit)
- View switching uses inline `style.display` (workaround for CSS specificity conflict with orion-theme.css)
- `activeView` variable in renderer.js not updated when switchView is used (cosmetic, no functional impact)

---

## Backend Architecture

### AI Routing Chain (`brain-router.js`)
```
User Message
  → Is internet task? → OpenRouter Grok (real-time)
  → Is code task? & Ollama available & localFirst? → Ollama Coder
  → Is reasoning task? & Ollama available & localFirst? → Ollama Reasoner
  → Simple task? & Ollama available & localFirst? → Ollama Fast
  → Default: Fallback chain
      → Groq (primary)
      → OpenRouter Claude
      → OpenRouter Grok
      → Gemini
      → Error: "No AI brain available"
```

### Message Flow
```
Renderer (sendMessage)
  → preload.js (chat bridge)
    → main.js (IPC handler: orion:chat)
      → AIBrain.asyncChat()
        → AgentRegistry.executeTask() (check sub-agents)
        → BrainRouter.route() (pick AI backend)
        → _groqStream() / _openRouterClaudeStream() / etc.
          → fetch() to cloud API with stream:true
          → _readOpenAIStream() (parse SSE chunks)
            → onChunk callback → IPC 'orion:chunk' → renderer
        → Return full response
      → MemoryEngine.saveMessage()
      → ContextManager.update()
      → UserProfile.learn()
    ← Return { response, mode, brain }
  ← renderer updates chat UI
```

### Memory Engine
- SQLite via sql.js (WebAssembly)
- Tables: `sessions`, `messages`, `projects`, `folders`, `goals`, `profile`, `context`
- Session-based chat history with auto-naming

---

## Build & Run

```bash
# Install dependencies
npm install

# Run in development mode (DevTools open)
npm start -- --dev

# Run normally
npm start
```

---

## Future Roadmap

### Short Term
- [ ] Wire Clients, Leads, Calendar views with real backend data
- [ ] Add Gemini API key rotation or upgrade (current: quota exceeded)
- [ ] Re-enable Ollama when local models are available
- [ ] Add typing indicator fixes for non-streaming mode
- [ ] Dashboard stats — live refresh on new messages

### Medium Term
- [ ] Drag-resizable column dividers
- [ ] Theme customization (accent color picker)
- [ ] Export/import sessions
- [ ] AI-powered task generation from chat
- [ ] Persistent dashboard stats across sessions

### Long Term
- [ ] Multi-user profiles
- [ ] Plugin system for third-party agents
- [ ] WebSocket real-time sync
- [ ] Mobile companion app
