/**
 * ORION — Electron Main Process v2
 * The heart of the desktop app. Manages window, IPC, and core modules.
 * ORION 2.0: Memory, Context, Proactive Features
 */

const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const fs = require('fs');

// Setup logging
const dataDir = path.join(__dirname, 'DATA');
if (!fs.existsSync(dataDir)) fs.mkdirSync(dataDir);
const logPath = path.join(dataDir, 'debug.log');

function log(msg) {
    const entry = `[${new Date().toISOString()}] ${msg}\n`;
    fs.appendFileSync(logPath, entry);
    console.log(entry.trim());
}

log('ORION 2.0 System Booting...');

// Core imports
const PersonalityEngine = require('./CORE/personality-engine');
const AIBrain = require('./CORE/ai-brain');
const MemoryEngine = require('./CORE/memory-engine');
const AgentRegistry = require('./CORE/agent-registry');
const CoderAgent = require('./CORE/coder-agent');
const BusinessAgent = require('./CORE/business-agent');
const SchedulerAgent = require('./CORE/scheduler-agent');
const OllamaBrain = require('./CORE/ollama');

// NEW: ORION 2.0 modules
const UserProfile = require('./CORE/user-profile');
const ContextManager = require('./CORE/context-manager');
const ArgumentEngine = require('./CORE/argument-engine');
const SuggestionEngine = require('./CORE/suggestion-engine');
const GoalTracker = require('./CORE/goal-tracker');

// Load settings
const settingsPath = path.join(__dirname, 'CONFIG', 'settings.json');
let settings;
try {
    settings = JSON.parse(fs.readFileSync(settingsPath, 'utf-8'));
} catch (err) {
    console.error('[ORION] Failed to load settings.json:', err.message);
    console.error('[ORION] Please ensure CONFIG/settings.json exists and is valid JSON.');
    process.exit(1);
}

// Initialize core modules
const ollama = new OllamaBrain(settings.ollama);
const personality = new PersonalityEngine();
const agentRegistry = new AgentRegistry(ollama);

// Register sub-agents
agentRegistry.register(new CoderAgent());
agentRegistry.register(new BusinessAgent());
agentRegistry.register(new SchedulerAgent());

const brain = new AIBrain(settings, personality, agentRegistry, ollama);

// Memory first (needed by other modules)
const memory = new MemoryEngine();

// NEW: Initialize ORION 2.0 modules after memory
let userProfile = null;
let contextManager = null;
let argumentEngine = null;
let suggestionEngine = null;
let goalTracker = null;

let mainWindow = null;

function createWindow() {
    mainWindow = new BrowserWindow({
        width: 1200,
        height: 800,
        minWidth: 800,
        minHeight: 600,
        frame: false,
        transparent: false,
        backgroundColor: '#0a0a0f',
        icon: path.join(__dirname, 'UI', 'assets', process.platform === 'win32' ? 'icon.ico' : 'icon.png'),
        webPreferences: {
            preload: path.join(__dirname, 'preload.js'),
            contextIsolation: true,
            nodeIntegration: false,
            sandbox: false
        }
    });

    mainWindow.loadFile(path.join(__dirname, 'UI', 'index2.html'));

    // Open DevTools in dev mode
    if (process.argv.includes('--dev')) {
        mainWindow.webContents.openDevTools();
    }

    mainWindow.on('closed', () => {
        mainWindow = null;
    });
}

// ============================================================
// IPC HANDLERS — Bridge between UI and Core
// ============================================================

// Send a message to ORION
ipcMain.handle('orion:chat', async (event, message, sessionId) => {
    console.log('[Main] orion:chat called with sessionId:', sessionId);
    log(`User Message: ${message}`);
    try {
        console.log('[Main] Brain object exists:', !!brain);
        console.log('[Main] Brain type:', brain ? brain.constructor.name : 'undefined');

        // Auto-detect mode from message
        const detectedMode = personality.detectMode(message);
        personality.setMode(detectedMode);

        // Update context manager with current session
        if (contextManager) {
            contextManager.setSession(sessionId);
            contextManager.addToContext('user', message);
        }

        // Check if we should argue
        let finalMessage = message;
        if (argumentEngine) {
            const shouldArgue = argumentEngine.shouldArgue(message);
            if (shouldArgue.shouldArgue) {
                const pushback = argumentEngine.generatePushback(shouldArgue);
                log(`[ArgumentEngine] Pushing back: ${pushback}`);
            }
        }

        const history = memory.getHistory(10, sessionId);
        console.log('[Main] History length:', history.length);
        const formattedHistory = history.map(h => ({
            role: h.role,
            content: h.content
        }));

        // Save user message with detected mode
        memory.saveMessage('user', message, detectedMode, 'user', sessionId);

        console.log('[Main] Calling brain.asyncChat...');

        let result;
        try {
            let chunkCount = 0;
            result = await brain.asyncChat(finalMessage, formattedHistory, (chunk) => {
                chunkCount++;
                event.sender.send('orion:chunk', chunk);
            });
            console.log('[Main] Brain response received:', result ? 'yes' : 'no');
            console.log('[Main] Response has response field:', result && !!result.response);
            log(`Response Complete. Mode: ${detectedMode}, Brain: ${result.brain.label}, Chunks: ${chunkCount}`);
        } catch (brainError) {
            console.error('[Main] Brain error:', brainError);
            console.error('[Main] Brain error stack:', brainError.stack);
            throw brainError;
        }

        // Save assistant response
        memory.saveMessage('assistant', result.response, personality.getModeDisplay(), result.brain.label, sessionId);

        // Update context with response
        if (contextManager) {
            contextManager.addToContext('assistant', result.response);
        }

        // Learn from interaction
        if (userProfile) {
            userProfile.learnFromInteraction(message, result.response);
        }

        return {
            response: result.response,
            mode: personality.getModeDisplay(),
            modeColor: personality.getModeColor(),
            brain: brain.getLastBrainInfo(),
            agent: result.agent || null,
            argument: argumentEngine ? argumentEngine.getStats() : null,
            timestamp: new Date().toISOString()
        };
    } catch (error) {
        console.error('[Main] FULL ERROR:', error);
        console.error('[Main] ERROR STACK:', error.stack);
        log(`CRITICAL CHAT ERROR: ${error.message}`);
        if (error.stack) log(error.stack);
        return {
            response: `[ORION SYSTEM ERROR] ${error.message}`,
            mode: personality.getModeDisplay(),
            brain: { brain: 'error', label: 'Error' },
            timestamp: new Date().toISOString()
        };
    }
});

// ============================================================
// SESSION HANDLERS
// ============================================================

ipcMain.handle('orion:getHistory', async (event, sessionId) => {
    try {
        return memory.getHistory(100, sessionId);
    } catch (err) {
        console.error('Error getting history:', err);
        return [];
    }
});

ipcMain.handle('orion:getSessions', async () => {
    try {
        return memory.getSessions();
    } catch (err) {
        console.error('Error getting sessions:', err);
        return [];
    }
});

ipcMain.handle('orion:getCurrentSessionId', async () => {
    try {
        console.log('[Session] Getting current session...');

        // Wait for memory to be ready if not ready
        let retries = 0;
        while (!memory.ready && retries < 10) {
            console.log('[Session] Waiting for memory...', retries);
            await new Promise(r => setTimeout(r, 100));
            retries++;
        }

        if (!memory.ready) {
            console.error('[Session] Memory NOT ready after retries!');
            // Try to create session anyway
            return memory.createSession('New Chat', 1) || 1;
        }

        console.log('[Session] Memory ready');

        const sessions = memory.getSessions();
        console.log('[Session] Found sessions:', sessions.length);

        if (sessions.length > 0) {
            console.log('[Session] Returning existing session:', sessions[0].id);
            return sessions[0].id;
        }

        // No sessions exist - create a new default session
        console.log('[Session] No sessions found, creating new...');
        const newSessionId = memory.createSession('New Chat', 1);
        console.log('[Session] Created new session:', newSessionId);
        return newSessionId || 1;
    } catch (err) {
        console.error('[Session] Error getting current session:', err);
        // Fallback - return 1
        return 1;
    }
});

ipcMain.handle('orion:createSession', async (event, title, projectId = 1) => {
    try {
        console.log('[Session] Creating session:', title, 'projectId:', projectId);
        const sessionId = memory.createSession(title || 'New Chat', projectId);
        console.log('[Session] Created session ID:', sessionId);
        if (contextManager) {
            contextManager.setSession(sessionId);
        }
        return sessionId;
    } catch (err) {
        console.error('[Session] Error creating session:', err);
        return null;
    }
});

ipcMain.handle('orion:switchSession', async (event, sessionId) => {
    try {
        if (contextManager) {
            contextManager.setSession(sessionId);
        }
        return memory.switchSession(sessionId);
    } catch (err) {
        console.error('Error switching session:', err);
        return { error: err.message };
    }
});

ipcMain.handle('orion:renameSession', async (event, sessionId, newTitle) => {
    try {
        return await memory.renameSession(sessionId, newTitle);
    } catch (err) {
        console.error('Error renaming session:', err);
        return { error: err.message };
    }
});

ipcMain.handle('orion:deleteSession', async (event, sessionId) => {
    try {
        return await memory.deleteSession(sessionId);
    } catch (err) {
        console.error('Error deleting session:', err);
        return { error: err.message };
    }
});

// ============================================================
// MODE HANDLERS (NEW)
// ============================================================

ipcMain.handle('orion:getMode', async () => {
    try {
        return {
            mode: personality.getCurrentMode(),
            display: personality.getModeDisplay(),
            color: personality.getModeColor(),
            description: personality.getModeDescription(),
            allModes: personality.getAllModes()
        };
    } catch (err) {
        console.error('Error getting mode:', err);
        return { mode: 'orion', display: 'ORION', color: '#00f0ff' };
    }
});

ipcMain.handle('orion:setMode', async (event, modeName) => {
    try {
        return personality.setMode(modeName);
    } catch (err) {
        console.error('Error setting mode:', err);
        return { success: false, error: err.message };
    }
});

// ============================================================
// USER PROFILE HANDLERS (NEW)
// ============================================================

ipcMain.handle('profile:get', async (event, key) => {
    try {
        return userProfile ? userProfile.get(key) : null;
    } catch (err) {
        console.error('Error getting profile:', err);
        return null;
    }
});

ipcMain.handle('profile:set', async (event, key, value, category) => {
    try {
        if (userProfile) {
            userProfile.set(key, value, category || 'general');
            return { success: true };
        }
        return { success: false };
    } catch (err) {
        console.error('Error setting profile:', err);
        return { success: false, error: err.message };
    }
});

ipcMain.handle('profile:getAll', async () => {
    try {
        return memory.getAllProfile();
    } catch (err) {
        console.error('Error getting all profile:', err);
        return {};
    }
});

ipcMain.handle('profile:getPreferences', async () => {
    try {
        return userProfile ? userProfile.getPreferences() : {};
    } catch (err) {
        console.error('Error getting preferences:', err);
        return {};
    }
});

ipcMain.handle('profile:updatePreferences', async (event, updates) => {
    try {
        if (userProfile) {
            userProfile.updatePreferences(updates);
            return { success: true };
        }
        return { success: false };
    } catch (err) {
        console.error('Error updating preferences:', err);
        return { success: false, error: err.message };
    }
});

// ============================================================
// GOAL HANDLERS (NEW)
// ============================================================

ipcMain.handle('goals:create', async (event, title, description, deadline) => {
    try {
        if (goalTracker) {
            return goalTracker.createGoal(title, description, deadline);
        }
        return null;
    } catch (err) {
        console.error('Error creating goal:', err);
        return null;
    }
});

ipcMain.handle('goals:list', async () => {
    try {
        return goalTracker ? goalTracker.getActiveGoals() : [];
    } catch (err) {
        console.error('Error listing goals:', err);
        return [];
    }
});

ipcMain.handle('goals:getAll', async () => {
    try {
        return goalTracker ? goalTracker.getAllGoals() : [];
    } catch (err) {
        console.error('Error getting all goals:', err);
        return [];
    }
});

ipcMain.handle('goals:complete', async (event, goalId) => {
    try {
        return goalTracker ? goalTracker.completeGoal(goalId) : { success: false };
    } catch (err) {
        console.error('Error completing goal:', err);
        return { success: false, error: err.message };
    }
});

ipcMain.handle('goals:fail', async (event, goalId, reason) => {
    try {
        return goalTracker ? goalTracker.failGoal(goalId, reason) : { success: false };
    } catch (err) {
        console.error('Error failing goal:', err);
        return { success: false, error: err.message };
    }
});

ipcMain.handle('goals:delete', async (event, goalId) => {
    try {
        return goalTracker ? goalTracker.deleteGoal(goalId) : { success: false };
    } catch (err) {
        console.error('Error deleting goal:', err);
        return { success: false, error: err.message };
    }
});

ipcMain.handle('goals:getStats', async () => {
    try {
        return goalTracker ? goalTracker.getStats() : {};
    } catch (err) {
        console.error('Error getting goal stats:', err);
        return {};
    }
});

ipcMain.handle('goals:getAgenda', async () => {
    try {
        return goalTracker ? goalTracker.getTodaysAgenda() : {};
    } catch (err) {
        console.error('Error getting agenda:', err);
        return {};
    }
});

ipcMain.handle('goals:getOverdue', async () => {
    try {
        return goalTracker ? goalTracker.getOverdueGoals() : [];
    } catch (err) {
        console.error('Error getting overdue goals:', err);
        return [];
    }
});

// ============================================================
// SUGGESTION HANDLERS (NEW)
// ============================================================

ipcMain.handle('suggestions:get', async () => {
    try {
        return suggestionEngine ? suggestionEngine.getSuggestions() : [];
    } catch (err) {
        console.error('Error getting suggestions:', err);
        return [];
    }
});

ipcMain.handle('suggestions:getProactive', async () => {
    try {
        return suggestionEngine ? suggestionEngine.getProactiveSuggestions() : [];
    } catch (err) {
        console.error('Error getting proactive suggestions:', err);
        return [];
    }
});

ipcMain.handle('suggestions:dismiss', async (event, followUpId) => {
    try {
        if (contextManager) {
            contextManager.dismissFollowUp(followUpId);
            return { success: true };
        }
        return { success: false };
    } catch (err) {
        console.error('Error dismissing suggestion:', err);
        return { success: false, error: err.message };
    }
});

ipcMain.handle('suggestions:createFollowUp', async (event, context, minutes) => {
    try {
        if (suggestionEngine) {
            return suggestionEngine.generateFollowUp(context, minutes || 60);
        }
        return null;
    } catch (err) {
        console.error('Error creating follow-up:', err);
        return null;
    }
});

// ============================================================
// ARGUMENT ENGINE HANDLERS (NEW)
// ============================================================

ipcMain.handle('argument:getStats', async () => {
    try {
        return argumentEngine ? argumentEngine.getStats() : {};
    } catch (err) {
        console.error('Error getting argument stats:', err);
        return {};
    }
});

ipcMain.handle('argument:logResponse', async (event, response) => {
    try {
        if (argumentEngine) {
            argumentEngine.logUserResponse(response);
        }
        return { success: true };
    } catch (err) {
        console.error('Error logging response:', err);
        return { success: false, error: err.message };
    }
});

ipcMain.handle('argument:critiquePlan', async (event, plan) => {
    try {
        return argumentEngine ? argumentEngine.critiquePlan(plan) : null;
    } catch (err) {
        console.error('Error critiquing plan:', err);
        return null;
    }
});

// ============================================================
// CONTEXT HANDLERS (NEW)
// ============================================================

ipcMain.handle('context:setActiveTask', async (event, task, minutes) => {
    try {
        if (contextManager) {
            contextManager.setActiveTask(task, minutes || 120);
            return { success: true };
        }
        return { success: false };
    } catch (err) {
        console.error('Error setting active task:', err);
        return { success: false, error: err.message };
    }
});

ipcMain.handle('context:getActiveTask', async () => {
    try {
        return contextManager ? contextManager.getActiveTask() : null;
    } catch (err) {
        console.error('Error getting active task:', err);
        return null;
    }
});

ipcMain.handle('context:clearActiveTask', async () => {
    try {
        if (contextManager) {
            contextManager.clearActiveTask();
            return { success: true };
        }
        return { success: false };
    } catch (err) {
        console.error('Error clearing active task:', err);
        return { success: false, error: err.message };
    }
});

// ============================================================
// LEGACY HANDLERS
// ============================================================

ipcMain.handle('orion:getStats', async () => {
    try {
        const memoryStats = memory.getStats();
        const goalStats = goalTracker ? goalTracker.getStats() : {};
        const argStats = argumentEngine ? argumentEngine.getStats() : {};
        const sessions = memory.getSessions();
        return { ...memoryStats, ...goalStats, ...argStats, totalSessions: sessions.length };
    } catch (err) {
        console.error('Error getting stats:', err);
        return {};
    }
});

ipcMain.handle('orion:clearHistory', async () => {
    try {
        memory.clearHistory();
        return { success: true };
    } catch (err) {
        console.error('Error clearing history:', err);
        return { success: false, error: err.message };
    }
});

ipcMain.handle('orion:getAgents', async () => {
    try {
        return agentRegistry.getStatus();
    } catch (err) {
        console.error('Error getting agents:', err);
        return { agents: [] };
    }
});

ipcMain.handle('orion:getOllama', async () => {
    try {
        return ollama.getInfo();
    } catch (err) {
        console.error('Error getting ollama info:', err);
        return { available: false, error: err.message };
    }
});

// ============================================================
// CLIENT HANDLERS
// ============================================================

ipcMain.handle('client:create', async (event, name, email, phone, notes) => {
    try {
        const id = memory.createClient(name, email || '', phone || '', notes || '');
        return { id, name, email, phone, notes };
    } catch (err) {
        console.error('Error creating client:', err);
        return { error: err.message };
    }
});

ipcMain.handle('client:list', async () => {
    try {
        return memory.getClients();
    } catch (err) {
        console.error('Error listing clients:', err);
        return [];
    }
});

ipcMain.handle('client:get', async (event, clientId) => {
    try {
        return memory.getClient(clientId);
    } catch (err) {
        console.error('Error getting client:', err);
        return null;
    }
});

ipcMain.handle('client:update', async (event, clientId, data) => {
    try {
        memory.updateClient(clientId, data);
        return { success: true };
    } catch (err) {
        console.error('Error updating client:', err);
        return { error: err.message };
    }
});

ipcMain.handle('client:delete', async (event, clientId) => {
    try {
        memory.deleteClient(clientId);
        return { success: true };
    } catch (err) {
        console.error('Error deleting client:', err);
        return { error: err.message };
    }
});

// ============================================================
// LEAD HANDLERS
// ============================================================

ipcMain.handle('lead:create', async (event, name, email, phone, company, stage, value, notes) => {
    try {
        const id = memory.createLead(name, email || '', phone || '', company || '', stage || 'cold', value || 0, notes || '');
        return { id, name, email, phone, company, stage, value, notes };
    } catch (err) {
        console.error('Error creating lead:', err);
        return { error: err.message };
    }
});

ipcMain.handle('lead:list', async () => {
    try {
        return memory.getLeads();
    } catch (err) {
        console.error('Error listing leads:', err);
        return [];
    }
});

ipcMain.handle('lead:get', async (event, leadId) => {
    try {
        return memory.getLead(leadId);
    } catch (err) {
        console.error('Error getting lead:', err);
        return null;
    }
});

ipcMain.handle('lead:update', async (event, leadId, data) => {
    try {
        memory.updateLead(leadId, data);
        return { success: true };
    } catch (err) {
        console.error('Error updating lead:', err);
        return { error: err.message };
    }
});

ipcMain.handle('lead:delete', async (event, leadId) => {
    try {
        memory.deleteLead(leadId);
        return { success: true };
    } catch (err) {
        console.error('Error deleting lead:', err);
        return { error: err.message };
    }
});

// ============================================================
// PROJECT HANDLERS
// ============================================================

ipcMain.handle('project:create', async (event, name, description) => {
    try {
        const projectId = memory.createProject(name, description);
        return { id: projectId, name, description };
    } catch (err) {
        console.error('Error creating project:', err);
        return { error: err.message };
    }
});

ipcMain.handle('project:list', async () => {
    try {
        return memory.getProjects();
    } catch (err) {
        console.error('Error listing projects:', err);
        return [];
    }
});

ipcMain.handle('project:get', async (event, projectId) => {
    try {
        return memory.getProject(projectId);
    } catch (err) {
        console.error('Error getting project:', err);
        return null;
    }
});

ipcMain.handle('project:update', async (event, projectId, name, description) => {
    try {
        memory.updateProject(projectId, name, description);
        return { success: true };
    } catch (err) {
        console.error('Error updating project:', err);
        return { error: err.message };
    }
});

ipcMain.handle('project:delete', async (event, projectId) => {
    try {
        memory.deleteProject(projectId);
        return { success: true };
    } catch (err) {
        console.error('Error deleting project:', err);
        return { error: err.message };
    }
});

// ============================================================
// FOLDER HANDLERS
// ============================================================

ipcMain.handle('folder:add', async (event, path, name) => {
    try {
        const folderId = memory.addFolder(path, name);
        return { id: folderId, path, name };
    } catch (err) {
        console.error('Error adding folder:', err);
        return { error: err.message };
    }
});

ipcMain.handle('folder:list', async () => {
    try {
        return memory.getFolders();
    } catch (err) {
        console.error('Error listing folders:', err);
        return [];
    }
});

ipcMain.handle('folder:setActive', async (event, folderId) => {
    try {
        memory.setActiveFolder(folderId);
        return { success: true };
    } catch (err) {
        console.error('Error setting active folder:', err);
        return { error: err.message };
    }
});

ipcMain.handle('folder:getActive', async () => {
    try {
        return memory.getActiveFolder();
    } catch (err) {
        console.error('Error getting active folder:', err);
        return null;
    }
});

ipcMain.handle('folder:sendToScrap', async (event, folderId) => {
    try {
        memory.sendFolderToScrap(folderId);
        return { success: true };
    } catch (err) {
        console.error('Error sending folder to scrap:', err);
        return { error: err.message };
    }
});

ipcMain.handle('folder:recover', async (event, folderId) => {
    try {
        memory.recoverFolder(folderId);
        return { success: true };
    } catch (err) {
        console.error('Error recovering folder:', err);
        return { error: err.message };
    }
});

ipcMain.handle('folder:delete', async (event, folderId) => {
    try {
        memory.deleteFolder(folderId);
        return { success: true };
    } catch (err) {
        console.error('Error deleting folder:', err);
        return { error: err.message };
    }
});

ipcMain.handle('folder:getForgotten', async () => {
    try {
        return memory.getForgottenFolders();
    } catch (err) {
        console.error('Error getting forgotten folders:', err);
        return [];
    }
});

ipcMain.handle('folder:selectDialog', async () => {
    try {
        const { dialog } = require('electron');
        const result = await dialog.showOpenDialog(mainWindow, {
            properties: ['openDirectory']
        });
        if (result.canceled || result.filePaths.length === 0) {
            return null;
        }
        return result.filePaths[0];
    } catch (err) {
        console.error('Error opening folder dialog:', err);
        return null;
    }
});

// ============================================================
// WINDOW CONTROLS
// ============================================================

ipcMain.handle('window:minimize', () => mainWindow?.minimize());
ipcMain.handle('window:maximize', () => {
    if (mainWindow?.isMaximized()) {
        mainWindow.unmaximize();
    } else {
        mainWindow?.maximize();
    }
});
ipcMain.handle('window:close', () => mainWindow?.close());

// ============================================================
// APP LIFECYCLE
// ============================================================

app.whenReady().then(async () => {
    log('Initializing ORION 2.0...');

    // Initialize memory database
    await memory.init();
    log('[ORION] Memory engine ready');

    // Initialize NEW modules
    userProfile = new UserProfile(memory);
    log('[ORION] User profile loaded');

    contextManager = new ContextManager(memory);
    log('[ORION] Context manager ready');

    argumentEngine = new ArgumentEngine(settings);
    log('[ORION] Argument engine ready');

    suggestionEngine = new SuggestionEngine(memory, contextManager, userProfile);
    log('[ORION] Suggestion engine ready');

    goalTracker = new GoalTracker(memory, contextManager);
    log('[ORION] Goal tracker ready');

    // Connect user profile to personality
    personality.setUserProfile(userProfile.getUserInfo());

    // Check Ollama health
    await ollama.checkHealth();

    createWindow();

    app.on('activate', () => {
        if (BrowserWindow.getAllWindows().length === 0) {
            createWindow();
        }
    });

    console.log('\n╔═══════════════════════════════════════════════════════════╗');
    console.log('║              ORION 2.0 SYSTEM ONLINE                      ║');
    console.log('╚═══════════════════════════════════════════════════════════╝');
    console.log(`[ORION] Current Mode: ${personality.getModeDisplay()}`);
    console.log(`[ORION] All Modes: ${Object.keys(personality.modes).join(', ')}`);
    console.log(`[ORION] Ollama: ${ollama.available ? 'CONNECTED' : 'OFFLINE'} (${ollama.availableModels.length} models)`);
    console.log(`[ORION] Agents: ${agentRegistry.getAll().map(a => a.name).join(', ')}`);
    console.log(`[ORION] Argument Strength: ${argumentEngine.strength}`);
});

app.on('window-all-closed', () => {
    memory.close();
    if (process.platform !== 'darwin') {
        app.quit();
    }
});

app.on('before-quit', () => {
    memory.close();
});