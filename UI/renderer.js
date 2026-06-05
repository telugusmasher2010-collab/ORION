/**
 * ORION — Renderer (UI Logic) v2.0
 * Enhanced with mode switching, goals, and suggestions
 */

// Custom modal system (replaces native prompt/confirm/alert — disabled in Electron 35+)
let _promptResolve = null;
let _confirmResolve = null;
let _alertResolve = null;

function orionPrompt(title, message, defaultValue = '') {
    return new Promise((resolve) => {
        _promptResolve = resolve;
        document.getElementById('prompt-modal-title').textContent = title;
        document.getElementById('prompt-modal-message').textContent = message;
        const input = document.getElementById('prompt-modal-input');
        input.value = defaultValue;
        document.getElementById('prompt-modal').classList.remove('hidden');
        setTimeout(() => { input.focus(); input.select(); }, 100);
    });
}

function promptModalConfirm() {
    const value = document.getElementById('prompt-modal-input').value;
    document.getElementById('prompt-modal').classList.add('hidden');
    if (_promptResolve) { _promptResolve(value); _promptResolve = null; }
}

function promptModalCancel() {
    document.getElementById('prompt-modal').classList.add('hidden');
    if (_promptResolve) { _promptResolve(null); _promptResolve = null; }
}

function orionConfirm(title, message) {
    return new Promise((resolve) => {
        _confirmResolve = resolve;
        document.getElementById('confirm-modal-title').textContent = title;
        document.getElementById('confirm-modal-message').textContent = message;
        document.getElementById('confirm-modal').classList.remove('hidden');
    });
}

function confirmModalConfirm() {
    document.getElementById('confirm-modal').classList.add('hidden');
    if (_confirmResolve) { _confirmResolve(true); _confirmResolve = null; }
}

function confirmModalCancel() {
    document.getElementById('confirm-modal').classList.add('hidden');
    if (_confirmResolve) { _confirmResolve(false); _confirmResolve = null; }
}

function orionAlert(title, message) {
    return new Promise((resolve) => {
        _alertResolve = resolve;
        document.getElementById('alert-modal-title').textContent = title;
        document.getElementById('alert-modal-message').textContent = message;
        document.getElementById('alert-modal').classList.remove('hidden');
    });
}

function alertModalClose() {
    document.getElementById('alert-modal').classList.add('hidden');
    if (_alertResolve) { _alertResolve(); _alertResolve = null; }
}

// Make modal functions globally available
window.orionBridgePrompt = orionPrompt;
window.orionBridgeConfirm = orionConfirm;
window.orionBridgeAlert = orionAlert;
window.promptModalConfirm = promptModalConfirm;
window.promptModalCancel = promptModalCancel;
window.confirmModalConfirm = confirmModalConfirm;
window.confirmModalCancel = confirmModalCancel;
window.alertModalClose = alertModalClose;

let isProcessing = false;
let currentStreamingMessage = null;
let activeView = 'dashboard';
let isListening = false;
let isSpeaking = false;
let recognition = null;
let speechSynth = window.speechSynthesis;
let currentUtterance = null;
let currentMode = 'orion';

// Global UI Functions
window.createNewChat = createNewChat;
window.showView = showView;
window.sendMessage = sendMessage;
window.clearChat = clearChat;
window.addMessage = addMessage;
window.forceUnlock = forceUnlock;
window.toggleSettings = toggleSettings;
window.quickStart = quickStart;
window.toggleVoiceInput = toggleVoiceInput;
window.toggleVoiceOutput = toggleVoiceOutput;
window.stopSpeaking = stopSpeaking;
window.switchMode = switchMode;
window.createGoal = createGoal;
window.showCreateProject = showCreateProject;
window.loadProjects = loadProjects;
window.loadFolders = loadFolders;
window.filterChats = filterChats;
window.addFolder = addFolder;
window.setActiveFolder = setActiveFolder;
window.sendFolderToScrap = sendFolderToScrap;
window.showForgottenFolders = showForgottenFolders;
window.showGoalInput = showGoalInput;
window.switchSession = switchSession;
window.loadHistory = loadHistory;
window.loadSessions = loadSessions;

document.addEventListener('DOMContentLoaded', async () => {
    logSystem('Initializing ORION...');
    setupInputHandlers();
    initVoiceSystem();
    setupResizeHandler();

    // Setup prompt modal keyboard handling
    const promptInput = document.getElementById('prompt-modal-input');
    if (promptInput) {
        promptInput.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') { e.preventDefault(); promptModalConfirm(); }
            if (e.key === 'Escape') { promptModalCancel(); }
        });
    }

    // Setup streaming listener
    window.orionBridge.onChunk((chunk) => {
        updateStreamingMessage(chunk);
    });

    // Plain start — show dashboard, don't auto-load last session
    // User picks a project or creates a new chat

    // Defer background data loading
    setTimeout(async () => {
        await loadAgentStatus();
        await loadOllamaStatus();
        logSystem('ORION ready. Namaskaram Abhi!');
    }, 200);
});

/**
 * View Manager
 */
function showView(viewId) {
    // ORION 3.0 uses switchView for view management
    if (typeof window.switchView === 'function') {
        if (viewId !== 'chat-area') {
            window.switchView(viewId);
        }
        return;
    }

    // Legacy view system (for old index.html)
    const views = document.querySelectorAll('.view');
    views.forEach(v => v.classList.add('hidden'));
    
    const target = document.getElementById(viewId);
    if (target) {
        target.classList.remove('hidden');
        activeView = viewId;
    }
}

/**
 * Quick Start from Dashboard
 */
async function quickStart(type) {
    try {
        let sessionId = await window.orionBridge.getCurrentSessionId();

        // Create new session if none exists
        if (!sessionId) {
            sessionId = await window.orionBridge.createSession('New Chat', 1);
            await window.orionBridge.switchSession(sessionId);
            await loadSessions();
        }

        showView('chat-area');
        const prompts = {
            code: "What do you want to build or code today?",
            business: "Which business are we working on? YouTube, Shopify, SaaS, or something else?",
            schedule: "What reminder or task do you want to set?",
            chat: "What's on your mind, Abhi?"
        };
        addMessage('orion', prompts[type] || "How can I help?");
    } catch (e) {
        console.error('Error in quickStart:', e);
    }
}

/**
 * Load Chat History — deferred, batch DOM insertion
 */
async function loadHistory() {
    try {
        const sessionId = await window.orionBridge.getCurrentSessionId();
        if (!sessionId) return;
        const history = await window.orionBridge.getHistory(sessionId);
        const messagesDiv = document.getElementById('messages');
        
        if (!history || history.length === 0) {
            return;
        }

        // Use DocumentFragment for batch DOM insertion (much faster)
        const fragment = document.createDocumentFragment();
        
        history.forEach(msg => {
            const div = document.createElement('div');
            const isOrion = msg.role === 'assistant';
            div.className = `message ${isOrion ? 'orion-message' : 'user-message'} fade-in`;
            
            const header = document.createElement('div');
            header.className = 'message-header';
            
            const author = document.createElement('span');
            author.className = 'message-author';
            author.textContent = isOrion ? '◆ ORION' : '● ABHI';
            
            const meta = document.createElement('span');
            meta.className = 'message-meta';
            meta.textContent = msg.timestamp ? new Date(msg.timestamp).toLocaleTimeString() : new Date().toLocaleTimeString();
            
            header.appendChild(author);
            header.appendChild(meta);
            
            const content = document.createElement('div');
            content.className = 'message-content';
            content.textContent = msg.content; // XSS-safe: use textContent, not innerHTML
            
            div.appendChild(header);
            div.appendChild(content);
            fragment.appendChild(div);
        });

        messagesDiv.appendChild(fragment);
        scrollToBottom();
        logSystem(`Restored ${history.length} messages.`);
    } catch (e) {
        logSystem('Error loading history: ' + e.message);
    }
}

/**
 * Create New Chat — creates new session in DB
 */
async function createNewChat(projectId = 1) {
    try {
        const sessionId = await window.orionBridge.createSession('New Chat', projectId);
        if (!sessionId) {
            console.error('Failed to create session');
            return;
        }
        await window.orionBridge.switchSession(sessionId);
        document.getElementById('messages').innerHTML = '';
        addMessage('orion', "New session started, Abhi. What's the goal?");

        // Navigate to projects view so user can see all sessions
        if (typeof switchView === 'function') {
            switchView('projects');
        }
    } catch (e) {
        console.error('Error creating chat:', e);
    }
}

/**
 * Send Message
 */
async function sendMessage() {
    const input = document.getElementById('message-input');
    if (!input) {
        console.error('Message input not found');
        return;
    }

    const message = input.value.trim();
    if (!message || isProcessing) return;

    input.value = '';
    input.style.height = 'auto';
    addMessage('user', message);

    currentStreamingMessage = createStreamingMessage();
    setProcessing(true);

    // Show typing indicator
    const indicator = document.getElementById('typing-indicator');
    if (indicator) indicator.classList.remove('hidden');

    try {
        console.log('[Renderer] Starting chat flow...');

        // Track active task from message
        const taskIntent = extractTaskIntent(message);
        if (taskIntent) {
            await window.orionBridge.setActiveTask(taskIntent);
        }

        let currentSessionId = await window.orionBridge.getCurrentSessionId();
        console.log('[Renderer] Session ID:', currentSessionId);

        // Ensure we have a valid session
        if (!currentSessionId || currentSessionId === null) {
            console.log('[Renderer] No session, creating one...');
            currentSessionId = await window.orionBridge.createSession('New Chat', 1);
            console.log('[Renderer] Created new session:', currentSessionId);
        }

        if (!currentSessionId) {
            showError('No chat selected. Please create a new chat first.');
            setProcessing(false);
            return;
        }

        console.log('[Renderer] Calling window.orionBridge.chat...');
        const result = await window.orionBridge.chat(message, currentSessionId);
        console.log('[Renderer] Chat result received:', result);

        if (result && result.response) {
            console.log('[Renderer] Response received, rendering...');
            finalizeStreamingMessage(result);

            // Auto-name session if it still says "New Chat"
            const sessions = await window.orionBridge.getSessions();
            const currentSession = sessions.find(s => s.id === currentSessionId);
            if (currentSession && currentSession.title === 'New Chat') {
                const words = message.trim().split(/\s+/);
                const newTitle = words.length <= 4 ? message.substring(0, 30) : words.slice(0, 4).join(' ') + '...';
                if (newTitle && newTitle.length > 0) {
                    await window.orionBridge.renameSession(currentSessionId, newTitle);
                    await loadSessions();
                }
            }

            // Speak the response if voice output is enabled
            speakText(result.response);
        } else {
            showError('No response received');
        }

        // Check for proactive suggestions after chat
        checkSuggestions().catch(() => {});

        // Check Ollama health periodically
        if (Math.random() < 0.1) { // ~10% of messages
            checkOllamaHealth().catch(() => {});
        }
    } catch (err) {
        console.error('Chat error:', err);
        logSystem(`ERROR: ${err.message}`);
        showError(err.message);
    } finally {
        setProcessing(false);
        currentStreamingMessage = null;
        const typingInd = document.getElementById('typing-indicator');
        if (typingInd) typingInd.classList.add('hidden');
    }
}



/**
 * Add message to chat — XSS-safe
 */
function addMessage(role, text) {
    const messages = document.getElementById('messages');
    const div = document.createElement('div');
    const isOrion = role === 'orion';
    div.className = `message ${isOrion ? 'orion-message' : 'user-message'} fade-in`;
    
    const header = document.createElement('div');
    header.className = 'message-header';
    
    const author = document.createElement('span');
    author.className = 'message-author';
    author.textContent = isOrion ? '◆ ORION' : '● ABHI';
    
    const meta = document.createElement('span');
    meta.className = 'message-meta';
    meta.textContent = new Date().toLocaleTimeString();
    
    header.appendChild(author);
    header.appendChild(meta);
    
    const content = document.createElement('div');
    content.className = 'message-content';
    content.textContent = text; // XSS-safe
    
    div.appendChild(header);
    div.appendChild(content);
    messages.appendChild(div);
    scrollToBottom();
}

function createStreamingMessage() {
    const messages = document.getElementById('messages');
    const div = document.createElement('div');
    div.className = 'message orion-message fade-in';
    
    const header = document.createElement('div');
    header.className = 'message-header';
    
    const author = document.createElement('span');
    author.className = 'message-author';
    author.textContent = '◆ ORION';
    
    const meta = document.createElement('span');
    meta.className = 'message-meta';
    meta.textContent = '⚡ PROCESSING';
    
    header.appendChild(author);
    header.appendChild(meta);
    
    const content = document.createElement('div');
    content.className = 'message-content';
    content.textContent = '...';
    
    div.appendChild(header);
    div.appendChild(content);
    messages.appendChild(div);
    scrollToBottom();
    return div;
}

function updateStreamingMessage(chunk) {
    if (!currentStreamingMessage) return;
    const content = currentStreamingMessage.querySelector('.message-content');
    if (content.textContent === '...') content.textContent = '';
    content.textContent += chunk;
    scrollToBottom();
}

function finalizeStreamingMessage(result) {
    if (!currentStreamingMessage) return;
    const meta = currentStreamingMessage.querySelector('.message-meta');
    const brain = window._lastBrainInfo;
    meta.textContent = brain ? `${brain.label} · ${new Date().toLocaleTimeString()}` : `ORION · ${new Date().toLocaleTimeString()}`;
}

function updateBrainIndicator(brain) {
    const indicator = document.getElementById('brain-indicator');
    if (indicator && brain) {
        indicator.textContent = brain.label || 'ORION';
    }
}

function showError(msg) {
    addMessage('orion', `[SYSTEM ERROR] ${msg}`);
}
window.showError = showError;

function setProcessing(state) {
    isProcessing = state;
    const input = document.getElementById('message-input');
    const btnSend = document.getElementById('btn-send');
    if (btnSend) btnSend.disabled = state;
    // Update status indicator
    setOrionState(state ? 'processing' : 'online');
}

function setOrionState(state) {
    const statusEl = document.getElementById('orion-status');
    if (!statusEl) return;

    // Remove all state classes
    statusEl.classList.remove('status-online', 'status-processing', 'status-listening', 'status-speaking');

    // Add new state
    const stateText = statusEl.querySelector('.status-text');

    switch(state) {
        case 'processing':
            statusEl.classList.add('status-processing');
            if (stateText) stateText.textContent = 'PROCESSING';
            break;
        case 'listening':
            statusEl.classList.add('status-listening');
            if (stateText) stateText.textContent = 'LISTENING';
            break;
        case 'speaking':
            statusEl.classList.add('status-speaking');
            if (stateText) stateText.textContent = 'SPEAKING';
            break;
        default: // online
            statusEl.classList.add('status-online');
            if (stateText) stateText.textContent = 'ONLINE';
    }
}

function scrollToBottom() {
    const container = document.getElementById('messages-container');
    if (container) container.scrollTop = container.scrollHeight;
}

function logSystem(msg) {
    const logs = document.getElementById('system-logs');
    if (logs) {
        const entry = document.createElement('div');
        entry.textContent = `[${new Date().toLocaleTimeString()}] ${msg}`;
        logs.appendChild(entry);
        logs.scrollTop = logs.scrollHeight;
    }
}

async function clearChat() {
    if (await orionConfirm('Clear Memory', 'Wipe ALL memory forever? This cannot be undone.')) {
        await window.orionBridge.clearHistory();
        location.reload();
    }
}

async function loadAgentStatus() {
    try {
        const status = await window.orionBridge.getAgents();
        if (status && status.agents) {
            logSystem(`Agents loaded: ${status.agents.map(a => a.name).join(', ')}`);
        }
    } catch (e) {
        console.error('Failed to load agent status:', e);
    }
}

async function loadOllamaStatus() {
    try {
        const info = await window.orionBridge.getOllama();
        const el = document.getElementById('ollama-status');
        if (el) {
            if (info.available) {
                el.textContent = `CONNECTED (${info.availableModels.length} models)`;
                el.className = 'status-online';
            } else {
                el.textContent = 'OFFLINE — Install Ollama';
                el.className = 'status-offline';
            }
        }
    } catch (e) {
        const el = document.getElementById('ollama-status');
        if (el) {
            el.textContent = 'OFFLINE';
            el.className = 'status-offline';
        }
    }
}

function forceUnlock() {
    setProcessing(false);
    currentStreamingMessage = null;
    logSystem('System force-unlocked.');
}

function toggleSettings() {
    const panel = document.getElementById('settings-panel');
    if (panel) panel.classList.toggle('hidden');
}

function setupInputHandlers() {
    const input = document.getElementById('message-input');
    if (input) {
        // Auto-resize textarea
        input.addEventListener('input', () => {
            input.style.height = 'auto';
            input.style.height = Math.min(input.scrollHeight, 120) + 'px';
        });

        // Enter to send, Shift+Enter for newline
        input.addEventListener('keydown', (e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                sendMessage();
            }
        });
    }
}

// ========================================
// MODE SWITCHING
// ========================================

async function switchMode(modeName) {
    try {
        const result = await window.orionBridge.setMode(modeName);
        if (result.success) {
            currentMode = modeName;
            updateModeUI(modeName);
            logSystem(`Mode switched to: ${modeName}`);

            const modeInfo = await window.orionBridge.getMode();
            const descEl = document.getElementById('mode-description');
            if (descEl && modeInfo.description) {
                descEl.textContent = modeInfo.description;
            }
        }
    } catch (err) {
        logSystem('Error switching mode: ' + err.message);
    }
}

function updateModeUI(modeName) {
    const buttons = document.querySelectorAll('.mode-btn');
    buttons.forEach(btn => {
        btn.classList.remove('active');
        if (btn.dataset.mode === modeName) {
            btn.classList.add('active');
        }
    });

    const indicator = document.getElementById('brain-indicator');
    if (indicator) {
        const modeLabels = {
            orion: '⚙️ ORION',
            friday: '🤖 FRIDAY',
            jarvis: '🧠 JARVIS',
            analyst: '📈 Analyst',
            coder: '💻 Coder',
            business: '💰 Business'
        };
        indicator.textContent = modeLabels[modeName] || modeName;
    }
}

function setupModeSelector() {
    const buttons = document.querySelectorAll('.mode-btn');
    buttons.forEach(btn => {
        btn.addEventListener('click', () => {
            const mode = btn.dataset.mode;
            if (mode) {
                switchMode(mode);
            }
        });
    });

    // Load current mode
    window.orionBridge.getMode().then(modeInfo => {
        if (modeInfo && modeInfo.mode) {
            currentMode = modeInfo.mode;
            updateModeUI(currentMode);
        }
    });
}

// ========================================
// GOAL MANAGEMENT
// ========================================

async function createGoal() {
    const title = await orionPrompt('New Goal', 'Enter goal title:');
    if (!title?.trim()) return;

    const deadline = await orionPrompt('New Goal', 'Enter deadline (YYYY-MM-DD HH:MM) or leave empty:');
    const goal = await window.orionBridge.createGoal(title.trim(), '', deadline || null);
    logSystem(`Goal created: ${title}`);
    loadGoals();
}

async function loadGoals() {
    try {
        const goals = await window.orionBridge.getGoals();
        const goalsList = document.getElementById('goals-list');
        if (!goalsList) return;

        if (!goals || goals.length === 0) {
            goalsList.innerHTML = '<div class="empty-state">No active goals</div>';
            return;
        }

        goalsList.innerHTML = '';
        goals.forEach(goal => {
            const div = document.createElement('div');
            div.className = 'goal-item';

            const info = document.createElement('div');
            info.innerHTML = `
                <div class="goal-item-title">${goal.title}</div>
                ${goal.deadline ? `<div class="goal-item-deadline">Due: ${new Date(goal.deadline).toLocaleString()}</div>` : ''}
            `;
            div.appendChild(info);

            const actions = document.createElement('div');
            actions.className = 'goal-item-actions';

            const completeBtn = document.createElement('button');
            completeBtn.className = 'goal-btn complete';
            completeBtn.textContent = '✓';
            completeBtn.title = 'Complete';
            completeBtn.onclick = () => completeGoalItem(goal.id);
            actions.appendChild(completeBtn);

            const deleteBtn = document.createElement('button');
            deleteBtn.className = 'goal-btn delete';
            deleteBtn.textContent = '✕';
            deleteBtn.title = 'Delete';
            deleteBtn.onclick = () => deleteGoalItem(goal.id);
            actions.appendChild(deleteBtn);

            div.appendChild(actions);
            goalsList.appendChild(div);
        });
    } catch (err) {
        console.error('Error loading goals:', err);
    }
}

async function completeGoalItem(goalId) {
    await window.orionBridge.completeGoal(goalId);
    logSystem('Goal completed!');
    loadGoals();
    loadStats();
}

async function deleteGoalItem(goalId) {
    await window.orionBridge.deleteGoal(goalId);
    logSystem('Goal deleted');
    loadGoals();
    loadStats();
}

function showGoalInput() {
    createGoal();
}

// ========================================
// SUGGESTIONS
// ========================================

async function loadSuggestions() {
    try {
        const suggestions = await window.orionBridge.getSuggestions();
        const suggestionsList = document.getElementById('suggestions-list');
        if (!suggestionsList) return;

        if (!suggestions || suggestions.length === 0) {
            suggestionsList.innerHTML = '<div class="empty-state">No pending suggestions</div>';
            return;
        }

        suggestionsList.innerHTML = '';
        suggestions.forEach(s => {
            const div = document.createElement('div');
            div.className = `suggestion-item ${s.type}`;
            div.textContent = s.message;
            suggestionsList.appendChild(div);
        });
    } catch (err) {
        console.error('Error loading suggestions:', err);
    }
}

// ========================================
// STATS
// ========================================

async function loadStats() {
    try {
        const stats = await window.orionBridge.getStats();
        const sessionsEl = document.getElementById('stat-sessions');
        const messagesEl = document.getElementById('stat-messages');
        const goalsEl = document.getElementById('stat-goals');
        const argumentsEl = document.getElementById('stat-arguments');

        if (sessionsEl && stats.totalSessions !== undefined) sessionsEl.textContent = stats.totalSessions;
        if (messagesEl && stats.totalMessages !== undefined) messagesEl.textContent = stats.totalMessages;
        if (goalsEl && stats.active !== undefined) goalsEl.textContent = stats.active;
        if (argumentsEl) {
            const argStats = await window.orionBridge.getArgumentStats();
            argumentsEl.textContent = argStats.totalArguments || 0;
        }
    } catch (err) {
        console.error('Error loading stats:', err);
    }
}

// ========================================
// SESSION MANAGEMENT
// ========================================

async function loadSessions() {
    try {
        const sessions = await window.orionBridge.getSessions();
        const chatList = document.getElementById('chat-list');
        if (!chatList) return;

        // Auto-select first session if none selected
        const currentId = await window.orionBridge.getCurrentSessionId();
        if (!currentId && sessions.length > 0) {
            await window.orionBridge.switchSession(sessions[0].id);
            await loadHistory();
        }

        chatList.innerHTML = '';

        sessions.forEach(session => {
            const div = document.createElement('div');
            div.className = 'nav-item';
            div.dataset.sessionId = session.id;
            div.dataset.title = session.title || 'New Chat';
            
            // Session title
            const titleSpan = document.createElement('span');
            titleSpan.className = 'session-title';
            titleSpan.textContent = session.title || 'New Chat';
            div.appendChild(titleSpan);
            
            // Rename button
            const renameBtn = document.createElement('button');
            renameBtn.className = 'session-btn';
            renameBtn.textContent = '✏️';
            renameBtn.title = 'Rename chat';
            renameBtn.onclick = (e) => {
                e.stopPropagation();
                orionPrompt('Rename Chat', 'Enter new chat name:', session.title || 'New Chat').then(newTitle => {
                    if (newTitle?.trim()) {
                        window.orionBridge.renameSession(session.id, newTitle.trim()).then(() => loadSessions());
                    }
                });
            };
            div.appendChild(renameBtn);
            
            // Delete button
            const deleteBtn = document.createElement('button');
            deleteBtn.className = 'session-btn';
            deleteBtn.textContent = '🗑️';
            deleteBtn.title = 'Delete chat';
            deleteBtn.onclick = async (e) => {
                e.stopPropagation();
                const confirmed = await orionConfirm('Delete Chat', 'Delete this chat permanently? This cannot be undone.');
                if (!confirmed) return;

                const currentId = await window.orionBridge.getCurrentSessionId();
                const wasCurrent = currentId === session.id;

                await window.orionBridge.deleteSession(session.id);

                if (wasCurrent) {
                    const sessions = await window.orionBridge.getSessions();
                    if (sessions.length > 0) {
                        await window.orionBridge.switchSession(sessions[0].id);
                        document.getElementById('messages').innerHTML = '';
                        await loadHistory();
                    }
                }
                await loadSessions();
            };
            div.appendChild(deleteBtn);
            
            // Click handler for session switching
            div.onclick = () => switchSession(session.id, session.title);
            chatList.appendChild(div);
        });
    } catch (e) {
        logSystem('Error loading sessions: ' + e.message);
    }
}

async function switchSession(sessionId, title) {
    await window.orionBridge.switchSession(sessionId);
    document.getElementById('messages').innerHTML = '';
    await loadHistory();
    showView('chat-area');

    // Update active state in sidebar (compare by data attribute, not textContent)
    const items = document.querySelectorAll('#chat-list .nav-item');
    items.forEach(item => {
        item.classList.remove('active');
        if (item.getAttribute('data-title') === title) {
            item.classList.add('active');
        }
    });
}

function initVoiceSystem() {
    // Check browser support
    const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
    if (!SpeechRecognition) {
        logSystem('Voice input not supported');
        return;
    }

    recognition = new SpeechRecognition();
    recognition.continuous = false;
    recognition.interimResults = false;
    recognition.lang = 'en-US';

    recognition.onresult = (event) => {
        const transcript = event.results[0][0].transcript.trim();
        if (transcript) {
            const input = document.getElementById('message-input');
            if (input) {
                input.value = transcript;
                input.style.height = 'auto';
                input.style.height = Math.min(input.scrollHeight, 120) + 'px';
                sendMessage();
            }
        }
        isListening = false;
        updateVoiceUI();
    };

    recognition.onerror = (event) => {
        console.error('[Voice] Error:', event.error);
        isListening = false;
        updateVoiceUI();
    };

    recognition.onend = () => {
        isListening = false;
        updateVoiceUI();
    };
}

function toggleVoiceInput() {
    if (!recognition) {
        logSystem('Voice input not available');
        return;
    }

    if (isListening) {
        recognition.stop();
        isListening = false;
        setOrionState('online');
    } else {
        try {
            recognition.start();
            isListening = true;
            setOrionState('listening');
            logSystem('Listening...');
        } catch (err) {
            console.error('[Voice] Start error:', err);
            isListening = false;
            setOrionState('online');
        }
    }
    updateVoiceUI();
}

function toggleVoiceOutput() {
    const btn = document.getElementById('btn-voice-output');
    if (!btn) return;

    const enabled = btn.dataset.enabled !== 'true';
    btn.dataset.enabled = enabled;
    btn.style.background = enabled ? 'rgba(0, 212, 255, 0.3)' : 'transparent';
    logSystem(enabled ? 'Voice output enabled' : 'Voice output disabled');

    if (!enabled && isSpeaking) {
        stopSpeaking();
    }
}

function stopSpeaking() {
    if (speechSynth && speechSynth.speaking) {
        speechSynth.cancel();
        isSpeaking = false;
        setOrionState('online');
        updateVoiceUI();
    }
}

function speakText(text) {
    const btn = document.getElementById('btn-voice-output');
    if (!btn || btn.dataset.enabled !== 'true') return;
    if (!speechSynth || isSpeaking) return;

    const cleanText = text
        .replace(/```[\s\S]*?```/g, 'code block')
        .replace(/`([^`]+)`/g, '$1')
        .replace(/\*\*([^*]+)\*\*/g, '$1')
        .replace(/\*([^*]+)\*/g, '$1')
        .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1')
        .replace(/\n+/g, '. ');

    currentUtterance = new SpeechSynthesisUtterance(cleanText);
    currentUtterance.rate = 1.0;
    currentUtterance.pitch = 1.0;
    currentUtterance.volume = 1.0;

    currentUtterance.onstart = () => {
        isSpeaking = true;
        setOrionState('speaking');
        updateVoiceUI();
    };

    currentUtterance.onend = () => {
        isSpeaking = false;
        setOrionState('online');
        updateVoiceUI();
    };

    currentUtterance.onerror = () => {
        isSpeaking = false;
        updateVoiceUI();
    };

    speechSynth.speak(currentUtterance);
}

function updateVoiceUI() {
    const micBtn = document.getElementById('btn-mic');
    const speakerBtn = document.getElementById('btn-voice-output');

    if (micBtn) {
        micBtn.style.background = isListening ? 'rgba(0, 212, 255, 0.3)' : 'transparent';
        micBtn.textContent = isListening ? '⏹' : '🎤';
    }

    if (speakerBtn) {
        speakerBtn.style.background = isSpeaking ? 'rgba(0, 212, 255, 0.3)' : 'transparent';
        speakerBtn.textContent = isSpeaking ? '⏹' : '🔊';
    }
}

// ========================================
// RESIZE HANDLER (Performance)
// ========================================

function setupResizeHandler() {
    let resizeTimeout;
    let isResizing = false;

    window.addEventListener('resize', () => {
        if (!isResizing) {
            document.body.classList.add('resizing');
            isResizing = true;
        }

        // Debounce resize end
        clearTimeout(resizeTimeout);
        resizeTimeout = setTimeout(() => {
            document.body.classList.remove('resizing');
            isResizing = false;
        }, 150);
    });
}

// ========================================
// PROJECTS
// ========================================

async function loadProjects() {
    try {
        const projects = await window.orionBridge.getProjects();
        const projectsList = document.getElementById('projects-list');
        if (!projectsList) return;

        projectsList.innerHTML = '';

        projects.forEach(project => {
            const div = document.createElement('div');
            div.className = 'project-item';
            div.innerHTML = `
                <div class="project-header" onclick="toggleProject(${project.id})">
                    <span class="project-expand">▶</span>
                    <span class="project-name">${project.name}</span>
                </div>
                <div class="project-chats" id="project-chats-${project.id}">
                    <button class="text-btn" onclick="createChatInProject(${project.id})">+ New Chat</button>
                </div>
            `;
            projectsList.appendChild(div);
        });
    } catch (e) {
        console.error('Error loading projects:', e);
    }
}

function toggleProject(projectId) {
    const item = document.querySelector(`.project-item:has(#project-chats-${projectId})`);
    if (item) {
        item.classList.toggle('expanded');
    }
}

async function showCreateProject() {
    const name = await orionPrompt('New Project', 'Project name:');
    if (!name) return;

    const description = await orionPrompt('New Project', 'Project description:') || '';
    await window.orionBridge.createProject(name, description);
    await loadProjects();
    logSystem(`Project "${name}" created`);
}

async function createChatInProject(projectId) {
    const sessionId = await window.orionBridge.createSession('New Chat', projectId);
    await window.orionBridge.switchSession(sessionId);
    document.getElementById('messages').innerHTML = '';
    showView('chat-area');
    addMessage('orion', "New session started in this project. What's the goal?");
    await loadSessions();
}

// ========================================
// FOLDERS
// ========================================

async function loadFolders() {
    try {
        const folders = await window.orionBridge.getFolders();
        const foldersList = document.getElementById('folders-list');
        if (!foldersList) return;

        foldersList.innerHTML = '';

        if (folders.length === 0) {
            foldersList.innerHTML = '<div class="empty-state" style="padding: 8px;">No folders added</div>';
            return;
        }

        folders.forEach(folder => {
            const div = document.createElement('div');
            div.className = `folder-item ${folder.is_active ? 'active' : ''}`;
            div.innerHTML = `
                <span class="folder-icon">📁</span>
                <span class="folder-name">${folder.name}</span>
                <div class="folder-actions">
                    <button class="icon-btn" onclick="setActiveFolder(${folder.id})" title="Set Active">✓</button>
                    <button class="icon-btn" onclick="sendFolderToScrap(${folder.id})" title="Send to Scrap">🗑️</button>
                </div>
            `;
            foldersList.appendChild(div);
        });
    } catch (e) {
        console.error('Error loading folders:', e);
    }
}

async function addFolder() {
    const path = await window.orionBridge.selectFolder();
    if (!path) return;

    const name = path.split(/[/\\]/).pop() || 'Folder';
    await window.orionBridge.addFolder(path, name);
    await loadFolders();
    logSystem(`Folder "${name}" added`);
}

async function setActiveFolder(folderId) {
    await window.orionBridge.setActiveFolder(folderId);
    await loadFolders();
    const folder = await window.orionBridge.getActiveFolder();
    logSystem(`Active workspace: ${folder.name}`);
}

async function sendFolderToScrap(folderId) {
    if (!await orionConfirm('Scrap Folder', 'Send this folder to scrap? (Can be recovered later)')) return;
    await window.orionBridge.sendFolderToScrap(folderId);
    await loadFolders();
    logSystem('Folder sent to scrap');
}

async function showForgottenFolders() {
    const folders = await window.orionBridge.getForgottenFolders();
    if (folders.length === 0) {
        await orionAlert('Forgotten Folders', 'No forgotten folders');
        return;
    }
    const list = folders.map(f => `${f.name} (${f.path})`).join('\n');
    await orionAlert('Forgotten Folders', list);
}

// ========================================
// SEARCH
// ========================================

function filterChats(query) {
    const items = document.querySelectorAll('#chat-list .nav-item');
    items.forEach(item => {
        const title = item.querySelector('.session-title')?.textContent || '';
        item.style.display = title.toLowerCase().includes(query.toLowerCase()) ? '' : 'none';
    });
}

// ========================================
// TASK TRACKER
// ========================================

function extractTaskIntent(message) {
    const patterns = [
        { regex: /^(?:make|build|create|write|develop|design|implement)\b/i, priority: 'high' },
        { regex: /^(?:fix|debug|resolve|repair)\b/i, priority: 'high' },
        { regex: /^(?:research|find|look into|investigate)\b/i, priority: 'medium' },
        { regex: /^(?:schedule|plan|organize|setup)\b/i, priority: 'medium' },
    ];
    for (const p of patterns) {
        if (p.regex.test(message.trim())) {
            return message.substring(0, 100);
        }
    }
    return message.length > 20 ? message.substring(0, 100) : null;
}

async function checkSuggestions() {
    try {
        const suggestions = await window.orionBridge.getSuggestions();
        if (suggestions && suggestions.length > 0) {
            const highPriority = suggestions.filter(s => s.priority === 'high');
            if (highPriority.length > 0) {
                logSystem(`⚠️ ${highPriority[0].message}`);
            }
        }
    } catch (e) {
        // Silently fail
    }
}

async function checkOllamaHealth() {
    try {
        const result = await window.orionBridge.checkOllamaHealth();
        const el = document.getElementById('ollama-status');
        if (el && result) {
            await loadOllamaStatus();
        }
    } catch (e) {
        // Silently fail
    }
}
