/**
 * ORION — Preload Script v2
 * Secure bridge between renderer (UI) and main process (Node.js).
 * Exposes all ORION 2.0 APIs to the UI.
 */

const { contextBridge, ipcRenderer } = require('electron');

let currentSessionId = null;
let initialized = false;

// Helper function to get current session ID (always returns valid ID)
async function getCurrentSessionId() {
    try {
        console.log('[Preload] Getting current session...');

        // Try to get existing session
        let id = await ipcRenderer.invoke('orion:getCurrentSessionId');
        console.log('[Preload] Got session ID:', id);

        // If no session exists or invalid, create a new one
        if (!id || id === null || id === 0) {
            console.log('[Preload] No ID, creating new session...');
            id = await ipcRenderer.invoke('orion:createSession', 'New Chat', 1);
            console.log('[Preload] Created new session:', id);
            currentSessionId = id;
        }

        // Final fallback
        if (!id || id === null || id === 0) {
            console.log('[Preload] Using fallback ID 1');
            id = 1;
        }

        return id;
    } catch (err) {
        console.error('[Preload] Error getting session:', err);
        // Last resort - return 1 as fallback
        return 1;
    }
}

contextBridge.exposeInMainWorld('orion', {
    // ========================================
    // CORE CHAT
    // ========================================
    chat: async (message) => {
        console.log('[Preload] chat called with message:', message.substring(0, 50));
        const sessionId = await getCurrentSessionId();
        console.log('[Preload] Got sessionId:', sessionId);
        const result = await ipcRenderer.invoke('orion:chat', message, sessionId);
        console.log('[Preload] Got response:', result ? 'yes' : 'no');
        return result;
    },

    getHistory: async () => {
        const sessionId = await getCurrentSessionId();
        return ipcRenderer.invoke('orion:getHistory', sessionId);
    },

    getSessions: () => ipcRenderer.invoke('orion:getSessions'),

    createSession: (title, projectId = 1) => {
        return ipcRenderer.invoke('orion:createSession', title, projectId).then(id => {
            currentSessionId = id;
            return id;
        });
    },

    switchSession: async (sessionId) => {
        currentSessionId = sessionId;
        initialized = true;
        return ipcRenderer.invoke('orion:switchSession', sessionId);
    },

    renameSession: (sessionId, newTitle) => ipcRenderer.invoke('orion:renameSession', sessionId, newTitle),

    deleteSession: async (sessionId) => {
        const result = await ipcRenderer.invoke('orion:deleteSession', sessionId);
        if (currentSessionId === sessionId) {
            currentSessionId = null;
        }
        return result;
    },

    getCurrentSessionId: async () => {
        if (!initialized) {
            currentSessionId = await ipcRenderer.invoke('orion:getCurrentSessionId');
            initialized = true;
        }
        return currentSessionId;
    },

    // ========================================
    // MODE MANAGEMENT (NEW)
    // ========================================
    getMode: () => ipcRenderer.invoke('orion:getMode'),

    setMode: (modeName) => ipcRenderer.invoke('orion:setMode', modeName),

    // ========================================
    // USER PROFILE (NEW)
    // ========================================
    getProfile: (key) => ipcRenderer.invoke('profile:get', key),

    setProfile: (key, value, category) => ipcRenderer.invoke('profile:set', key, value, category),

    getAllProfile: () => ipcRenderer.invoke('profile:getAll'),

    getPreferences: () => ipcRenderer.invoke('profile:getPreferences'),

    updatePreferences: (updates) => ipcRenderer.invoke('profile:updatePreferences', updates),

    // ========================================
    // GOALS (NEW)
    // ========================================
    createGoal: (title, description, deadline) => ipcRenderer.invoke('goals:create', title, description, deadline),

    getGoals: () => ipcRenderer.invoke('goals:list'),

    getAllGoals: () => ipcRenderer.invoke('goals:getAll'),

    completeGoal: (goalId) => ipcRenderer.invoke('goals:complete', goalId),

    failGoal: (goalId, reason) => ipcRenderer.invoke('goals:fail', goalId, reason),

    deleteGoal: (goalId) => ipcRenderer.invoke('goals:delete', goalId),

    getGoalStats: () => ipcRenderer.invoke('goals:getStats'),

    getAgenda: () => ipcRenderer.invoke('goals:getAgenda'),

    getOverdueGoals: () => ipcRenderer.invoke('goals:getOverdue'),

    // ========================================
    // SUGGESTIONS (NEW)
    // ========================================
    getSuggestions: () => ipcRenderer.invoke('suggestions:get'),

    getProactiveSuggestions: () => ipcRenderer.invoke('suggestions:getProactive'),

    dismissSuggestion: (followUpId) => ipcRenderer.invoke('suggestions:dismiss', followUpId),

    createFollowUp: (context, minutes) => ipcRenderer.invoke('suggestions:createFollowUp', context, minutes),

    // ========================================
    // ARGUMENT ENGINE (NEW)
    // ========================================
    getArgumentStats: () => ipcRenderer.invoke('argument:getStats'),

    logArgumentResponse: (response) => ipcRenderer.invoke('argument:logResponse', response),

    critiquePlan: (plan) => ipcRenderer.invoke('argument:critiquePlan', plan),

    // ========================================
    // CONTEXT (NEW)
    // ========================================
    setActiveTask: (task, minutes) => ipcRenderer.invoke('context:setActiveTask', task, minutes),

    getActiveTask: () => ipcRenderer.invoke('context:getActiveTask'),

    clearActiveTask: () => ipcRenderer.invoke('context:clearActiveTask'),

    // ========================================
    // LEGACY
    // ========================================
    getStats: () => ipcRenderer.invoke('orion:getStats'),

    clearHistory: () => ipcRenderer.invoke('orion:clearHistory'),

    getAgents: () => ipcRenderer.invoke('orion:getAgents'),

    getOllama: () => ipcRenderer.invoke('orion:getOllama'),

    // ========================================
    // PROJECTS
    // ========================================
    createProject: (name, description) => ipcRenderer.invoke('project:create', name, description),
    getProjects: () => ipcRenderer.invoke('project:list'),
    getProject: (projectId) => ipcRenderer.invoke('project:get', projectId),
    updateProject: (projectId, name, description) => ipcRenderer.invoke('project:update', projectId, name, description),
    deleteProject: (projectId) => ipcRenderer.invoke('project:delete', projectId),

    // ========================================
    // FOLDERS
    // ========================================
    addFolder: (path, name) => ipcRenderer.invoke('folder:add', path, name),
    getFolders: () => ipcRenderer.invoke('folder:list'),
    setActiveFolder: (folderId) => ipcRenderer.invoke('folder:setActive', folderId),
    getActiveFolder: () => ipcRenderer.invoke('folder:getActive'),
    sendFolderToScrap: (folderId) => ipcRenderer.invoke('folder:sendToScrap', folderId),
    recoverFolder: (folderId) => ipcRenderer.invoke('folder:recover', folderId),
    deleteFolder: (folderId) => ipcRenderer.invoke('folder:delete', folderId),
    getForgottenFolders: () => ipcRenderer.invoke('folder:getForgotten'),
    selectFolder: () => ipcRenderer.invoke('folder:selectDialog'),

    // ========================================
    // WINDOW CONTROLS
    // ========================================
    minimize: () => ipcRenderer.invoke('window:minimize'),

    maximize: () => ipcRenderer.invoke('window:maximize'),

    close: () => ipcRenderer.invoke('window:close'),

    // ========================================
    // CLIENTS
    // ========================================
    createClient: (name, email, phone, notes) => ipcRenderer.invoke('client:create', name, email, phone, notes),
    listClients: () => ipcRenderer.invoke('client:list'),
    getClient: (clientId) => ipcRenderer.invoke('client:get', clientId),
    updateClient: (clientId, data) => ipcRenderer.invoke('client:update', clientId, data),
    deleteClient: (clientId) => ipcRenderer.invoke('client:delete', clientId),

    // ========================================
    // LEADS
    // ========================================
    createLead: (name, email, phone, company, stage, value, notes) =>
        ipcRenderer.invoke('lead:create', name, email, phone, company, stage, value, notes),
    listLeads: () => ipcRenderer.invoke('lead:list'),
    getLead: (leadId) => ipcRenderer.invoke('lead:get', leadId),
    updateLead: (leadId, data) => ipcRenderer.invoke('lead:update', leadId, data),
    deleteLead: (leadId) => ipcRenderer.invoke('lead:delete', leadId),

    // ========================================
    // STREAMING
    // ========================================
    onChunk: (callback) => {
        // Remove any existing listener to prevent duplicates on reload
        ipcRenderer.removeAllListeners('orion:chunk');
        ipcRenderer.on('orion:chunk', (event, chunk) => callback(chunk));
    }
});