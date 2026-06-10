# ORION Phase A — Branding & UI/UX Overhaul

> **Week:** June 8-14, 2026
> **Goal:** Transform ORION from functional prototype to polished desktop app with real brand identity
> **Constraint:** WSL2 for dev, GA for Windows builds. No local Rust compilation (WDAC).

---

## Current State

| Aspect | Current | Target |
|--------|---------|--------|
| Logo | `◈` diamond emoji | Custom SVG logo (ORION wordmark + icon) |
| Theme | Dark blue (#0a0e17) + cyan accent (#00f0ff) | Refined palette, glass-morphism, consistent spacing |
| Sidebar Icons | Emoji (🏠 📂 etc.) | Custom SVG icons or Unicode symbols |
| Layout | 3-column (sidebar, center+chat, right) | Same layout, polished spacing & transitions |
| CSS | Two overlapping files (orion-3.css 1630L + orion-theme.css 1362L) | Single clean CSS with CSS variables |
| Animations | Minimal (fadeIn, slideIn) | View transitions, skeleton loading, micro-interactions |
| Dashboard | Hero with ◈, basic quick cards, stats | Real logo, animated stats, polished cards |
| Chat UI | Basic message bubbles, emoji action buttons | Styled bubbles, better input area, voice button design |
| Views | 10 views (basic cards/grids) | Consistent card design, loading states, hover effects |
| Right Panel | Collapsed by default, basic status cards | Collapsible with smooth transition, richer content |

**Files involved:**
- `UI/index.html` (547 lines)
- `UI/styles/orion-3.css` (1630 lines)
- `UI/styles/orion-theme.css` (1362 lines) — possibly merge into orion-3.css
- `UI/styles/animations.css` (39 lines)
- `UI/styles/status-indicator.css` (1875 lines)
- `UI/orion-3.js` (1230 lines)
- `UI/renderer.js` (1222 lines)
- `UI/tauri-bridge.js` (224 lines)
- `UI/assets/icon.png` — the app icon (currently was JPEG mislabeled as PNG)
- `src-tauri/icons/` — all build-time icons

---

## Day 1 — Logo & Brand Identity

**Objective:** Create ORION's visual identity — a proper logo (icon + wordmark) and refined color palette.

**Files:**
- Create: `UI/assets/orion-logo.svg`
- Create: `UI/assets/orion-icon.svg`
- Create: `UI/assets/orion-wordmark.svg`
- Modify: `UI/styles/orion-3.css` (update CSS variables, add logo styles)
- Modify: `UI/index.html` (replace ◈ with SVG logo in titlebar & dashboard)
- Modify: `src-tauri/icons/icon.png` (update with real logo icon)
- Modify: `src-tauri/icons/icon.ico` (regenerate from logo)

### Task 1.1: Design the ORION Logo SVG

Create `UI/assets/orion-logo.svg` — a geometric/tech-inspired logo. Think:
- "ORION" as a constellation-like symbol (dots + connecting lines)
- Or a stylized "O" with orbital rings
- Or a geometric eye/star motif

**Design direction for the logo:**
- Geometric, minimal, tech-forward
- Colors: cyan (#00f0ff) and deep space (#0a0e17)
- Constellation theme: 4-5 connected nodes forming a star pattern with "ORION" name

### Task 1.2: Refine Color Palette

Update CSS variables in `orion-3.css` with a cohesive palette:

```
--bg-primary: #070b14        (deeper space)
--bg-secondary: #0d1225      (slightly lighter)
--bg-tertiary: #141b33       (card backgrounds)
--bg-glass: rgba(13, 18, 37, 0.85)
--border-subtle: rgba(0, 240, 255, 0.12)
--border-active: rgba(0, 240, 255, 0.4)
--accent-primary: #00f0ff    (keep — works well)
--accent-secondary: #7c3aed  (purple accent)
--accent-glow: rgba(0, 240, 255, 0.2)
--text-primary: #eef5ff
--text-secondary: #8899cc
--text-muted: #4a5a7a
```

### Task 1.3: Replace Emoji Icons in Sidebar

Replace emoji in `index.html` sidebar with clean Unicode symbols or SVG:

```
Dashboard → ◉  (U+25C9)
Projects  → ▦  (U+25A6)
Tasks     → ☐  (U+2610)
Clients   → 👤
Leads     → ▲
Systems   → ⚡
Calendar  → 📅
Analytics → ▤  (U+25A4)
Logs      → ≡  (U+2261)
Settings  → ⚙
```

### Task 1.4: Generate New App Icon

- Use the logo SVG to generate:
  - `src-tauri/icons/icon.png` (1024x1024 RGBA PNG)
  - `src-tauri/icons/icon.ico` (multi-size ICO)
  - `src-tauri/icons/32x32.png`
  - `src-tauri/icons/128x128.png`
  - `src-tauri/icons/128x128@2x.png`

**Verification:** `file src-tauri/icons/icon.png` shows "PNG image data, 1024x1024, 8-bit/color RGBA"

### Task 1.5: Wire Logo into Titlebar & Dashboard

In `index.html`:
- Titlebar: Replace `<span class="logo-icon">◈</span>` with `<img class="logo-icon" src="assets/orion-icon.svg">`
- Dashboard hero: Replace `<div class="hero-icon">◈</div>` with `<img class="hero-logo" src="assets/orion-logo.svg">`

---

## Day 2 — Theme Consolidation & View Transitions

**Objective:** Clean up the CSS mess, add smooth transitions between views, skeleton loading states.

**Files:**
- Modify: `UI/styles/orion-3.css` (major rewrite for clean, organized CSS)
- Modify: `UI/styles/animations.css` (add view transitions, skeleton animations)
- Modify: `UI/styles/status-indicator.css` (refine for new theme)
- Modify: `UI/orion-3.js` (add view transition logic)
- Option: Delete `UI/styles/orion-theme.css` (merge into orion-3.css)

### Task 2.1: Merge & Clean CSS

`orion-3.css` (1630L) and `orion-theme.css` (1362L) overlap heavily. Merge into `orion-3.css`:
- Take the refined color palette from orion-theme.css
- Take the layout/structure from orion-3.css
- Remove all duplicates
- Reorganize by sections with clear comments
- Target: ~2000 lines clean, well-organized CSS

**Section organization:**
1. CSS Variables
2. Reset & Base
3. Scrollbar
4. Title Bar
5. Sidebar (Left)
6. Center Panel
7. Views (each view gets a subsection)
8. Chat Section
9. Right Panel
10. Modals
11. Animations
12. Utility classes

### Task 2.2: Add View Transition Animations

In `animations.css`, add:

```css
/* View transitions */
@keyframes viewEnter {
    from { opacity: 0; transform: translateY(12px) scale(0.98); }
    to { opacity: 1; transform: translateY(0) scale(1); }
}

@keyframes viewExit {
    from { opacity: 1; transform: translateY(0) scale(1); }
    to { opacity: 0; transform: translateY(-8px) scale(0.98); }
}

.view.active {
    animation: viewEnter 0.25s cubic-bezier(0.16, 1, 0.3, 1) forwards;
}

.view.exit {
    animation: viewExit 0.2s ease forwards;
}
```

In `orion-3.js`, modify `switchView()` function:
- Add exit animation class to current view
- Wait 150ms, then add active class to new view
- Use requestAnimationFrame for smooth timing

### Task 2.3: Skeleton Loading States

Add CSS skeleton animations and apply to loading containers:

```css
@keyframes shimmer {
    0% { background-position: -200% 0; }
    100% { background-position: 200% 0; }
}

.skeleton {
    background: linear-gradient(90deg, 
        var(--bg-tertiary) 25%, 
        rgba(0, 240, 255, 0.05) 50%, 
        var(--bg-tertiary) 75%
    );
    background-size: 200% 100%;
    animation: shimmer 1.5s ease-in-out infinite;
    border-radius: var(--radius-sm);
}
```

Replace "Loading..." text placeholders in views with skeleton elements:
- `#projects-container` > skeleton cards
- `#clients-list` > skeleton rows
- `#recent-sessions-list` > skeleton items

### Task 2.4: Add Micro-interactions

- Button hover: scale(1.02) + brighter glow
- Card hover: lift 2px + border glow
- Input focus: subtle ring animation
- Sidebar item active: left border indicator

---

## Day 3 — Layout Refinement

**Objective:** Polished sidebar, responsive behavior, right panel animations.

**Files:**
- Modify: `UI/styles/orion-3.css` (layout sections)
- Modify: `UI/index.html` (maybe minor tweaks)
- Modify: `UI/orion-3.js` (sidebar toggle, divider resize)

### Task 3.1: Sidebar Polish

- Expand/collapse with smooth width transition (CSS transition, not JS)
- Active nav item: left accent bar + subtle glow
- Tooltip on collapsed state (show on hover over icon)
- Bottom user avatar: circular, subtle border, status dot
- Sidebar toggle button: animated rotate icon

### Task 3.2: Divider Resize Polish

The dividers between sidebar|center and center|right panel:
- Thicker hover area (6px) for easier grab
- Cursor change on hover
- Snap-to-edge when dragged close to minimum
- Smooth transition when panel collapses

### Task 3.3: Right Panel Animation

The right panel currently collapses with `hidden` class (display: none — no animation):
- Use CSS `width` transition instead
- Slide in/out with 0.25s ease
- Content fades in after panel opens

---

## Day 4 — Dashboard & Views Redesign

**Objective:** Make the dashboard impressive, polish all view cards.

**Files:**
- Modify: `UI/styles/orion-3.css` (view-specific styles)
- Modify: `UI/index.html` (dashboard section)
- Modify: `UI/orion-3.js` (dashboard data loading)

### Task 4.1: Dashboard Hero

Current hero has ◈ + "ORION" + subtitle. Refresh to:

```
┌─────────────────────────────────────┐
│         [ORION LOGO SVG]            │
│          ORION                       │
│     Your personal AI command center  │
│                                       │
│   Status: ● ONLINE    Brain: Groq   │
│   Memory: ● ACTIVE    Mode: ORION   │
└─────────────────────────────────────┘
```

The status info should be live data from Rust backend.

### Task 4.2: Animated Stats Cards

The mini-stats (Sessions, Messages, Goals) at bottom of dashboard:
- Animated counter on load (count up from 0 to actual value)
- Each card gets a subtle top-border accent color
- Hover: slight lift + glow matching the accent

### Task 4.3: Quick Action Cards

The 3 quick cards (New Project, Open Project, New Chat):
- Consistent icon + title + description layout
- Hover: lift + border glow (cyan for New Project, purple for Open, green for Chat)
- Loading skeleton state for when DB is initializing

### Task 4.4: Recent Chats List

- Each item: title, preview (first 50 chars), timestamp, message count
- Hover: background highlight + subtle scale
- Click: loads that session's chat

---

## Day 5 — Chat UI Refinements & Final Polish

**Objective:** Beautiful, usable chat interface.

**Files:**
- Modify: `UI/styles/orion-3.css` (chat section)
- Modify: `UI/index.html` (chat section tweaks)
- Modify: `UI/renderer.js` (message rendering)
- Modify: `UI/orion-3.js` (maybe)

### Task 5.1: Message Bubbles

Current: basic divs with color.
Target:
- User messages: right-aligned, accent color background, slight glow
- ORION messages: left-aligned, glass-morphism background, subtle border
- Code blocks: monospace, dark background, syntax-aware styling
- Timestamps: small, muted, on hover
- Enter animation: slide in from respective side

### Task 5.2: Input Area

Current: basic textarea + action buttons.
Target:
- Textarea: auto-growing, glass-morphism background, subtle border on focus
- Send button: cyan gradient, pulse on hover
- Mic button: styled consistently, recording state animation
- Voice output toggle: consistent with mic
- Hint text below: "ORION is thinking..." with animated dots

### Task 5.3: Typing Indicator

Enhance the 3-dot animation:
- Dots: expanding/contracting with staggered delay
- Text: "ORION is thinking..." with subtle fade
- Smooth transition when ORION starts/stops responding

### Task 5.4: Chat Divider

The divider between the view panel and chat section:
- Visual connection to the panel system
- Drag handle for resizing
- Marker showing it's chat area

---

## How to Work This Week

**Dev loop (WSL2):**
```bash
wsl
cd /mnt/c/ORION
cargo tauri dev
```
→ ORION window opens, hot-reloads on any UI file change. Edit CSS → refresh → see instantly.

**No WSL2?** You can still edit UI files directly in VS Code/notepad and test by running the installed ORION (it loads from `C:\Users\telug\AppData\Local\ORION\`). But changes won't show until you rebuild. The WSL2 dev loop is 10x faster for UI work.

**For Windows releases:**
Push to GitHub → GA builds → download `.exe` from Actions. Only needed once per milestone.

---

## Risks & Notes

1. **CSS Merge (Day 2):** orion-3.css and orion-theme.css have overlapping styles. Merge carefully — test each view after merge.
2. **SVG Logo:** If you can't design a logo yourself, I can generate the SVG code for you. Just tell me the style direction.
3. **App Icon:** I need to regenerate the ICO file. On Windows, `ffmpeg` can do PNG→ICO or I can use a Windows tool.
4. **No backend changes needed** — this is 100% frontend work. All Rust backend stays untouched.
5. **Time budget:** 1-2 hours/day max. Each day's task is designed to fit in that window.

---

## Verification

After each day:
- [ ] `cargo check` passes (no Rust compilation errors)
- [ ] UI loads without console errors
- [ ] All views render correctly
- [ ] Chat works (send + receive messages)
- [ ] Sidebar navigation works
- [ ] Git commit with descriptive message
