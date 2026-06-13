# ORION — PLAN AND STRUCTURE
## Master Document for All ORION Plans, Architecture & Progress

---

## Project Overview

**ORION** — Personal AI Assistant & Multi-Agent Operator
- Real-life JARVIS equivalent
- Tauri v2 desktop app (Rust backend, HTML/CSS/JS frontend)
- 8-week roadmap: reqwest → Voice → Vision → Action
- Built by Abhi (solo developer)

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | HTML/CSS/JS + Tauri IPC bridge |
| Backend | Rust (Tauri v2) |
| Database | rusqlite (native SQLite) |
| HTTP Client | Node.js http-bridge.js → **reqwest (in progress)** |
| Window | WebView2 (~30MB) |
| CI/CD | GitHub Actions — auto-release on push (MSI + NSIS) |

---

## Architecture

```
Frontend (UI/) → tauri-bridge.js → Tauri IPC → Rust backend (lib.rs)
                                                    │
                                          CORE modules (src-tauri/src/core/)
                                                    │
                                          Node.js http-bridge.js ──→ Cloud APIs
                                                    │
                                          Ollama (local) ──→ Local models
```

---

## Migration Complete ✅ — Current State (2026-06-12)

**All CORE modules ported to Rust ✅** — 13 modules in `src-tauri/src/core/`:
- ai_brain, brain_router, personality_engine, agent_registry, agent
- context_manager, argument_engine, suggestion_engine, goal_tracker
- user_profile, ollama_brain, constants

**61/61 IPC commands verified ✅**
**Clean builds ✅** — `cargo check`, `cargo clippy`, `cargo build --release` all pass
**Vercel-style UI redesign applied ✅**
**GitHub Actions CI/CD ✅** — auto-release on push to main

---

## The 5 JARVIS Pillars — Where ORION Stands

| # | Pillar | Description | ORION Status |
|---|--------|-------------|-------------|
| 1 | **Brain** 🧠 | Think, answer, make decisions | ✅ Multi-brain routing, personality engine, agents |
| 2 | **Voice** 🎤 | Natural speech I/O, not robot voice | ❌ **MISSING** — text only |
| 3 | **Memory** 📝 | Remember user, preferences, goals | ✅ user_profile, goal_tracker, context_manager |
| 4 | **Vision** 👁️ | See & understand screens, images, documents | ❌ **MISSING** |
| 5 | **Action** ⚡ | Execute tasks — files, emails, system control | ❌ **MISSING** — chatbot only |

---

## 8-Week Roadmap: reqwest → Voice → Vision → Action

### Week 1 — Foundation (reqwest Integration)
**Goal:** Kill the last Node.js dependency. ORION becomes fully self-contained Rust binary.

| Day | Task | Deliverable |
|-----|------|-------------|
| Day 1 | Add reqwest + tokio deps to Cargo.toml. Port `http-bridge.js` HTTP logic to native Rust in `ai_brain.rs`. | reqwest working for Groq API calls |
| Day 2 | Port OpenRouter + Gemini API calls to Rust. Remove Node.js bridge references. | All 3 API routes native Rust |
| Day 3 | Port Ollama HTTP calls (if using HTTP, not CLI). Handle streaming SSE responses natively. | Streaming works in Rust |
| Day 4 | Remove `node_modules` (319MB) from tracking & disk. Delete `http-bridge.js`. Clean up CORE/ dir. | node_modules gone, repo clean |
| Day 5 | End-to-end test: launch ORION, send chat, verify all 3 API providers work without Node. | Full test pass |
| Day 6 | `cargo build --release` + push to main. GitHub Actions auto-builds zero-dependency .exe. | Release with fully self-contained binary |
| Day 7 | Buffer day / catch-up / bug fixes from reqwest integration | |

### Week 2 — Voice: Foundation (STT + TTS)
**Goal:** ORION can hear and speak.

| Day | Task | Deliverable |
|-----|------|-------------|
| Day 1 | Add TTS system — Web Speech API or Tauri shell plugin. Basic `speak(text)` IPC command. | ORION speaks its first words |
| Day 2 | Voice waveform UI — mic button, recording indicator, audio visualizer in frontend | Visual feedback for voice |
| Day 3 | STT (Speech-to-Text) — Web Speech API or Whisper integration. `listen()` → returns text | ORION hears you |
| Day 4 | Wire voice input into chat flow: listen → transcribe → send as message → get reply → speak | Full voice chat loop |
| Day 5 | Voice activation — wake word or push-to-talk. Persistent mic button in UI | Hands-free trigger |
| Day 6 | Polish: voice settings panel (select mic, speed, voice type), error handling | Settings + robustness |
| Day 7 | Test + release | |

### Week 3 — Voice: Advanced (Natural Speech)
**Goal:** Voice feels natural — tone, interruptions, personality.

| Day | Task | Deliverable |
|-----|------|-------------|
| Day 1 | Multi-voice support per mode (ORION=deep, FRIDAY=playful, JARVIS=british) | Mode-tied voices |
| Day 2 | Interruption handling — mic stays hot, new speech cancels current TTS | Natural conversation flow |
| Day 3 | Emotion/prosody in speech — varies tone based on response intent | Not robotic |
| Day 4 | Background noise handling, voice activity detection | Reliable in real environments |
| Day 5 | Voice command mode — "ORION, open Chrome" bypasses chat | Direct action trigger |
| Day 6 | Polish + integration test | |
| Day 7 | Release | |

### Week 4 — Vision: Foundation (Screen + Image Understanding)
**Goal:** ORION can see your screen and images.

| Day | Task | Deliverable |
|-----|------|-------------|
| Day 1 | Screen capture — Tauri screenshot plugin or OS-level capture. `capture_screen()` IPC | ORION takes a screenshot |
| Day 2 | Image analysis pipeline — send screenshot to vision-capable model (GPT-4o / Gemini Vision / Llama 3.2 Vision) | ORION describes what it sees |
| Day 3 | Image input in chat — drag/drop or paste images, send to vision model | You can show ORION pictures |
| Day 4 | Document understanding — send PDF/DOCX as image frames, extract text + layout | ORION reads documents |
| Day 5 | Frontend: image preview in chat, loading states, zoom | Smooth UX |
| Day 6 | Polish + test with real scenarios (code screenshots, error dialogs, documents) | |
| Day 7 | Release | |

### Week 5 — Vision: Advanced (Computer Use)
**Goal:** ORION can interact with what it sees.

| Day | Task | Deliverable |
|-----|------|-------------|
| Day 1 | Element detection — identify buttons, links, inputs from screenshot | ORION knows UI elements |
| Day 2 | Click simulation — translate "click the X button" into screen coordinates + mouse click | ORION clicks things |
| Day 3 | Type simulation — send keystrokes to focused element | ORION types for you |
| Day 4 | Basic automation flow: "Open Chrome, go to gmail, search for invoice" | Multi-step task via vision |
| Day 5 | Scrolling, navigation, window management | Full screen control |
| Day 6 | Safety guardrails — approval prompts before destructive actions | Safe to use |
| Day 7 | Integration test + release | |

### Week 6 — Action: Foundation (File + System)
**Goal:** ORION does things, not just talks.

| Day | Task | Deliverable |
|-----|------|-------------|
| Day 1 | File operations — read, write, move, delete files via natural language | "Create a folder called Projects" works |
| Day 2 | Application launcher — open apps, switch windows, minimize/maximize | "Open VS Code" works |
| Day 3 | Clipboard operations — copy/paste text between apps | ORION can transfer data |
| Day 4 | Browser automation — open tabs, navigate, fill forms (via vision + action combo) | ORION uses the web for you |
| Day 5 | Notifications system — ORION can send push alerts | Background alerts |
| Day 6 | System info — battery, CPU, RAM, network status, uptime | ORION knows the system state |
| Day 7 | Test + release | |

### Week 7 — Action: Advanced (Email + Tasks + Automation)
**Goal:** ORION is a true productivity assistant.

| Day | Task | Deliverable |
|-----|------|-------------|
| Day 1 | Email integration — send, read, search emails (IMAP/SMTP via Rust) | "Send an email to dad" works |
| Day 2 | Task execution engine — parse "do X" → plan steps → execute sequentially | Task pipeline |
| Day 3 | Scheduled actions — "remind me at 5pm to call the client" with actual execution | Time-based actions |
| Day 4 | Webhook/system command execution — ORION can run scripts, trigger APIs | Integrates with anything |
| Day 5 | Custom action recipes — user-defined multi-step macros | "Deploy the app" = 20 steps, 1 command |
| Day 6 | Safety controls — permission levels, action approval queue, undo | Trustworthy executor |
| Day 7 | Integration test + release | |

### Week 8 — Polish + Integration + Release
**Goal:** Everything works together. Ship ORION JARVIS v1.0.

| Day | Task | Deliverable |
|-----|------|-------------|
| Day 1 | Cross-pillar integration — voice + vision + action in one flow. "ORION, see this error and fix it" | All pillars work together |
| Day 2 | Auto-updater (Tauri built-in) — ORION checks for updates on launch | Self-updating |
| Day 3 | System tray integration — background operation, quick menu | Always available |
| Day 4 | Startup — ORION launches on Windows boot | Always ready |
| Day 5 | Performance profiling — memory, startup time, response latency | Known metrics |
| Day 6 | Final bug bash — edge cases, error states, crash recovery | Rock solid |
| Day 7 | v1.0 Release — tag, release notes, installer | 🚀 SHIP IT |

---

## How These Pillars Map to the JARVIS Video (GrowthOS Telugu)

The video defines 5 systems that must work together for a real JARVIS:

1. **Brain** (మెదడు) — Think, answer, decide → ✅ DONE
2. **Voice** (వాయిస్) — Natural speech, assistant-like → Week 2-3
3. **Memory** (మెమరీ) — Remember everything about you → ✅ DONE
4. **Vision** (విజన్) — See world, screens, docs → Week 4-5
5. **Action** (యాక్షన్) — DO things, not just talk → Week 6-7

"వీటిలో ఒకటి ఫెయిల్ అయిన ఎంటైర్ ప్రాజెక్ట్ ఫెయిల్ అవుతుంది"
— If even one fails, the entire project fails.

---

## Week 1 — Detailed Plan (Foundation: reqwest Integration)

### Prerequisites
- Rust toolchain with `cargo` ✅
- `C:\ORION\` repo on `main` branch ✅
- Cargo.toml with existing deps (tauri 2, rusqlite, serde, chrono) ✅
- Builds clean (`cargo check`, `cargo clippy`, `cargo build --release`) ✅

### Day 1: Add reqwest + Port Groq HTTP Calls

**Files to modify:**
- `src-tauri/Cargo.toml` — add reqwest + tokio deps
- `src-tauri/src/core/ai_brain.rs` — replace Node.js bridge calls with native HTTP
- `src-tauri/src/lib.rs` — update command handlers if needed

**Step-by-step:**
1. `cargo add reqwest --features json,stream`
2. `cargo add tokio --features full`
3. Read `CORE/http-bridge.js` fully to understand the HTTP request format
4. Create `src-tauri/src/core/http_client.rs` — native reqwest wrapper for Groq API
5. Wire streaming SSE response handling (each chunk forwarded via Tauri event)
6. `cargo check` — fix compile errors
7. Test: send a chat and verify Groq responds without Node.js process running

**Implementation notes:**
- http-bridge.js accepts `BridgeRequest { action, model, messages, ... }` JSON via stdin
- stdout returns SSE chunks: `data: {"text": "...", "done": false}\n\n`
- Replace with `reqwest::Client::post(url).json(&body).send().await`
- Stream response with `response.bytes_stream()` → parse SSE chunks → emit Tauri events
- Fallback: keep the bridge as optional fallback initially, switch via config flag

### Day 2: Port OpenRouter + Gemini HTTP Calls

**Files to modify:**
- `src-tauri/src/core/http_client.rs` — add OpenRouter + Gemini endpoints
- `src-tauri/src/core/brain_router.rs` — update routing to use native HTTP

**Step-by-step:**
1. Add OpenRouter endpoint (similar structure to Groq, different URL + auth header)
2. Add Gemini endpoint (different auth — API key as query param)
3. Update `brain_router.rs` to call native http_client instead of bridge
4. Remove bridge spawn logic from ai_brain.rs — no more `Command::new("node")`
5. `cargo check` + clippy
6. Test with all 3 providers

### Day 3: Port Ollama + SSE Streaming

**Files to modify:**
- `src-tauri/src/core/http_client.rs` — Ollama HTTP endpoint (localhost:11434)
- `src-tauri/src/core/ollama_brain.rs` — update to use reqwest

**Step-by-step:**
1. Ollama API is `POST http://localhost:11434/api/chat` — add to http_client
2. Ensure streaming SSE from Ollama works the same way as cloud APIs
3. Remove `Command::new("ollama")` subprocess in ollama_brain.rs
4. `cargo check` + clippy
5. Test all 4 providers with streaming

### Day 4: Clean Up Node.js Files

**Files to modify:**
- Remove `CORE/http-bridge.js`
- `.gitignore` — ensure node_modules/ is in it
- UI/tauri-bridge.js — remove Node bridge references (if any)

**Step-by-step:**
1. `git rm CORE/http-bridge.js`
2. Check `.gitignore` has `node_modules/`
3. `rm -rf node_modules` (if still on disk)
4. Search for any remaining Node bridge references in codebase
5. Clean up `package.json` if still present (not needed anymore)

### Day 5: End-to-End Testing

**Test scenarios:**
1. Launch ORION (`cargo run` or existing binary)
2. Send a chat message → expect reply from Groq
3. Switch personality modes → verify voice (if wired) or text response changes
4. Test OpenRouter → expect Claude/Grok response
5. Test Gemini → expect Gemini response
6. Test Ollama → expect local model response
7. Try conversation history → memory persists across messages
8. Try offline mode → graceful fallback if no internet

### Day 6: Release Build + Push

1. `cargo build --release`
2. `cargo clippy` — zero warnings
3. Run the release binary — test chat
4. `git add . && git commit -m "build #22: reqwest integration — Node bridge removed"`
5. `git tag v3.0.0-b22`
6. `git push origin main --tags`
7. Verify GitHub Actions auto-release creates installer with reqwest-based binary

### Day 7: Buffer / Bug Fixes

**Known risks:**
- TLS/SSL cert issues on Windows with reqwest — may need `native-tls` or `rustls` feature
- SSE streaming format differences between providers — Groq uses `data: {...}\n\n`, Gemini uses different format
- Cargo.lock merge conflicts if deps change
- Binary size increase from reqwest (vs Node.js subprocess)

---

## Priority Rule

**Week 1 is the hard prerequisite.** reqwest integration removes the last Node.js dependency, making ORION fully self-contained. Nothing else can proceed smoothly without this foundation.

After Week 1, Abhi can choose the order — but the recommended sequence is:
1. **Voice** (Weeks 2-3) — Biggest UX impact, highest perceived value
2. **Vision** (Weeks 4-5) — Foundation for computer use
3. **Action** (Weeks 6-7) — Requires vision to be most useful
4. **Integration** (Week 8) — Polish and ship

---

## Performance Targets

| Metric | Current | Target (Week 8) |
|--------|---------|-----------------|
| RAM (idle) | ~30-50MB | ~50-80MB |
| Startup time | ~1s | ~1s |
| Bundle size | 14MB | ~20MB (with reqwest, vision libs) |
| CPU idle | ~0% | ~0% |

---

*Last updated: 2026-06-12 (8-week roadmap added)*
