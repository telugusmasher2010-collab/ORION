# ORION — PLAN AND STRUCTURE
## Master Document for All ORION Plans, Architecture & Progress

---

## Project Overview

**ORION** — Personal AI Assistant & Multi-Agent Operator
- Real-life JARVIS equivalent
- Electron desktop app → **Migrating to Tauri + Rust**
- 6-month development timeline
- Built by Abhi (solo developer)

---

## Tech Stack (Current — Transitioning)

| Layer | Current (Electron) | Target (Tauri) |
|-------|-------------------|----------------|
| Frontend | HTML/CSS/JS | HTML/CSS/JS (unchanged) |
| Backend | Node.js | Rust |
| Database | sql.js (SQLite in JS) | rusqlite (native SQLite) |
| HTTP Client | Node.js fetch | reqwest (Rust) |
| IPC | Electron ipcMain/ipcRenderer | Tauri invoke commands |
| Window | Chromium (~200MB) | WebView2 (~30MB) |

---

## Architecture

### Current Structure
```
ORION/
├── main.js                 → Electron main process (REPLACE)
├── preload.js              → IPC bridge (REPLACE)
├── package.json            → Node.js config
├── UI/
│   ├── index2.html         → Main UI (KEEP)
│   ├── orion-3.css         → Styles (KEEP)
│   ├── orion-3.js          → UI logic (KEEP, minor edits)
│   ├── renderer.js         → Chat/UI renderer (KEEP, minor edits)
│   ├── tauri-bridge.js     → NEW: Tauri/Electron abstraction
│   └── assets/             → Icons, images
├── CORE/
│   ├── ai-brain.js         → Multi-brain AI manager
│   ├── brain-router.js     → Routes to local/cloud AI
│   ├── memory-engine.js    → SQLite memory system
│   ├── personality-engine.js → Personality & modes
│   ├── context-manager.js  → Conversation context
│   ├── argument-engine.js  → Challenge mode logic
│   ├── suggestion-engine.js → Proactive suggestions
│   ├── goal-tracker.js     → Goal/task tracking
│   ├── user-profile.js     → User preferences
│   ├── agent-registry.js   → Sub-agent manager
│   ├── coder-agent.js      → Code generation agent
│   ├── business-agent.js   → Business operations agent
│   ├── scheduler-agent.js  → Calendar/scheduling agent
│   └── ollama.js           → Ollama local LLM client
├── CONFIG/
│   ├── settings.json
│   ├── modes.json
│   └── voice-profiles.json
├── DATA/                   → SQLite database, logs
└── src-tauri/              → NEW: Tauri project
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── capabilities/
    └── src/
        ├── main.rs         → Entry point
        └── lib.rs          → IPC commands + CORE logic
```

### Target Structure (After Migration)
```
ORION/
├── UI/                     → Frontend (unchanged)
├── src-tauri/              → Rust backend
│   ├── src/
│   │   ├── main.rs         → Entry point
│   │   ├── lib.rs          → Tauri commands
│   │   ├── core/
│   │   │   ├── ai_brain.rs
│   │   │   ├── brain_router.rs
│   │   │   ├── memory_engine.rs
│   │   │   ├── personality_engine.rs
│   │   │   ├── context_manager.rs
│   │   │   ├── argument_engine.rs
│   │   │   ├── suggestion_engine.rs
│   │   │   ├── goal_tracker.rs
│   │   │   ├── user_profile.rs
│   │   │   ├── agent_registry.rs
│   │   │   ├── coder_agent.rs
│   │   │   ├── business_agent.rs
│   │   │   ├── scheduler_agent.rs
│   │   │   └── ollama_client.rs
│   │   └── db/
│   │       └── schema.rs   → SQLite schema + migrations
│   ├── Cargo.toml
│   └── tauri.conf.json
├── CONFIG/
└── DATA/
```

---

## Migration Complete — Current State (2026-06-12)

**All CORE modules ported to Rust ✅** — 13 modules in `src-tauri/src/core/`:
- ai_brain, brain_router, personality_engine, agent_registry, agent
- context_manager, argument_engine, suggestion_engine, goal_tracker
- user_profile, ollama_brain, constants

**61/61 IPC commands verified ✅** — Every Tauri command cross-checked between Rust handler, `generate_handler![]`, and JS `tauriInvoke` calls.

**Clean builds ✅** — `cargo check`, `cargo clippy`, `cargo build --release` all pass with zero warnings.

**Vercel-style UI redesign applied ✅** — Dashboard with greeting bar, quick input, action cards, stats, recent chats. Sidebar with brand icon, nav sections. Right panel with profile, shortcuts, status.

**GitHub Actions CI/CD ✅** — Push to main auto-builds the .exe and creates a GitHub Release with MSI + NSIS installers.

### Remaining Work

| Priority | Item | Notes |
|----------|------|-------|
| **HIGH** | Replace http-bridge.js with native reqwest | Only remaining Node.js dependency. Makes ORION fully self-contained. |
| **MEDIUM** | Clean up node_modules (319MB) | Blocked on reqwest replacement above |
| **MEDIUM** | End-to-end integration test | Launch app, verify all features work end-to-end |
| **LOW** | Auto-updater (Tauri built-in) | For push notifications when new releases are published |
| **LOW** | System tray integration | Tauri system tray for background operation |
| **LOW** | Plan doc sync | This doc should reflect latest state |

---

## ORION 3.0 UI Roadmap (Pre-Migration Plan)

These were the original UI tasks before the Tauri migration decision. They are now folded into Phase 3 of the migration plan (Days 11-15).

| Day | Status | Focus |
|-----|--------|-------|
| Day 1 | DONE | 3-column layout, sidebar, dashboard, right panel |
| Day 2 | TODO | Wire data: goals, projects, suggestions into panels |
| Day 3 | TODO | Voice waveform UI, mic animations |
| Day 4 | TODO | Transitions between views, loading states |
| Day 5 | TODO | Responsive design, mobile breakpoints |
| Day 6 | TODO | Full integration testing |
| Day 7 | TODO | Bug fixes, polish |

---

## 6-Phase Roadmap (Original)

```
ORION 2.0 ──▶ ORION 3.0 ──▶ ORION 4.0 ──▶ ORION 5.0 ──▶ ORION 6.0 ──▶ ORION 7.0
  Fixed     │   New UI      │   Voice     │  Automation │   Learning   │   Final
  Bugs      │   Dashboard   │   Features  │   Actions   │   System     │   Release
════════════╪═══════════════╪═════════════╪═════════════╪══════════════╪═══════════════
   DONE      │   WIP         │   FUTURE     │   FUTURE     │   FUTURE      │   FUTURE
```

---

## GPU Tower Independence Plan

**Goal:** Make ORION completely independent from third-party software, API keys, and models
**Timeline:** 6 months (tower arrives ~late 2026)

### Hardware (Confirmed)
- CPU: AMD Threadripper 7970X (32 cores)
- GPU: 2x NVIDIA RTX 4090 (48GB VRAM total)
- RAM: 128GB DDR5 ECC
- Storage: High-speed NVMe SSD

### Target Architecture
```
ORION Desktop App (Laptop)
        |
        |  HTTP to tower IP
        v
GPU Tower (Python + vLLM + CUDA)
        |
        ├── GPU 0: DeepSeek R1 or 70B model (chat/reasoning)
        └── GPU 1: Code model 14B (code generation)

Zero third-party dependencies. Zero API costs. Fully air-gapped capable.
```

### Current Dependencies (to be removed)
- Groq API (llama-3.3-70b) → REPLACE with local 70B model
- Gemini API → REPLACE with local model
- OpenRouter API (Claude, Grok) → REPLACE with local model
- Ollama → REPLACE with custom PyTorch inference server

---

## Performance Targets

| Metric | Electron (Current) | Tauri (Target) |
|--------|-------------------|----------------|
| RAM (idle) | 200-300MB | 30-50MB |
| RAM (active) | 300-500MB | 60-120MB |
| Startup time | 2-5 seconds | 0.5-1.5 seconds |
| Bundle size | 150-250MB | 2-5MB |
| CPU idle | 1-3% | ~0% |

---

## Known Bugs

| Bug | Status | Notes |
|-----|--------|-------|
| Chat sends but no reply | FIXED | preload.js ignored sessionId param. Fixed in renderer.js migration to bridge. |
| Chat sessionId mismatch (Tauri) | FIXED | JS sent camelCase `sessionId`, Rust expected snake_case `session_id`. Fixed in bridge. |
| Tauri loads wrong HTML file | FIXED | Was loading index.html (old Electron page). Renamed index2.html → index.html. |
| Missing "manager" model key | FIXED | Added `"manager": "qwen3.5:latest"` to settings.json models. |
| Bridge missing 36 methods | FIXED | Added all missing methods to tauri-bridge.js. Now 51 methods total. |
| clear_history sends unused session_id | FIXED | Bridge sent session_id, Rust took no params. Removed from bridge. |
| set_mode key mismatch | FIXED | Bridge sent `mode`, Rust expected `mode_name`. Fixed key. |
| create_goal wrong shape | FIXED | Bridge sent `{ goal }` object, Rust expected flat params. Fixed. |
| select_folder sends unused id | FIXED | Bridge sent `{ id }`, Rust took no params. Removed. |
| add_folder missing path | FIXED | Bridge sent only `{ name }`, Rust needed `{ path, name }`. Fixed. |
| fallbackChain config dead code | KNOWN | brain-router.js hardcodes its own fallback order, ignores settings.json fallbackChain. Fix Day 5. |

---

## Key Decisions Log

| Date | Decision | Reason |
|------|----------|--------|
| 2026-05-31 | Migrate from Electron to Tauri + Rust | 10x RAM savings, better streaming, GPU tower prep |
| 2026-05-31 | Use incremental migration (bridge layer) | Allows Electron/Tauri coexistence during transition |
| 2026-05-31 | Keep HTML/CSS/JS frontend unchanged | Minimize UI rework, only backend changes |
| 2026-05-31 | Use rusqlite over sql.js | Native SQLite, faster, less memory |
| 2026-05-31 | UI/logo redesign deferred until after Tauri migration | Tauri backend is the foundation — no mistakes allowed. Design once on final platform, not twice. |
| 2026-05-31 | Renamed index2.html → index.html | Tauri loads index.html by default; old Electron page renamed to index-electron-old.html |
| 2026-05-31 | Bridge parameter mapping (camelCase → snake_case) | Rust uses snake_case, JS uses camelCase; bridge maps sessionId → session_id |
| 2026-05-31 | Added "manager" model to settings.json | brain-router.js calls getModelForTask('manager') — was missing from config |
| 2026-06-01 | Days 2+3 combined: full IPC + SQLite in one session | All 39 Rust commands + rusqlite database with 12 tables |
| 2026-06-01 | 5 bridge parameter mismatches fixed | clear_history, set_mode, create_goal, select_folder, add_folder — all deserialization bugs |

---

## Bugs Found & Fixed (Day 1 Verification)

| Severity | Issue | Fix |
|----------|-------|-----|
| CRITICAL | Tauri loaded index.html (old Electron page) instead of index2.html | Renamed index2.html → index.html |
| CRITICAL | Bridge only had 13 methods, frontend needed 43 | Added all 43 methods to tauri-bridge.js |
| CRITICAL | Chat param mismatch: sessionId (JS) vs session_id (Rust) | Bridge now maps sessionId → session_id |
| HIGH | Missing "manager" model key in settings.json | Added "manager": "qwen3.5:latest" |
| MEDIUM | fallbackChain config is dead code (brain-router hardcodes) | Noted for Day 5 port to Rust |

## Bugs Found & Fixed (Day 2 Verification)

| Severity | Issue | Fix |
|----------|-------|-----|
| CRITICAL | `clear_history` bridge sent `session_id` but Rust took no params | Removed param from bridge call |
| CRITICAL | `set_mode` bridge sent `mode` key, Rust expected `mode_name` | Fixed bridge key to `mode_name` |
| CRITICAL | `create_goal` bridge sent `{ goal }` object, Rust expected flat params | Fixed bridge to send `title, description, priority, category` |
| CRITICAL | `add_folder` bridge sent `{ name }` only, Rust required `{ path, name }` | Fixed bridge to send both `path` and `name` |
| HIGH | `select_folder` bridge sent `{ id }` but Rust took no params (placeholder dialog) | Removed param from bridge call |

---

## Priority Rule

**Tauri migration is the #1 priority. No exceptions.**

- Backend (Rust) is the foundation — everything depends on it
- No UI polish, no logo changes, no visual redesign until migration is complete
- Every session focuses on Rust porting until Day 16 is done
- UI redesign happens AFTER Electron is deleted (post-Day 16 or separate sprint)

---

*Last updated: 2026-06-12*
